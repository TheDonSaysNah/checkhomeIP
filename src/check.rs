use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Error};
use tokio::time::sleep;

use crate::mail;

pub struct CheckIP;

impl CheckIP {
    async fn check() -> Result<String, Error> {
        let apis = ["https://httpbin.org/ip", "https://api.ipify.org/?format=json", "https://api.seeip.org/jsonip"];

        for (k, v) in apis.iter().enumerate() {
            tracing::debug!("Sending HTTP request to {v}");
            match reqwest::get(*v).await {
                Ok(resp) => {
                    match resp.json::<HashMap<String, String>>().await {
                        Ok(r) => {
                            if k == 0 {
                                if let Some(ip) = r.get("origin") {
                                    return Ok(String::from(ip));
                                } else {
                                    tracing::error!("Failed to get IP from response!");
                                }
                            } else if let Some(ip) = r.get("ip") {
                                return Ok(String::from(ip));
                            } else {
                                tracing::error!("Failed to get IP from response!");
                            }
                        }
                        Err(e) =>{ tracing::error!("Failed to parse response as JSON: {}", e); }
                    }
                }
                Err(e) => tracing::error!("Failed to send request to remote server: {}", e),
            }
        }
        Err(anyhow!("Failed to complete request to all providers!"))
    }


    pub async fn init() {
        let mut old_time: u64;
        let mut current_ip: String = String::new();
        let wait_time = std::env::var("RECHECK_INTERVAL").unwrap().parse::<u64>().unwrap();

        // Get current IP and store in var
        tracing::info!("Getting initial IP");
        match Self::check().await {
            Ok(new_ip) => {
                if new_ip != current_ip {
                    current_ip = new_ip.clone();
                    tracing::info!("Initial IP set: {new_ip}");
                }
            }
            Err(e) => tracing::error!("Error: {}", e),
        }
        old_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        loop {
            if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() >= old_time + wait_time {
                match Self::check().await {
                    Ok(new_ip) => {
                        if new_ip != current_ip {
                            tracing::info!("Your home IP has changed from {current_ip} to {new_ip}");
                            mail::send_email(&current_ip, &new_ip).await;
                            current_ip = new_ip;
                        }
                        else { tracing::info!("IP hasn't changed. Ignoring") }
                    }
                    Err(e) => tracing::error!("Error: {}", e),
                }
                old_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            } else {
                tracing::debug!("Not checking IP due to {wait_time} seconds not passing");
                sleep(Duration::from_secs(1)).await
            }
        }
    }
}
