use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Error};
use local_ip_address::local_ip;
use reqwest::Client;
use tokio::time::sleep;
use crate::{notify, Args};

pub struct CheckIP;

impl CheckIP {
    async fn check(client: &Client) -> Result<String, Error> {
        let apis = ["https://httpbin.org/ip", "https://api.ipify.org/?format=json", "https://api.seeip.org/jsonip"];

        for (k, v) in apis.iter().enumerate() {
            tracing::debug!("Sending HTTP request to {v}");
            match client.get(*v).timeout(Duration::from_secs(10)).send().await {
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
                Err(e) => tracing::error!("Error: {}", e),
            }
        }
        Err(anyhow!("failed to complete request to all providers!"))
    }


    pub async fn init(app: Args) {
        let mut first = false;
        let mut old_time = 0;
        let mut current_ip: String = String::new();
        let client = reqwest::Client::new();
        let wait_time = std::env::var("RECHECK_INTERVAL").unwrap().parse::<u64>().unwrap();

        tracing::info!("Getting initial IP");
        loop {
            if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() >= old_time + wait_time {
                match Self::check(&client).await {
                    Ok(new_ip) => {
                        if new_ip != current_ip {
                            if !current_ip.is_empty() && first {
                                let body = format!("{} IP has changed from \"{current_ip}\" to \"{new_ip}\"", if app.client.is_some() { app.client.as_ref().unwrap() } else {"Your home"});
                                tracing::info!("Your home IP has changed from \"{current_ip}\" to \"{new_ip}\"");
                                if app.email { notify::send_email(body.clone(), None).await; }
                                if app.ntfy { notify::send_ntfy(&client, body, None).await; }
                            }
                            current_ip = new_ip.clone();
                        }
                        else if new_ip == current_ip && first { tracing::info!("IP hasn't changed. Ignoring") }

                        if !first { // Get current IP and store in var
                            tracing::info!("Initial IP set: \"{new_ip}\"");
                            if app.client.is_some() {
                                let localip = local_ip().unwrap();
                                tracing::info!("Local IP set at \"{localip}\"");
                                let body = format!("External IP \"{current_ip}\"\nLocal IP: {localip}");
                                if app.email { notify::send_email(body.clone(), app.client.clone()).await; }
                                if app.ntfy { notify::send_ntfy(&client, body, app.client.clone()).await; }

                            }
                            first = true;
                        }
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
