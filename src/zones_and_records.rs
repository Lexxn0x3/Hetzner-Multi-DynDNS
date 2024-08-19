use std::{collections::HashMap, sync::{Arc, Mutex}};

use serde::Deserialize;
use reqwest::Client;
use log::{info, warn};

use crate::{config::Config, ApiError};

#[derive(Deserialize, Debug)]
struct ZoneResponse {
    zones: Vec<Zone>,
}
#[derive(Deserialize, Debug)]
struct ZoneDetails {
    zone: Zone,
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
}

#[derive(Deserialize, Debug)]
struct Pagination {
}

#[derive(Deserialize, Debug)]
struct RecordsResponse {
    records: Vec<Record>,
}

#[derive(Deserialize, Debug)]
struct Record {
    r#type: String,
    id: String,
    name: String,
    value: String,
}
pub struct ZoneCache {
    map: Arc<Mutex<HashMap<String, String>>>,
}

impl ZoneCache {
    pub fn new() -> Self {
        ZoneCache {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_zone_name(&self, config: &Config, zone_id: &str) -> Result<String, ApiError> {
        let mut map = self.map.lock().unwrap();
        
        // Check if the zone ID is already in the map
        if let Some(zone_name) = map.get(zone_id) {
            return Ok(zone_name.clone());
        }
        log::debug!("zone id \"{}\" not in cache", zone_id);

        let client = Client::new();

        // If not, fetch the zone details from the API
        let url = format!("https://dns.hetzner.com/api/v1/zones/{}", zone_id);
        let response = client.get(&url)
            .header("Auth-API-Token", &config.api_token)
            .send()
            .await?;

        if response.status().is_success() {
            let zone_details: ZoneDetails = response.json().await.map_err(ApiError::RequestError)?;
            let zone_name = zone_details.zone.name.clone();
            map.insert(zone_id.to_string(), zone_name.clone());
            Ok(zone_name)
        } else {
            Err(ApiError::UnknownStatus(response.status().as_u16(), response.text().await.unwrap_or_default()))
        }
    }
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
