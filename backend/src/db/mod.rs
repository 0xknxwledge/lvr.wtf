pub mod aurora;
pub mod brontes;

use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    async fn connect(&self) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
    async fn is_connected(&self) -> bool;
}