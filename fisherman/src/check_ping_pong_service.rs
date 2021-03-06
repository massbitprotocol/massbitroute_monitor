use crate::fisherman_service::FishermanService;
use crate::{Config, CONFIG};
use anyhow::{Error, Result};
use futures::{stream, StreamExt};
use futures_util::TryStreamExt;
use log::{debug, info};
use mbr_check_component::check_module::check_module::ComponentInfo;
use reqwest::Client;
use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;

type SuccessRate = f32;

#[async_trait::async_trait]
pub trait CheckPingPong {
    async fn check_ping_pong(
        list_providers: Arc<RwLock<Vec<ComponentInfo>>>,
        domain: String,
    ) -> Result<HashMap<ComponentInfo, SuccessRate>>;
}

#[async_trait::async_trait]
impl CheckPingPong for FishermanService {
    async fn check_ping_pong(
        list_providers: Arc<RwLock<Vec<ComponentInfo>>>,
        domain: String,
    ) -> Result<HashMap<ComponentInfo, SuccessRate>> {
        // Copy list provider
        let mut list_providers_clone;
        {
            list_providers_clone = (*list_providers.read().await).clone();
        }
        let mut result: Arc<RwLock<HashMap<ComponentInfo, f32>>> =
            Arc::new(RwLock::new(HashMap::new()));

        // Parallel call
        let client = Client::new();
        for i in 1..CONFIG.ping_sample_number {
            let bodies = stream::iter(list_providers_clone.clone())
                .map(|component| {
                    let mut url = format!("http://{}/ping", component.ip);
                    let client = client.clone();
                    let domain = domain.clone();
                    tokio::spawn(async move {
                        let resp = client
                            .get(url)
                            .header("X-Api-Key", component.token.as_str())
                            .header("Host", component.get_host_header(&domain.to_string()))
                            .send()
                            .await?;
                        let resp = resp.text().await?;
                        let resp: Result<(ComponentInfo, String)> = Ok((component, resp));
                        resp
                    })
                })
                .buffer_unordered(CONFIG.ping_parallel_requests);
            bodies
                .for_each(|b| async {
                    match b {
                        Ok(Ok((component, resp))) => {
                            //debug!("Ping {} result: {}", component.id, resp);
                            if resp == CONFIG.ping_request_response {
                                *result.write().await.entry(component).or_insert(1f32) += 1f32;
                            }
                        }
                        Ok(Err(e)) => debug!("Got a reqwest::Error: {}", e),
                        Err(e) => debug!("Got a tokio::JoinError: {}", e),
                    }
                })
                .await;
        }
        let result = result
            .read()
            .await
            .iter()
            .filter_map(|(component, sum_success)| {
                let success_rate = sum_success / (CONFIG.ping_sample_number as f32);
                info!(
                    "component: {} {:?}, success rate: {}%",
                    component.id,
                    component.component_type,
                    success_rate * 100f32
                );
                if success_rate < CONFIG.ping_success_ratio_threshold {
                    Some((component.clone(), success_rate))
                } else {
                    None
                }
            })
            .collect();

        Ok(result)
    }
}
