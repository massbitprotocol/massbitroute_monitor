use crate::{CONFIG, ZONE};
use anyhow::Error;
use log::{debug, info};
use mbr_check_component::check_module::check_module::{
    CheckComponent, CheckMkReport, ComponentInfo, ComponentType,
};
use mbr_stats::chain_adapter::ChainAdapter;
use sp_keyring::AccountKeyring;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::convert::TryInto;
use std::sync::Arc;
use std::time::Duration;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::Pair;
use substrate_api_client::{compose_extrinsic, Api, UncheckedExtrinsicV4, XtStatus};

pub trait SubmitProviderReport {
    fn submit_provider_report(
        &self,
        provider_id: [u8; 36],
        requests: u64,
        success_percent: u32,
        response_time: u32,
    ) -> Result<(), anyhow::Error>;
}

impl SubmitProviderReport for ChainAdapter {
    fn submit_provider_report(
        &self,
        provider_id: [u8; 36],
        requests: u64,
        success_percent: u32,
        response_time: u32,
    ) -> Result<(), Error> {
        // set the recipient
        let api = self.api.as_ref().unwrap().clone();
        // the names are given as strings
        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
            api,
            &CONFIG.mvp_extrinsic_dapi,
            &CONFIG.mvp_extrinsic_submit_provider_report,
            provider_id,
            requests,
            success_percent,
            response_time
        );

        info!(
            "[+] Composed Extrinsic report provider {:?}:\n {:?}\n",
            provider_id, xt,
        );

        // send and watch extrinsic until InBlock
        let tx_hash = self
            .api
            .as_ref()
            .unwrap()
            .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
            .unwrap_or_default();
        info!("[+] Transaction got included. Hash: {:?}", tx_hash);
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct FishermanService {
    pub number_of_sample: u64,
    pub sample_interval_ms: u64,
    pub entry_point: String,
    pub check_component_service: CheckComponent,

    // For submit data
    pub mvp_url: String,
    pub signer_phrase: String,
    pub chain_adapter: Arc<ChainAdapter>,
    pub is_no_report: bool,
}

#[derive(Clone, Debug, Default)]
pub struct FishermanBuilder {
    inner: FishermanService,
}
impl FishermanBuilder {
    pub fn build(self) -> FishermanService {
        self.inner
    }
    pub fn with_number_of_sample(mut self, number: u64) -> FishermanBuilder {
        self.inner.number_of_sample = number;
        self
    }
    pub fn with_sample_interval_ms(mut self, number: u64) -> FishermanBuilder {
        self.inner.sample_interval_ms = number;
        self
    }
    pub async fn with_mvp_url(mut self, path: String) -> FishermanBuilder {
        self.inner.mvp_url = path.clone();
        // RPSee client for subscribe new block
        // let client = WsClientBuilder::default().build(&path).await;
        // println!("chain client: {:?}", client);

        // substrate_api_client for send extrinsic and subscribe event
        //let (signer,seed) = Pair::from_phrase(self.inner.signer_phrase.as_str(),None).expect("Wrong signer-phrase");
        // Fixme: find Ferdie Pair from phrase
        let signer = AccountKeyring::Ferdie.pair();

        let ws_client = WsRpcClient::new(&self.inner.mvp_url);
        let chain_adapter = ChainAdapter {
            json_rpc_client: None,
            ws_rpc_client: Some(ws_client.clone()),
            api: Api::new(ws_client.clone())
                .map(|api| api.set_signer(signer))
                .ok(),
        };
        self.inner.chain_adapter = Arc::new(chain_adapter);
        self
    }
    pub fn with_signer_phrase(mut self, signer_phrase: String) -> FishermanBuilder {
        self.inner.signer_phrase = signer_phrase;
        self
    }
    pub fn with_entry_point(mut self, url: String) -> FishermanBuilder {
        self.inner.entry_point = url;
        self
    }
    pub fn with_check_component_service(
        mut self,
        check_component_service: CheckComponent,
    ) -> FishermanBuilder {
        self.inner.check_component_service = check_component_service;
        self
    }
    pub fn with_chain_adapter(mut self, chain_adapter: Arc<ChainAdapter>) -> FishermanBuilder {
        self.inner.chain_adapter = chain_adapter;
        self
    }
    pub fn with_no_report(mut self, no_report_mode: bool) -> Self {
        self.inner.is_no_report = no_report_mode;
        self
    }
}

impl FishermanService {
    pub fn builder() -> FishermanBuilder {
        FishermanBuilder::default()
    }

    fn is_history_continuous_fail_reach_limit(
        reports_history: &VecDeque<HashMap<ComponentInfo, ComponentReport>>,
        component: &ComponentInfo,
    ) -> bool {
        let mut count = 0;
        for reports in reports_history.iter() {
            if let Some(report) = reports.get(component) {
                if !report.is_healthy(&component.component_type) {
                    count += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        println!("Number continuous-fails of id {}: {}", component.id, count);
        match component.component_type {
            ComponentType::Node => count >= CONFIG.node_response_failed_number,
            ComponentType::Gateway => count >= CONFIG.gateway_response_failed_number,
            ComponentType::DApi => false,
        }
    }

    pub async fn loop_check_component(mut self) {
        let number_of_sample = self.number_of_sample;
        info!("number_of_sample:{}", number_of_sample);
        let sample_interval_ms = self.sample_interval_ms;
        info!("sample_interval_ms:{}", sample_interval_ms);
        let mut reports_history: VecDeque<HashMap<ComponentInfo, ComponentReport>> =
            VecDeque::new();

        info!("Run check component");
        loop {
            // Load new nodes/gateways list
            if let Err(e) = self
                .check_component_service
                .reload_components_list(Some(&CONFIG.checking_component_status), &ZONE)
                .await
            {
                info!("reload_components_list error: {:?}", e);
                tokio::time::sleep(Duration::from_millis(CONFIG.delay_between_check_loop_ms)).await;
                continue;
            };
            info!(
                "Reload list node: {:?}",
                self.check_component_service.list_nodes
            );
            info!(
                "Reload list gateway: {:?}",
                self.check_component_service.list_gateways
            );

            let mut average_reports: HashMap<ComponentInfo, ComponentReport> = HashMap::new();
            let mut collect_reports: HashMap<ComponentInfo, Vec<CheckMkReport>> = HashMap::new();
            for n in 0..number_of_sample {
                info!("Run {} times", n + 1);
                if let Ok(reports) = self
                    .check_component_service
                    .check_components(&CONFIG.check_task_list_fisherman)
                    .await
                {
                    debug!("reports:{:?}", reports);
                    for (component, report) in reports {
                        // with each component collect reports in to vector
                        match collect_reports.entry(component) {
                            Entry::Occupied(o) => {
                                o.into_mut().push(report);
                            }
                            Entry::Vacant(v) => {
                                v.insert(vec![report]);
                            }
                        }
                    }
                };

                tokio::time::sleep(Duration::from_millis(sample_interval_ms)).await;
            }

            debug!("collect_reports: {:?}", collect_reports);
            // Calculate average report
            for (component, reports) in collect_reports.iter() {
                info!("component:{:?}", component.id);
                average_reports.insert(component.clone(), ComponentReport::from(reports));
            }

            // Display report for debug
            for (component, report) in average_reports.iter() {
                info!("id: {}, type: {:?}, chain {:?}, request_number: {}, success_number: {}, response_time_ms:{:?}ms, healthy: {}",
                    component.id,
                    component.component_type,
                    component.blockchain,
                    report.request_number,
                    report.success_number,
                    report.response_time_ms,
                    report.is_healthy(&component.component_type)
                );
            }

            if !self.is_no_report {
                // Check and send report
                for (component_info, report) in average_reports.iter() {
                    // Check for healthy and submit report
                    if !report.is_healthy(&component_info.component_type)
                        && component_info.status == "staked"
                        && Self::is_history_continuous_fail_reach_limit(
                            &reports_history,
                            &component_info,
                        )
                    {
                        info!("Submit report: {:?}", component_info);
                        let provider_id: [u8; 36] =
                            component_info.id.as_bytes().try_into().unwrap();
                        info!("provider_id: {:?}", String::from_utf8_lossy(&provider_id));
                        // Submit report
                        if let Err(e) = self
                            .chain_adapter
                            .submit_provider_report(
                                provider_id,
                                report.request_number,
                                report.get_success_percent(),
                                report.response_time_ms.unwrap_or_default(),
                            )
                            .and_then(|_| {
                                match component_info.component_type {
                                    ComponentType::Node => {
                                        self.check_component_service
                                            .list_nodes
                                            .retain(|component| *component.id != component_info.id);
                                    }
                                    ComponentType::Gateway => {
                                        self.check_component_service
                                            .list_gateways
                                            .retain(|component| *component.id != component_info.id);
                                    }
                                    _ => {}
                                }
                                info!("list_nodes:{:?}", self.check_component_service.list_nodes);
                                info!(
                                    "list_gateways:{:?}",
                                    self.check_component_service.list_gateways
                                );
                                Ok(())
                            })
                        {
                            info!("submit_provider_report error:{:?}", e);
                        }
                    }
                }
            }

            // Store to history
            reports_history.push_back(average_reports);
            while reports_history.len() > CONFIG.reports_history_queue_length_max {
                reports_history.pop_front();
            }

            tokio::time::sleep(Duration::from_millis(CONFIG.delay_between_check_loop_ms)).await;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ComponentReport {
    request_number: u64,
    success_number: u64,
    response_time_ms: Option<u32>,
}

impl ComponentReport {
    pub fn is_healthy(&self, component_type: &ComponentType) -> bool {
        let response_threshold = match component_type {
            ComponentType::Node => CONFIG.node_response_time_threshold,
            ComponentType::Gateway => CONFIG.gateway_response_time_threshold,
            _ => u32::default(),
        };
        // If there is not enough info return false
        if self.success_number == 0 || self.response_time_ms == None {
            return false;
        }
        (self.get_success_percent() >= CONFIG.success_percent_threshold)
            && (self.response_time_ms.unwrap() <= response_threshold)
    }
    pub fn get_success_percent(&self) -> u32 {
        if self.request_number > 0 {
            (self.success_number * 100 / self.request_number) as u32
        } else {
            0
        }
    }
}

impl From<&Vec<CheckMkReport>> for ComponentReport {
    fn from(reports: &Vec<CheckMkReport>) -> Self {
        let check_number = CONFIG.number_of_samples;
        let mut success_number = 0;
        let response_times = reports
            .iter()
            .filter_map(|report| {
                if report.is_component_status_ok() {
                    success_number += 1;
                }
                if let Some(response_time) =
                    report.metric.metric.get(&*CONFIG.response_time_key_name)
                {
                    Some(serde_json::from_value(response_time.clone()).unwrap())
                } else {
                    None
                }
            })
            .collect::<Vec<u32>>();
        let number_of_call = response_times.len() as u32;
        let response_time_ms = match number_of_call == 0 {
            true => None,
            false => Some(response_times.into_iter().sum::<u32>() / number_of_call),
        };

        ComponentReport {
            request_number: check_number,
            success_number,
            response_time_ms,
        }
    }
}
