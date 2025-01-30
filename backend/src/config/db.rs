use crate::Error;
use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AuroraConfig {
    pub gcp_host: String,
    pub public_host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub connection_timeout: u64,
    pub retry_interval: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrontesConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub connection_timeout: u64,
    pub retry_interval: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConnectionConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub connection_timeout: u64,
    pub retry_interval: u64,
}

impl AuroraConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            gcp_host: env::var("AURORA_GCP_HOST").unwrap_or_else(|_| "dummy_gcp_host".to_string()),
            public_host: env::var("AURORA_PUBLIC_HOST").unwrap_or_else(|_| "dummy_public_host".to_string()),
            port: env::var("AURORA_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .map_err(|_| Error::Config("Invalid AURORA_PORT format".to_string()))?,
            user: env::var("AURORA_USER").unwrap_or_else(|_| "dummy_user".to_string()),
            password: env::var("AURORA_PASSWORD").unwrap_or_else(|_| "dummy_password".to_string()),
            database: env::var("AURORA_DATABASE").unwrap_or_else(|_| "dummy_database".to_string()),
            connection_timeout: env::var("AURORA_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            retry_interval: env::var("AURORA_RETRY_INTERVAL")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        })
    }
    
    pub fn get_host_for_environment(&self) -> String {
        // If running locally (determined by environment variable), use public host
        if env::var("RUNNING_LOCALLY").unwrap_or_default() == "true" {
            self.public_host.clone()
        } else {
            self.gcp_host.clone()
        }
    }
}

impl BrontesConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: env::var("BRONTES_HOST").unwrap_or_else(|_| "dummy_host".to_string()),
            port: env::var("BRONTES_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .map_err(|_| Error::Config("Invalid BRONTES_PORT format".to_string()))?,
            user: env::var("BRONTES_USER").unwrap_or_else(|_| "dummy_user".to_string()),
            password: env::var("BRONTES_PASSWORD").unwrap_or_else(|_| "dummy_password".to_string()),
            connection_timeout: env::var("BRONTES_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            retry_interval: env::var("BRONTES_RETRY_INTERVAL")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub aurora: AuroraConfig,
    pub brontes: BrontesConfig,
}

impl DatabaseConfig {
    pub fn new() -> Result<Self> {
        Ok(Self {
            aurora: AuroraConfig::from_env()?,
            brontes: BrontesConfig::from_env()?,
        })
    }
}