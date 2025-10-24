use crate::models::{
    Backend, Cluster, Frontend, Query, RuntimeInfo,
};
use crate::utils::{ApiError, ApiResult};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;


pub struct StarRocksClient {
    pub http_client: Client,
    pub cluster: Cluster,
}

impl StarRocksClient {
    pub fn new(cluster: Cluster) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(cluster.connection_timeout as u64))
            .build()
            .unwrap_or_default();

        Self {
            http_client,
            cluster,
        }
    }

    pub fn get_base_url(&self) -> String {
        let protocol = if self.cluster.enable_ssl {
            "https"
        } else {
            "http"
        };
        format!(
            "{}://{}:{}",
            protocol, self.cluster.fe_host, self.cluster.fe_http_port
        )
    }

    // Get backends via HTTP API
    pub async fn get_backends(&self) -> ApiResult<Vec<Backend>> {
        let url = format!("{}/api/show_proc?path=/backends", self.get_base_url());
        tracing::debug!("Fetching backends from: {}", url);

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.cluster.username, Some(&self.cluster.password_encrypted))
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch backends: {}", e);
                ApiError::cluster_connection_failed(format!("Request failed: {}", e))
            })?;

        if !response.status().is_success() {
            tracing::error!("Backends API returned error status: {}", response.status());
            return Err(ApiError::cluster_connection_failed(format!(
                "HTTP status: {}",
                response.status()
            )));
        }

        let data: Value = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse backends response: {}", e);
            ApiError::cluster_connection_failed(format!("Failed to parse response: {}", e))
        })?;

        // Try new format (direct array) first, then fall back to old format
        if let Ok(backends) = serde_json::from_value::<Vec<Backend>>(data.clone()) {
            tracing::debug!("Retrieved {} backends using new format", backends.len());
            return Ok(backends);
        }

        // Fallback to old format
        let backends = Self::parse_proc_result::<Backend>(&data)?;
        tracing::debug!("Retrieved {} backends using old format", backends.len());
        Ok(backends)
    }

    // Execute SQL command via HTTP API
    pub async fn execute_sql(&self, sql: &str) -> ApiResult<()> {
        let url = format!("{}/api/query", self.get_base_url());
        tracing::debug!("Executing SQL: {}", sql);

        let body = serde_json::json!({
            "query": sql
        });

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.cluster.username, Some(&self.cluster.password_encrypted))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute SQL: {}", e);
                ApiError::cluster_connection_failed(format!("Request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("SQL execution failed with status {}: {}", status, error_text);
            return Err(ApiError::cluster_connection_failed(
                format!("SQL execution failed: {}", error_text)
            ));
        }

        tracing::info!("SQL executed successfully: {}", sql);
        Ok(())
    }

    // Drop backend node
    pub async fn drop_backend(&self, host: &str, heartbeat_port: &str) -> ApiResult<()> {
        let sql = format!("ALTER SYSTEM DROP backend \"{}:{}\"", host, heartbeat_port);
        tracing::info!("Dropping backend: {}:{}", host, heartbeat_port);
        self.execute_sql(&sql).await
    }

    // Get frontends via HTTP API
    pub async fn get_frontends(&self) -> ApiResult<Vec<Frontend>> {
        let url = format!("{}/api/show_proc?path=/frontends", self.get_base_url());
        tracing::debug!("Fetching frontends from: {}", url);

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.cluster.username, Some(&self.cluster.password_encrypted))
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch frontends: {}", e);
                ApiError::cluster_connection_failed(format!("Request failed: {}", e))
            })?;

        if !response.status().is_success() {
            tracing::error!("Frontends API returned error status: {}", response.status());
            return Err(ApiError::cluster_connection_failed(format!(
                "HTTP status: {}",
                response.status()
            )));
        }

        let data: Value = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse frontends response: {}", e);
            ApiError::cluster_connection_failed(format!("Failed to parse response: {}", e))
        })?;

        // Try new format (direct array) first, then fall back to old format
        if let Ok(frontends) = serde_json::from_value::<Vec<Frontend>>(data.clone()) {
            return Ok(frontends);
        }

        // Fallback to old format
        let frontends = Self::parse_proc_result::<Frontend>(&data)?;
        Ok(frontends)
    }

    // Get current queries
    pub async fn get_queries(&self) -> ApiResult<Vec<Query>> {
        let url = format!(
            "{}/api/show_proc?path=/current_queries",
            self.get_base_url()
        );

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.cluster.username, Some(&self.cluster.password_encrypted))
            .send()
            .await
            .map_err(|e| ApiError::cluster_connection_failed(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ApiError::cluster_connection_failed(format!(
                "HTTP status: {}",
                response.status()
            )));
        }

        let data: Value = response.json().await.map_err(|e| {
            ApiError::cluster_connection_failed(format!("Failed to parse response: {}", e))
        })?;

        // Try new format (direct array) first, then fall back to old format
        if let Ok(queries) = serde_json::from_value::<Vec<Query>>(data.clone()) {
            return Ok(queries);
        }

        // Fallback to old format
        let queries = Self::parse_proc_result::<Query>(&data)?;
        Ok(queries)
    }

    // Get runtime info
    pub async fn get_runtime_info(&self) -> ApiResult<RuntimeInfo> {
        let url = format!("{}/api/show_runtime_info", self.get_base_url());

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.cluster.username, Some(&self.cluster.password_encrypted))
            .send()
            .await
            .map_err(|e| ApiError::cluster_connection_failed(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ApiError::cluster_connection_failed(format!(
                "HTTP status: {}",
                response.status()
            )));
        }

        let runtime_info: RuntimeInfo = response.json().await.map_err(|e| {
            ApiError::cluster_connection_failed(format!("Failed to parse response: {}", e))
        })?;

        Ok(runtime_info)
    }

    // Get metrics in Prometheus format
    pub async fn get_metrics(&self) -> ApiResult<String> {
        let url = format!("{}/metrics", self.get_base_url());

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.cluster.username, Some(&self.cluster.password_encrypted))
            .send()
            .await
            .map_err(|e| ApiError::cluster_connection_failed(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ApiError::cluster_connection_failed(format!(
                "HTTP status: {}",
                response.status()
            )));
        }

        let metrics_text = response.text().await.map_err(|e| {
            ApiError::cluster_connection_failed(format!("Failed to read response: {}", e))
        })?;

        Ok(metrics_text)
    }


    // Parse PROC result format
    fn parse_proc_result<T: serde::de::DeserializeOwned>(data: &Value) -> ApiResult<Vec<T>> {
        // StarRocks PROC result format: {"columnNames": [...], "rows": [[...]]}
        let column_names = data["columnNames"]
            .as_array()
            .ok_or_else(|| ApiError::internal_error("Invalid PROC result format"))?;

        let rows = data["rows"]
            .as_array()
            .ok_or_else(|| ApiError::internal_error("Invalid PROC result format"))?;

        let mut results = Vec::new();

        for row in rows {
            let row_array = row
                .as_array()
                .ok_or_else(|| ApiError::internal_error("Invalid row format"))?;

            // Create a JSON object from column names and row values
            let mut obj = serde_json::Map::new();
            for (i, col_name) in column_names.iter().enumerate() {
                if let Some(col_name_str) = col_name.as_str()
                    && let Some(value) = row_array.get(i) {
                        obj.insert(col_name_str.to_string(), value.clone());
                    }
            }

            let item: T = serde_json::from_value(Value::Object(obj))
                .map_err(|e| ApiError::internal_error(format!("Failed to parse item: {}", e)))?;

            results.push(item);
        }

        Ok(results)
    }

    // Parse Prometheus metrics format
    pub fn parse_prometheus_metrics(&self, metrics_text: &str) -> ApiResult<std::collections::HashMap<String, f64>> {
        let mut metrics = std::collections::HashMap::new();

        for line in metrics_text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse format: metric_name{labels} value
            if let Some((name_part, value_str)) = line.rsplit_once(' ')
                && let Ok(value) = value_str.parse::<f64>() {
                    // Extract metric name (before '{' or the whole name_part)
                    let metric_name = if let Some(pos) = name_part.find('{') {
                        &name_part[..pos]
                    } else {
                        name_part
                    };
                    
                    metrics.insert(metric_name.to_string(), value);
                }
        }

        Ok(metrics)
    }
}

