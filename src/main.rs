mod config;
mod zones_and_records;

use log::LevelFilter;
use reqwest::{Client, Error};
use serde_json::json;
use tokio::time::sleep;
use zones_and_records::ZoneCache;
use std::time::Duration;
use crate::config::{Config, RecordConfig};
use std::fmt;

#[derive(Debug)]
enum ApiError {
    RequestError(reqwest::Error),
    UnknownStatus(u16, String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::RequestError(e) => write!(f, "Request error: {}", e),
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
    env_logger::Builder::new()
        .filter(None, LevelFilter::Info) // Set the log level here
        .init();
        // Load configuration
    let config = match Config::from_file("config.toml") {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load configuration: {:?}", e);
            return;
        }
    };
    
    // Check if records are empty
    if config.records.is_empty() {
        log::warn!("No records defined in the configuration. Fetching all zones and records...");
        if let Err(e) = zones_and_records::fetch_zones_and_records(&config).await {
            log::error!("Failed to fetch zones and records: {}", e);
        }
        return; // Exit after fetching zones and records if no records are defined
    }

    let mut last_ip = String::new();
    let zone_cache = ZoneCache::new();
    loop {
        match get_external_ip().await {
            Ok(current_ip) => {
                if current_ip != last_ip {
                    log::info!("IP changed from {} to {}", last_ip, current_ip);

                    for record in &config.records {
                        match update_dns_record(&config, record, &current_ip, &zone_cache).await {
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

async fn update_dns_record(config: &Config, record: &RecordConfig, ip: &str, zone_cache: &ZoneCache) -> Result<(), ApiError> {
    // Get the zone name from the cache or fetch it if necessary
    let zone_name = zone_cache.get_zone_name(config, &record.zone_id).await?;

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

    if response.status().is_success() {
        let full_domain = format!("{}.{}", record.name, zone_name);
        log::info!("Updated DNS record: {} to IP: {}", full_domain, ip);
        Ok(())
    } else {
        log::error!("Failed to update DNS record: {}. Status: {}", record.name, response.status());
        Err(ApiError::UnknownStatus(response.status().as_u16(), response.text().await.unwrap_or_default()))
    }
}
