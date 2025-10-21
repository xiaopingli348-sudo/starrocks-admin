use crate::models::cluster::Cluster;
use crate::utils::error::ApiResult;
use mysql_async::{Pool, OptsBuilder, SslOpts};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manager for MySQL connection pools using mysql_async
/// Maintains a pool for each cluster to avoid reconnecting on every query
#[derive(Clone)]
pub struct MySQLPoolManager {
    pools: Arc<RwLock<HashMap<i64, Pool>>>,
}

impl MySQLPoolManager {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MySQLPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MySQLPoolManager {
    /// Get or create a connection pool for the given cluster
    pub async fn get_pool(&self, cluster: &Cluster) -> ApiResult<Pool> {
        let cluster_id = cluster.id;

        // Try to get existing pool
        {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(&cluster_id) {
                return Ok(pool.clone());
            }
        }

        // Create new pool using mysql_async
        // StarRocks uses MySQL protocol, mysql_async is more compatible than sqlx
        let opts = OptsBuilder::default()
            .ip_or_hostname(&cluster.fe_host)
            .tcp_port(cluster.fe_query_port as u16)
            .user(Some(&cluster.username))
            .pass(Some(&cluster.password_encrypted))
            .db_name(None::<String>) // No default database
            .prefer_socket(false) // Disable socket preference for StarRocks compatibility
            .ssl_opts(None::<SslOpts>) // No SSL for now
            .pool_opts(
                mysql_async::PoolOpts::default()
                    .with_constraints(
                        mysql_async::PoolConstraints::new(1, 10)
                            .unwrap()
                    )
            );

        let pool = Pool::new(opts);

        // NOTE: Removed test connection! It was causing connection pool corruption
        // The first actual query will validate connectivity

        tracing::info!(
            "Created MySQL connection pool for cluster {} ({}:{})",
            cluster_id,
            cluster.fe_host,
            cluster.fe_query_port
        );

        // Store pool
        {
            let mut pools = self.pools.write().await;
            pools.insert(cluster_id, pool.clone());
        }

        Ok(pool)
    }

    /// Remove a pool for a specific cluster (useful when cluster is deleted or updated)
    pub async fn remove_pool(&self, cluster_id: i64) {
        let mut pools = self.pools.write().await;
        if let Some(pool) = pools.remove(&cluster_id) {
            drop(pool); // Pool will be closed when all references are dropped
            tracing::info!("Removed MySQL connection pool for cluster {}", cluster_id);
        }
    }

    /// Clear all pools (useful for cleanup)
    pub async fn clear_all(&self) {
        let mut pools = self.pools.write().await;
        pools.clear();
        tracing::info!("Cleared all MySQL connection pools");
    }
}

