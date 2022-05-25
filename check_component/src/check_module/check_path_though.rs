use crate::check_module::check_module::ComponentType::Node;
use crate::check_module::check_module::{
    ActionResponse, CheckMkStatus, ComponentInfo, ComponentType,
};
use crate::{CONFIG, HASH_TEST_20K};
use clap::Arg;
use futures_util::future::join_all;
use log::{debug, info, warn};
use reqwest::header::HeaderValue;
use reqwest::{Error, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ActionCallListPath {
    action_type: String,
    request_type: String,
    header: HashMap<String, String>,
    time_out: usize,
    return_fields: Vec<ReturnField>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
struct ReturnField {
    data_from: ResponsePart,
    field_name: Option<String>,
    field_data: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
enum ResponsePart {
    Header,
    Body,
}

impl Default for ResponsePart {
    fn default() -> Self {
        ResponsePart::Header
    }
}

impl ActionCallListPath {
    fn get_url(gateway: &ComponentInfo, node: &ComponentInfo) -> String {
        format!("https://{}/_node/{}/ping", gateway.ip, node.id)
    }
    fn get_host(component: &ComponentInfo, domain: &String) -> String {
        match component.component_type {
            ComponentType::Node => {
                format!("{}.node.mbr.{}", component.id, domain)
            }
            ComponentType::Gateway => {
                format!("{}.gw.mbr.{}", component.id, domain)
            }
            ComponentType::DApi => Default::default(),
        }
    }

    pub async fn call_action_list_path(
        &self,
        check_component: ComponentInfo,
        components: Vec<ComponentInfo>,
        domain: String,
    ) -> Result<ActionResponse, anyhow::Error> {
        let return_name = "call_action_list_path".to_string();
        let check_component_org = Arc::new(check_component);
        let domain_org = Arc::new(domain);
        info!(
            "Check path from {:?} to: {:?}",
            check_component_org, components
        );
        // Run in parallel
        let mut tasks = Vec::new();
        for component in components {
            let return_fields = self.return_fields.clone();
            let check_component = check_component_org.clone();
            let domain = domain_org.clone();
            let task = tokio::spawn(async move {
                let mut success = true;
                let mut conclude = CheckMkStatus::Ok;
                let mut message: String = String::new();
                info!(
                    "Check path from {} {} to {} {}",
                    check_component.component_type.to_string(),
                    check_component.id,
                    component.component_type.to_string(),
                    component.id
                );
                // Create url
                let url = match check_component.component_type {
                    ComponentType::Node => Self::get_url(&component, &check_component),
                    ComponentType::Gateway => Self::get_url(&check_component, &component),
                    ComponentType::DApi => Default::default(),
                };

                // Create client
                let host = Self::get_host(&check_component, &domain);
                debug!("host: {}", host);
                let client_builder = reqwest::ClientBuilder::new();
                let client = client_builder
                    .danger_accept_invalid_certs(true)
                    .build()
                    .unwrap();
                let res_data = reqwest::Client::builder()
                    .danger_accept_invalid_certs(true)
                    .timeout(Duration::from_millis(CONFIG.check_path_timeout_ms))
                    .build()
                    .unwrap()
                    .get(url)
                    .header("Host", host)
                    .send()
                    .await;

                // Process response data
                match res_data {
                    Ok(res_data) => {
                        let body = res_data.text().await.ok();

                        for return_field in return_fields.iter() {
                            match return_field.data_from {
                                // ResponsePart::Header => {
                                //     match return_field.field_name.as_ref().unwrap().as_str() {
                                //         "X-Mbr-Checksum" => {
                                //             if let Some(value) = res_data.headers().get(
                                //                 return_field.field_name.as_ref().unwrap().to_string(),
                                //             ) {
                                //                 let checksum = value.to_str().unwrap();
                                //                 if checksum != HASH_TEST_20K.as_str() {
                                //                     conclude = CheckMkStatus::Critical;
                                //                     message.push_str(&format!(
                                //                         "{} {} has wrong checksum {} in path check.",
                                //                         component.component_type.to_string(),
                                //                         component.id,
                                //                         checksum
                                //                     ));
                                //                 };
                                //             } else {
                                //                 conclude = CheckMkStatus::Critical;
                                //                 message.push_str(&format!(
                                //                     "{} {} has no checksum in path check.",
                                //                     component.component_type.to_string(),
                                //                     component.id
                                //                 ));
                                //             }
                                //         }
                                //         _ => {}
                                //     }
                                // }
                                ResponsePart::Body => match body.clone() {
                                    Some(body) => {
                                        if body != *return_field.field_data.as_ref().unwrap() {
                                            conclude = CheckMkStatus::Critical;
                                            success = false;
                                            message.push_str(&format!(
                                                "{} {} has wrong return data `{}` in path check.",
                                                component.component_type.to_string(),
                                                component.id,
                                                body
                                            ));
                                        }
                                    }
                                    None => {
                                        conclude = CheckMkStatus::Critical;
                                        success = false;
                                        message.push_str(&format!(
                                            "{} {} has no body in path check.",
                                            component.component_type.to_string(),
                                            component.id,
                                        ));
                                    }
                                },
                                _ => {
                                    warn!("Not support return_field")
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let error_mess = format!(
                            "{} {} cannot be connected, error: {}.",
                            component.component_type.to_string(),
                            component.id,
                            e
                        );
                        warn!("{}", error_mess);
                        conclude = CheckMkStatus::Critical;
                        success = false;
                        message.push_str(&error_mess);
                    }
                }
                (conclude, success, message)
            });
            tasks.push(task);
        }
        let responses = join_all(tasks).await;
        let mut success = true;
        let mut conclude = CheckMkStatus::Ok;
        let mut message: String = String::new();
        for res in responses {
            match res {
                Ok((sub_conclude, sub_success, sub_message)) => {
                    success = success && sub_success;
                    if sub_conclude == CheckMkStatus::Critical {
                        conclude = CheckMkStatus::Critical
                    }
                    message.push_str(sub_message.as_str());
                }
                Err(err) => {
                    success = false;
                    conclude = CheckMkStatus::Critical;
                    message.push_str(format!("Error: {}", err).as_str());
                }
            }
        }

        let action_res = ActionResponse {
            success,
            conclude,
            return_name,
            result: HashMap::new(),
            message,
        };
        debug!("check path action_res: {:?}", action_res);

        Ok(action_res)
    }
}
