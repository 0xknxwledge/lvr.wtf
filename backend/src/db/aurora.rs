use crate::config::AuroraConfig;
use crate::Error;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use mysql_async::{params, Pool, PoolConstraints, PoolOpts, SslOpts};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info, warn};
use crate::DatabaseConnection;
use mysql_async::prelude::Queryable;

#[derive(Debug, Deserialize, Clone)]
pub struct LVRDetails {
    pub block_number: u64,
    pub details: String,
    pub index: u32,
}

pub struct AuroraConnection {
    pools: Arc<DashMap<u64, Pool>>, // Map index to its own pool
    config: AuroraConfig,
    reconnect_attempts: u32,
    reconnect_delay: std::time::Duration,
}

impl AuroraConnection {
    pub fn new(config: AuroraConfig) -> Result<Self> {
        Ok(Self {
            pools: Arc::new(DashMap::new()),
            config,
            reconnect_attempts: 3,
            reconnect_delay: std::time::Duration::from_secs(5),
        })
    }

    async fn get_or_create_pool(&self, index: u64) -> Result<(Pool, bool)> {
        if let Some(pool) = self.pools.get(&index) {
            return Ok((pool.clone(), false));
        }

        let pool = self.create_pool().await?;
        self.pools.insert(index, pool.clone());
        Ok((pool, true))
    }

    async fn create_pool(&self) -> Result<Pool> {
        let host = self.config.get_host_for_environment();
        info!("Creating connection pool with configuration:");
        info!(
            "Host: {}, Port: {}, Database: {}",
            host, self.config.port, self.config.database
        );
    
        let pool_constraints = PoolConstraints::new(0, 12).context("Failed to create pool constraints")?;
        let pool_opts = PoolOpts::default().with_constraints(pool_constraints);
    
        let opts = mysql_async::OptsBuilder::default()
            .ip_or_hostname(host)
            .tcp_port(self.config.port)
            .user(Some(self.config.user.clone()))
            .pass(Some(self.config.password.clone()))
            .db_name(Some(self.config.database.clone()))
            .ssl_opts(SslOpts::default().with_danger_accept_invalid_certs(true))
            .pool_opts(pool_opts);
    
        let pool = Pool::new(opts);
    
        match pool.get_conn().await {
            Ok(_) => {
                info!("Successfully established test connection to database");
                Ok(pool)
            }
            Err(e) => {
                error!("Failed to establish test connection: {}", e);
                Err(anyhow!("Failed to verify connection: {}", e))
            }
        }
    }

    pub async fn fetch_lvr_details(
        &self,
        index: u64,
        chunk_start: u64,
        chunk_end: u64,
    ) -> Result<Vec<LVRDetails>> {
        info!(
            "Starting LVR details fetch for index {} from block {} to {}",
            index, chunk_start, chunk_end
        );

        let mut all_results: Vec<LVRDetails> = Vec::new();
        let batch_size: u64 = 7200;
        let mut current_start = chunk_start;
        let mut attempts = 0;
        let total_blocks = chunk_end - chunk_start;
        let total_batches = (total_blocks as f64 / batch_size as f64).ceil() as u64;
        let mut completed_batches = 0;

        while current_start < chunk_end {
            attempts += 1;
            let current_end = std::cmp::min(current_start + batch_size, chunk_end);

            let (pool, created) = self.get_or_create_pool(index).await?;
            if created {
                info!("Created pool for markout time index {}.", index);
            } else {
                info!("Reusing pool for markout time index {}.", index);
            }

            match self
                .try_fetch_lvr_details_batch(&pool, index, current_start, current_end)
                .await
            {
                Ok(batch_results) => {
                    let batch_count = batch_results.len();
                    all_results.extend(batch_results);
                    current_start = current_end;
                    attempts = 0;
                    completed_batches += 1;

                    info!(
                        "Completed batch {}/{} for index {} ({:.1}% complete). Retrieved {} records. Total records so far: {}",
                        completed_batches,
                        total_batches,
                        index,
                        (completed_batches as f64 / total_batches as f64) * 100.0,
                        batch_count,
                        all_results.len()
                    );
                }
                Err(e) => {
                    if attempts >= self.reconnect_attempts {
                        error!(
                            "Failed to fetch LVR details after {} attempts for index {} (batch {}/{}, blocks {}-{}): {}",
                            self.reconnect_attempts,
                            index,
                            completed_batches + 1,
                            total_batches,
                            current_start,
                            current_end,
                            e
                        );
                        return Err(Error::Database(format!(
                            "Failed to fetch LVR details batch after {} attempts: {}",
                            self.reconnect_attempts, e
                        ))
                        .into());
                    }

                    warn!(
                        "Attempt {} to fetch LVR details batch {}-{} failed: {}. Retrying in {} seconds...",
                        attempts,
                        current_start,
                        current_end,
                        e,
                        self.reconnect_delay.as_secs()
                    );

                    tokio::time::sleep(self.reconnect_delay).await;
                }
            }
        }

        info!(
            "Completed fetching all LVR details for index {}. Retrieved {} total records across {} batches",
            index,
            all_results.len(),
            total_batches
        );

        Ok(all_results)
    }

    async fn try_fetch_lvr_details_batch(
        &self,
        pool: &Pool,
        index: u64,
        batch_start: u64,
        batch_end: u64,
    ) -> Result<Vec<LVRDetails>> {
        let mut conn = pool
            .get_conn()
            .await
            .context("Failed to get connection from pool")?;

        let query = r"
            SELECT blockNumber, details, `index`
            FROM t_lvr
            WHERE blockNumber > :batch_start AND blockNumber <= :batch_end
            AND details IS NOT NULL
            AND `index` = :index
            ORDER BY blockNumber ASC, `index` ASC
        ";

        info!(
            "Executing query for index {} with parameters: batch_start={}, batch_end={}, index={}",
            index, batch_start, batch_end, index
        );

        let start_time = std::time::Instant::now();

        let params = params! {
            "batch_start" => batch_start,
            "batch_end" => batch_end,
            "index" => index,
        };

        let result: Vec<LVRDetails> = conn
            .exec_map(
                query,
                params,
                |(block_number, details, index): (u64, String, u32)| LVRDetails {
                    block_number,
                    details,
                    index,
                },
            )
            .await
            .with_context(|| {
                format!(
                    "Failed to execute LVR details query with parameters: batch_start={}, batch_end={}, index={}",
                    batch_start, batch_end, index
                )
            })?;

        let elapsed = start_time.elapsed();

        info!(
            "Fetched LVR data for index {} for block range {}-{} ({} records) in {:?}",
            index,
            batch_start,
            batch_end,
            result.len(),
            elapsed
        );

        Ok(result)
    }
}


#[async_trait]
impl DatabaseConnection for AuroraConnection {
    async fn connect(&self) -> Result<()> {
        // Create an initial test pool to verify connectivity
        for attempt in 0..self.reconnect_attempts {
            match self.create_pool().await {
                Ok(pool) => {
                    // Test the connection
                    if pool.get_conn().await.is_ok() {
                        // Store this as a default pool with index 0
                        self.pools.insert(0, pool);
                        return Ok(());
                    }
                }
                Err(e) => {
                    if attempt == self.reconnect_attempts - 1 {
                        return Err(e.context("Failed to connect after maximum attempts"));
                    }
                    tokio::time::sleep(self.reconnect_delay).await;
                }
            }
        }

        Err(anyhow!("Failed to connect after maximum attempts"))
    }

    async fn disconnect(&self) -> Result<()> {
        // Clear all pools
        self.pools.clear();
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        // Check if any pool is connected
        if let Some(pool) = self.pools.get(&0) {
            pool.get_conn().await.is_ok()
        } else {
            false
        }
    }
}
