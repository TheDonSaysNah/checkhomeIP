use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Error};
use tokio::time::sleep;

use crate::mail;

const WAIT_TIME: u64 = 300;
pub struct CheckIP;

impl CheckIP {
    async fn check() -> Result<String, Error> {
        tracing::trace!("Sending HTTP request...");
        match reqwest::get("https://httpbin.org/ip").await {
            Ok(resp) => {
                match resp.json::<HashMap<String, String>>().await {
                    Ok(r) => {
                        if let Some(ip) = r.get("origin") {
                            Ok(String::from(ip))
                        } else {
                            Err(anyhow!("failed to get IP from response!"))
                        }
                    }
                    Err(e) => Err(anyhow!("failed to parse response as JSON: {e}")),
                }
            }
            Err(e) => Err(anyhow!("failed to send request to remote server: {e}")),
        }
    }


    pub async fn init() {
        let mut old_time: u64 = 0;
        let mut current_ip: String = String::new();

        // Get current IP and store in var
        match Self::check().await {
            Ok(new_ip) => {
                if new_ip != current_ip {
                    current_ip = new_ip.clone();
                    tracing::info!("Initial IP set: {new_ip}");
                    old_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                }
            }
            Err(e) => tracing::error!("Error: {}", e),
        }

        loop {
            if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() >= old_time + WAIT_TIME {
                match Self::check().await {
                    Ok(new_ip) => {
                        if new_ip != current_ip {
                            tracing::info!("Your home IP has changed from {current_ip} to {new_ip}");
                            mail::send_email(&current_ip, &new_ip).await;
                            current_ip = new_ip;
                        }
                        else {
                            tracing::info!("IP hasn't changed. Ignoring")
                        }
                        old_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    }
                    Err(e) => tracing::error!("Error: {}", e),
                }
            } else {
                tracing::debug!("Not checking IP since {WAIT_TIME} secs not passed");
                sleep(Duration::from_secs(1)).await
            }
        }
    }
}
