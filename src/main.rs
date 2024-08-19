mod config;
mod zones_and_records;

use log::{error, warn};
use reqwest::{Client, Error};
use serde_json::json;
use tokio::time::sleep;
use std::time::Duration;
use crate::config::{Config, RecordConfig};
use std::fmt;

#[derive(Debug)]
enum ApiError {
    RequestError(reqwest::Error),
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
    UnknownStatus(u16, String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::RequestError(e) => write!(f, "Request error: {}", e),
            ApiError::Unauthorized => write!(f, "Unauthorized: Invalid API token"),
            ApiError::Forbidden => write!(f, "Forbidden: Access denied"),
            ApiError::NotFound => write!(f, "Not Found: Record or zone not found"),
            ApiError::Conflict => write!(f, "Conflict: Record conflict"),
            ApiError::UnprocessableEntity => write!(f, "Unprocessable Entity: Invalid data"),
            ApiError::UnknownStatus(code, msg) => write!(f, "Unknown status {}: {}", code, msg),
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::RequestError(err)
    }
}

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Load configuration
    let config = match Config::from_file("config.toml") {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load configuration: {:?}", e);
            return;
        }
    };

    // Set the log level
    log::set_max_level(match config.log_level.to_lowercase().as_str() {
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    });

    // Check if records are empty
    if config.records.is_empty() {
        warn!("No records defined in the configuration. Fetching all zones and records...");
        if let Err(e) = zones_and_records::fetch_zones_and_records(&config).await {
            error!("Failed to fetch zones and records: {}", e);
        }
        return; // Exit after fetching zones and records if no records are defined
    }

    let mut last_ip = String::new();

    loop {
        match get_external_ip().await {
            Ok(current_ip) => {
                if current_ip != last_ip {
                    log::info!("IP changed from {} to {}", last_ip, current_ip);

                    for record in &config.records {
                        match update_dns_record(&config, record, &current_ip).await {
                            Ok(_) => {}
                            Err(e) => log::error!("Failed to update record {}: {}", record.name, e),
                        }
                    }

                    last_ip = current_ip;
                }
            }
            Err(e) => log::error!("Failed to get external IP: {:?}", e),
        }

        sleep(Duration::from_secs(config.interval_secs)).await;
    }
}

#[derive(Debug)]
enum IpError {
    RequestError(()),
    InvalidIp(()),
}

impl From<Error> for IpError {
    fn from(_: Error) -> Self {
        IpError::RequestError(())
    }
}

async fn get_external_ip() -> Result<String, IpError> {
    let response = reqwest::get("https://api.ipify.org").await?;
    let ip = response.text().await?;

    if ip.parse::<std::net::IpAddr>().is_ok() {
        Ok(ip)
    } else {
        Err(IpError::InvalidIp(()))
    }
}

async fn update_dns_record(config: &Config, record: &RecordConfig, ip: &str) -> Result<(), ApiError> {
    let client = Client::new();
    let url = format!("https://dns.hetzner.com/api/v1/records/{}", record.record_id);

    let body = json!({
        "name": record.name,
        "ttl": record.ttl,
        "type": record.record_type,
        "value": ip,
        "zone_id": record.zone_id,
    });

    let response = client.put(&url)
        .header("Auth-API-Token", &config.api_token)
        .json(&body)
        .send()
        .await?;

    match response.status().as_u16() {
        200 => {
            log::info!("Updated DNS record: {} to IP: {}", record.name, ip);
            Ok(())
        }
        401 => Err(ApiError::Unauthorized),
        403 => Err(ApiError::Forbidden),
        404 => Err(ApiError::NotFound),
        409 => Err(ApiError::Conflict),
        422 => Err(ApiError::UnprocessableEntity),
        code => Err(ApiError::UnknownStatus(code, response.text().await.unwrap_or_default())),
    }
}
