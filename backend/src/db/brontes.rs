use crate::config::BrontesConfig;
use crate::DatabaseConnection;
use crate::Error;
use crate::BRONTES_ADDRESSES;
use async_trait::async_trait;
use clickhouse::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use tracing::{warn,info, error};

#[derive(Debug, Deserialize, Clone)]
pub struct LVRAnalysis {
    pub pool_address: String,
    pub block_number: u64,
    pub lvr: f64,
}


pub struct BrontesConnection {
    client: Arc<Mutex<Option<Client>>>,
    config: BrontesConfig,
    reconnect_attempts: u32,
    reconnect_delay: std::time::Duration,
}

impl BrontesConnection {
    pub fn new(config: BrontesConfig) -> Result<Self> {
        Ok(Self {
            client: Arc::new(Mutex::new(None)),
            config,
            reconnect_attempts: 3,
            reconnect_delay: std::time::Duration::from_secs(5),
        })
    }

    async fn create_client(&self) -> Result<Client> {
        let url = format!("http://{}:{}",
            self.config.host,
            self.config.port
        );
    
        Ok(Client::default()
            .with_url(url)
            .with_user(self.config.user.clone())
            .with_password(self.config.password.clone()))
    }

    pub async fn fetch_lvr_analysis(&self, chunk_start: u64, chunk_end: u64) -> Result<Vec<LVRAnalysis>> {
        info!(
            "Starting LVR analysis fetch from block {} to {}", 
            chunk_start, chunk_end
        );

        let mut all_results = Vec::new();
        let batch_size: u64 = 7200;
        let mut current_start = chunk_start;
        let mut attempts = 0;
        let total_blocks = chunk_end - chunk_start;
        let total_batches = (total_blocks as f64 / batch_size as f64).ceil() as u64;
        let mut completed_batches = 0;

        while current_start < chunk_end {
            attempts += 1;
            let current_end = std::cmp::min(current_start + batch_size, chunk_end);
            let client = self.get_or_create_client().await?;

            match self.try_fetch_lvr_analysis_batch(&client, current_start, current_end).await {
                Ok(batch_results) => {
                    let batch_count = batch_results.len();
                    all_results.extend(batch_results);
                    current_start = current_end;
                    attempts = 0;
                    completed_batches += 1;

                    info!(
                        "Completed batch {}/{} ({:.1}% complete). Retrieved {} records. Total records so far: {}", 
                        completed_batches,
                        total_batches,
                        (completed_batches as f64 / total_batches as f64) * 100.0,
                        batch_count,
                        all_results.len()
                    );
                },
                Err(e) => {
                    if attempts >= self.reconnect_attempts {
                        error!(
                            "Failed to fetch LVR analysis after {} attempts (batch {}/{}, blocks {}-{}): {}", 
                            self.reconnect_attempts,
                            completed_batches + 1,
                            total_batches,
                            current_start,
                            current_end,
                            e
                        );
                        return Err(Error::Database(format!(
                            "Failed to fetch LVR analysis batch after {} attempts: {}", 
                            self.reconnect_attempts,
                            e
                        )).into());
                    }

                    warn!(
                        "Attempt {} to fetch LVR analysis batch {}-{} failed: {}. Retrying in {} seconds...",
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
            "Completed fetching all LVR analysis. Retrieved {} total records across {} batches",
            all_results.len(),
            total_batches
        );

        Ok(all_results)
    }

    async fn try_fetch_lvr_analysis_batch(&self, client: &Client, batch_start: u64, batch_end: u64) -> Result<Vec<LVRAnalysis>> {    
        // De-checksum the addresses
        let pools: Vec<_> = BRONTES_ADDRESSES.iter().map(|&s| s).collect();
        let mut cursor = client
            .query(
                r#"
                SELECT 
                    p.profit AS pool_address,
                    block_number,
                    SUM(p.profit_amt + p.revenue_amt) AS lvr
                FROM brontes.block_analysis
                ARRAY JOIN cex_dex_arbed_pool_all AS p
                WHERE p.profit in (?)
                    AND run_id = 1000
                    AND p.profit != '0x0000000000000000000000000000000000000000'
                    AND p.revenue != '0x0000000000000000000000000000000000000000'
                    AND block_number > ?
                    AND block_number <= ?
                GROUP BY block_number, pool_address
                ORDER BY block_number ASC
                "#
            )
            .bind(pools)
            .bind(batch_start)
            .bind(batch_end)
            .fetch::<(String, u64, f64)>()?;

        info!(
            "Executing query for block range {}-{}", 
            batch_start, batch_end
        );

        let mut results = Vec::new();
        while let Some((pool_address, block_number, lvr)) = cursor.next().await? {
            results.push(LVRAnalysis {
                pool_address,
                block_number,
                lvr,
            });
        }

        info!(
            "Retrieved {} records for block range {}-{}",
            results.len(),
            batch_start,
            batch_end
        );
    
        Ok(results)
    }

    async fn get_or_create_client(&self) -> Result<Client> {
        let mut client_guard = self.client.lock().await;
        if client_guard.is_none() {
            *client_guard = Some(self.create_client().await?);
        }
        Ok(client_guard.as_ref().unwrap().clone())
    }
}

#[async_trait]
impl DatabaseConnection for BrontesConnection {
    async fn connect(&self) -> Result<()> {
        let mut current_attempt = 0;
        
        while current_attempt < self.reconnect_attempts {
            match self.create_client().await {
                Ok(client) => {
                    let mut client_guard = self.client.lock().await;
                    *client_guard = Some(client);
                    return Ok(());
                }
                Err(e) => {
                    current_attempt += 1;
                    if current_attempt == self.reconnect_attempts {
                        return Err(e);
                    }
                    tokio::time::sleep(self.reconnect_delay).await;
                }
            }
        }
        
        Err(anyhow::anyhow!("Failed to connect after max attempts"))
    }
    
    async fn disconnect(&self) -> Result<()> {
        let mut client_guard = self.client.lock().await;
        *client_guard = None;
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        let client_guard = self.client.lock().await;
        if let Some(client) = &*client_guard {
            match client.query("SELECT 1 as value")
                .fetch::<u8>()
            {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }
}