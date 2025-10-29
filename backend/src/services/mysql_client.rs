use crate::utils::error::ApiError;
use mysql_async::{Pool, prelude::Queryable};
use std::sync::Arc;

#[derive(Clone)]
pub struct MySQLClient {
    pool: Arc<Pool>,
}

impl MySQLClient {
    pub fn from_pool(pool: Pool) -> Self {
        Self { pool: Arc::new(pool) }
    }

    /// Execute a query and return results as (column_names, rows)
    /// Optionally set catalog and database context before executing the query
    pub async fn query_raw(
        &self,
        sql: &str,
        catalog: Option<&str>,
        database: Option<&str>,
    ) -> Result<(Vec<String>, Vec<Vec<String>>), ApiError> {
        tracing::debug!("Getting MySQL connection from pool...");
        let mut conn = self.pool.get_conn().await.map_err(|e| {
            tracing::error!("Failed to get connection from pool: {}", e);
            ApiError::cluster_connection_failed(format!("Failed to get connection: {}", e))
        })?;

        // First, set catalog if provided (on the same connection)
        // Note: StarRocks may not support USE CATALOG via MySQL protocol
        // If catalog is the default catalog or USE CATALOG fails, we'll continue anyway
        if let Some(cat) = catalog {
            if !cat.is_empty() && cat != "default_catalog" {
                // Try USE CATALOG without backticks (StarRocks syntax)
                let use_catalog_sql = format!("USE CATALOG {}", cat);
                tracing::debug!("Executing USE CATALOG on same connection: {}", use_catalog_sql);
                if let Err(e) = conn.query::<mysql_async::Row, _>(&use_catalog_sql).await {
                    // If USE CATALOG fails (not supported or already active), continue with query
                    // The default catalog is usually already active
                    tracing::warn!("USE CATALOG {} failed (may not be supported): {}, continuing anyway", cat, e);
                    // Don't fail - continue with the query, it might work in the default catalog context
                }
            }
            // If catalog is "default_catalog" or empty, no need to switch
        }

        // Then, set database if provided (on the same connection)
        if let Some(db) = database {
            if !db.is_empty() {
                let use_db_sql = format!("USE {}", db);
                tracing::debug!("Executing USE DATABASE on same connection: {}", use_db_sql);
                if let Err(e) = conn.query::<mysql_async::Row, _>(&use_db_sql).await {
                    tracing::warn!("Failed to execute USE DATABASE {}: {}", db, e);
                    drop(conn);
                    return Err(ApiError::internal_error(format!(
                        "Failed to switch to database {}: {}",
                        db, e
                    )));
                }
            }
        }

        tracing::debug!("Executing MySQL query: '{}'", sql);
        let rows: Vec<mysql_async::Row> = conn.query(sql).await.map_err(|e| {
            tracing::error!("MySQL query execution failed: {}", e);
            ApiError::internal_error(format!("SQL execution failed: {}", e))
        })?;

        tracing::debug!("Query returned {} rows", rows.len());

        let mut result_rows = Vec::new();
        let mut columns = Vec::new();

        if !rows.is_empty() {
            // Extract column names from first row
            for col in rows[0].columns_ref().iter() {
                columns.push(col.name_str().to_string());
            }
            tracing::debug!("Column names: {:?}", columns);

            // Extract data from all rows
            for (row_idx, row) in rows.iter().enumerate() {
                let mut row_data = Vec::new();
                let col_count = row.columns_ref().len();

                for col_idx in 0..col_count {
                    // Directly access Value using Index trait to avoid FromValue conversion
                    let value_str = value_to_string(&row[col_idx], row_idx, col_idx);
                    row_data.push(value_str);
                }

                // Log first few rows for debugging
                if row_idx < 3 {
                    let preview: Vec<_> = row_data.iter().take(3).collect();
                    tracing::debug!("Row {}: {:?}", row_idx, preview);
                }

                result_rows.push(row_data);
            }
        } else {
            tracing::debug!("Query returned no rows");
        }

        // CRITICAL: Explicitly drop connection to ensure proper cleanup
        drop(conn);
        tracing::debug!("Connection returned to pool");

        Ok((columns, result_rows))
    }

    /// Execute a query and return results as Vec<serde_json::Value> (JSON objects)
    /// Each row is a JSON object with column names as keys
    pub async fn query(&self, sql: &str) -> Result<Vec<serde_json::Value>, ApiError> {
        let (column_names, rows) = self.query_raw(sql, None, None).await?;

        let mut result = Vec::new();
        for row in rows {
            let mut obj = serde_json::Map::new();
            for (i, col_name) in column_names.iter().enumerate() {
                if let Some(value) = row.get(i) {
                    obj.insert(col_name.clone(), serde_json::Value::String(value.clone()));
                }
            }
            result.push(serde_json::Value::Object(obj));
        }

        Ok(result)
    }

    pub async fn execute(&self, sql: &str) -> Result<u64, ApiError> {
        tracing::debug!("Executing MySQL statement: '{}'", sql);

        let mut conn = self.pool.get_conn().await.map_err(|e| {
            tracing::error!("Failed to get connection for execute: {}", e);
            ApiError::cluster_connection_failed(format!("Failed to get connection: {}", e))
        })?;

        let result: Vec<mysql_async::Row> = conn.query(sql).await.map_err(|e| {
            tracing::error!("MySQL execute failed: {}", e);
            ApiError::cluster_connection_failed(format!("Query failed: {}", e))
        })?;

        tracing::debug!("Execute affected {} rows", result.len());

        // CRITICAL: Explicitly drop connection to ensure proper cleanup
        drop(conn);

        Ok(result.len() as u64)
    }
}

// Convert mysql_async::Value directly to String without using FromValue trait
// This avoids panic when encountering NULL values
fn value_to_string(value: &mysql_async::Value, row_idx: usize, col_idx: usize) -> String {
    match value {
        mysql_async::Value::NULL => {
            tracing::debug!("Row {}, Column {}: NULL value", row_idx, col_idx);
            "NULL".to_string()
        },
        mysql_async::Value::Bytes(bytes) => {
            let result = String::from_utf8_lossy(bytes).to_string();
            if bytes.len() > 100 {
                tracing::debug!(
                    "Row {}, Column {}: Bytes {} bytes, preview: {:?}",
                    row_idx,
                    col_idx,
                    bytes.len(),
                    result.chars().take(50).collect::<String>()
                );
            }
            result
        },
        mysql_async::Value::Int(i) => {
            tracing::debug!("Row {}, Column {}: Int {}", row_idx, col_idx, i);
            i.to_string()
        },
        mysql_async::Value::UInt(u) => {
            tracing::debug!("Row {}, Column {}: UInt {}", row_idx, col_idx, u);
            u.to_string()
        },
        mysql_async::Value::Float(f) => {
            tracing::debug!("Row {}, Column {}: Float {}", row_idx, col_idx, f);
            f.to_string()
        },
        mysql_async::Value::Double(d) => {
            tracing::debug!("Row {}, Column {}: Double {}", row_idx, col_idx, d);
            d.to_string()
        },
        mysql_async::Value::Date(year, month, day, hour, minute, second, _micro) => {
            let result = format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hour, minute, second
            );
            tracing::debug!("Row {}, Column {}: Date {}", row_idx, col_idx, result);
            result
        },
        mysql_async::Value::Time(_neg, days, hours, minutes, seconds, _micro) => {
            let result = format!("{}:{:02}:{:02}", days * 24 + (*hours as u32), minutes, seconds);
            tracing::debug!("Row {}, Column {}: Time {}", row_idx, col_idx, result);
            result
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mysql_async::OptsBuilder;

    async fn get_test_cluster() -> crate::models::Cluster {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:///tmp/starrocks-admin/starrocks-admin.db".to_string());

        let pool = sqlx::SqlitePool::connect(&db_url).await.unwrap();
        let cluster: crate::models::Cluster = sqlx::query_as("SELECT * FROM clusters WHERE id = 2")
            .fetch_one(&pool)
            .await
            .unwrap();
        pool.close().await;

        cluster
    }

    async fn create_test_pool() -> Pool {
        let cluster = get_test_cluster().await;

        let opts = OptsBuilder::default()
            .ip_or_hostname(&cluster.fe_host)
            .tcp_port(cluster.fe_query_port as u16)
            .user(Some(&cluster.username))
            .pass(Some(&cluster.password_encrypted))
            .db_name(None::<String>)
            .prefer_socket(false)
            .pool_opts(
                mysql_async::PoolOpts::default()
                    .with_constraints(mysql_async::PoolConstraints::new(1, 10).unwrap()),
            );

        Pool::new(opts)
    }

    #[tokio::test]
    async fn test_simple_query() {
        let pool = create_test_pool().await;
        let client = MySQLClient::from_pool(pool);

        let (columns, rows) = client.query_raw("SELECT 1", None, None).await.unwrap();
        assert_eq!(columns.len(), 1);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "1");
    }

    #[tokio::test]
    async fn test_string_query() {
        let pool = create_test_pool().await;
        let client = MySQLClient::from_pool(pool);

        let (_columns, rows) = client.query_raw("SELECT 'hello'", None, None).await.unwrap();
        assert_eq!(rows[0][0], "hello");
    }

    #[tokio::test]
    async fn test_get_query_profile_stability() {
        let query_id = "7d663e42-ae29-11f0-8a21-9eb34e998e27";

        let pool = create_test_pool().await;
        let client = MySQLClient::from_pool(pool);

        let mut _success_count = 0;
        let mut _empty_count = 0;

        for _i in 1..=50 {
            let sql = format!("SELECT get_query_profile('{}')", query_id);
            let result = client.query_raw(&sql, None, None).await;

            match result {
                Ok((_, rows)) => {
                    if rows.is_empty() || rows[0].is_empty() || rows[0][0].is_empty() {
                        _empty_count += 1;
                    } else {
                        _success_count += 1;
                    }
                },
                Err(_) => {
                    _empty_count += 1;
                },
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }

    #[tokio::test]
    async fn test_partitions_meta_with_null() {
        let pool = create_test_pool().await;
        let client = MySQLClient::from_pool(pool);

        // Test the actual problematic query
        let sql = "select * from information_schema.partitions_meta order by Max_CS LIMIT 5";
        let result = client.query_raw(sql, None, None).await;

        match result {
            Ok((_columns, rows)) => {
                assert!(!rows.is_empty(), "Expected non-empty result");
            },
            Err(e) => {
                panic!("Query failed: {}", e);
            },
        }
    }
}
