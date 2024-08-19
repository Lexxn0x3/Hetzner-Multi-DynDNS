use serde::Deserialize;
use reqwest::{Client, Error};
use log::{info, warn, error};

use crate::{config::Config, ApiError};

#[derive(Deserialize, Debug)]
struct ZoneResponse {
    zones: Vec<Zone>,
    meta: Meta,
}

#[derive(Deserialize, Debug)]
struct Zone {
    id: String,
    name: String,
    ns: Vec<String>,
    owner: String,
    status: String,
    ttl: u64,
    records_count: u64,
    is_secondary_dns: bool,
}

#[derive(Deserialize, Debug)]
struct Meta {
    pagination: Pagination,
}

#[derive(Deserialize, Debug)]
struct Pagination {
    page: u64,
    per_page: u64,
    last_page: u64,
    total_entries: u64,
}

#[derive(Deserialize, Debug)]
struct RecordsResponse {
    records: Vec<Record>,
}

#[derive(Deserialize, Debug)]
struct Record {
    r#type: String,
    id: String,
    zone_id: String,
    name: String,
    value: String,
    ttl: u64,
}

pub async fn fetch_zones_and_records(config: &Config) -> Result<(), ApiError> {
    let client = Client::new();
    let zones_url = "https://dns.hetzner.com/api/v1/zones";
    let records_url = "https://dns.hetzner.com/api/v1/records";

    // Fetch zones
    let zones_response = client.get(zones_url)
        .header("Auth-API-Token", &config.api_token)
        .send()
        .await?
        .json::<ZoneResponse>()
        .await
        .map_err(ApiError::RequestError)?;

    if zones_response.zones.is_empty() {
        warn!("No zones found.");
    } else {
        for zone in zones_response.zones {
            info!("Zone: {} (ID: {})", zone.name, zone.id);
            info!("  Status: {}", zone.status);
            info!("  NS: {:?}", zone.ns);
            info!("  Owner: {}", zone.owner);
            info!("  Records Count: {}", zone.records_count);
            info!("  TTL: {}", zone.ttl);
            info!("  Is Secondary DNS: {}", zone.is_secondary_dns);

            // Fetch records for each zone
            let records_response = client.get(records_url)
                .header("Auth-API-Token", &config.api_token)
                .query(&[("zone_id", &zone.id)])
                .send()
                .await?
                .json::<RecordsResponse>()
                .await
                .map_err(ApiError::RequestError)?;

            if records_response.records.is_empty() {
                warn!("No records found for zone: {}", zone.name);
            } else {
                for record in records_response.records {
                    info!("  Record: {} (ID: {}, Type: {}, Value: {})", record.name, record.id, record.r#type, record.value);
                }
            }
        }
    }

    Ok(())
}
