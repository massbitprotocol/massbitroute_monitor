use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::chain_adapter::{ChainAdapter, Projects};

use jsonrpsee::core::client::{
    Subscription, SubscriptionClientT,
};

use jsonrpsee::ws_client::WsClientBuilder;
use jsonrpsee::{rpc_params};
use log::{info};
use prometheus_http_query::aggregations::{sum};

use prometheus_http_query::{Aggregate, Client as PrometheusClient, InstantVector, Selector};
use regex::Regex;
use serde_json::{Value};


use sp_keyring::AccountKeyring;
use std::convert::TryInto;
use std::fmt::Formatter;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::Api;

use tokio::sync::RwLock;
use tokio::task;
use tokio::time::{sleep, Duration};

type TimeStamp = i64;

const NUMBER_BLOCK_FOR_COUNTING: isize = 2;
const DATA_NAME: &str = "nginx_vts_filter_requests_total";
const PROJECT_FILTER: &str = ".*::proj::api_method";
const PROJECT_FILTER_PROJECT_ID_REGEX: &str =
    r"[0-9a-fA-F]{8}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{12}";
const UPDATE_PROJECT_QUOTA_INTERVAL: u64 = 10; //sec

#[derive(Debug)]
pub enum ComponentType {
    Node,
    Dapi,
    Gateway,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ConfigData;

#[derive(Debug, Default)]
pub struct ComponentStats {
    // input file
    pub config_data_uri: String,
    // config data
    pub config_data: Option<ConfigData>,
    // Url
    pub prometheus_gateway_url: String,
    pub prometheus_node_url: String,
    // Massbit verification protocol chain url
    pub mvp_url: String,
    pub signer_phrase: String,

    // For collecting data
    pub gateway_adapter: Arc<DataCollectionAdapter>,
    pub node_adapter: Arc<DataCollectionAdapter>,
    // Projects data
    pub list_project_url: String,
    pub projects: Arc<RwLock<Projects>>,

    // For submit data
    pub chain_adapter: Arc<ChainAdapter>,
}

#[derive(Clone, Default)]
pub struct DataCollectionAdapter {
    client: Option<PrometheusClient>,
}

impl std::fmt::Debug for DataCollectionAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataCollectionAdapter<>")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct QueryData {
    name: String,
    tags: Vec<String>,
    value: isize,
    from_block_number: isize,
    to_block_number: isize,
    from_timestamp: i64,
    to_timestamp: i64,
    is_keep: bool,
}

pub struct StatsBuilder {
    inner: ComponentStats,
}

impl Default for StatsBuilder {
    fn default() -> Self {
        Self {
            inner: ComponentStats {
                config_data_uri: "".to_string(),
                config_data: None,
                prometheus_gateway_url: "".to_string(),
                prometheus_node_url: "".to_string(),
                mvp_url: "".to_string(),
                signer_phrase: "".to_string(),
                gateway_adapter: Default::default(),
                node_adapter: Default::default(),
                list_project_url: "".to_string(),
                projects: Default::default(),
                chain_adapter: Default::default(),
            },
        }
    }
}

impl ComponentStats {
    pub fn builder() -> StatsBuilder {
        StatsBuilder::default()
    }

    pub async fn get_projects_quota_list(
        projects: Arc<RwLock<Projects>>,
        list_project_url: &String,
        status: &str,
    ) -> Result<(), anyhow::Error> {
        let res = reqwest::get(list_project_url).await?.text().await?;
        let mut projects_new: Projects = serde_json::from_str(res.as_str())?;
        // Filter by status
        projects_new
            .0
            .retain(|_id, project| project.status == *status);
        {
            let mut lock = projects.write().await;
            lock.0 = projects_new.0;
            info!("projects quota update by portal: {:?}", lock.0);
        }

        Ok(())
    }

    fn capture_project_id(filter: &str) -> Option<String> {
        let re = Regex::new(PROJECT_FILTER_PROJECT_ID_REGEX).unwrap();
        let project_id = re
            .captures(filter)
            .and_then(|id| id.get(0).and_then(|id| Some(id.as_str().to_string())));
        //info!("filter: {}, project_id:{:?}",filter, project_id);
        project_id
    }

    async fn get_request_number(
        &self,
        filter_name: &str,
        filter_value: &str,
        data_name: &str,
        time: TimeStamp,
    ) -> anyhow::Result<HashMap<String, usize>> {
        let q: InstantVector = Selector::new()
            .metric(data_name)
            .regex_match(filter_name, filter_value)
            .with("code", "2xx")
            .try_into()?;
        let q = sum(q, Some(Aggregate::By(&["filter"])));
        let response = self
            .gateway_adapter
            .client
            .as_ref()
            .ok_or(anyhow::Error::msg("None client"))?
            .query(q, Some(time), None)
            .await?;

        let res = if let Some(instant) = response.as_instant() {
            Ok(instant
                .iter()
                .filter_map(|iv| {
                    iv.metric().get("filter").and_then(|filter| {
                        let value = iv.sample().value() as usize;
                        // Fixme: get project ID in the filter string
                        let project = ComponentStats::capture_project_id(filter.as_str())
                            .and_then(|project_id| Some((project_id.to_string(), value)));
                        info!("project: {:?}", project);
                        project
                    })
                })
                .collect::<HashMap<String, usize>>())
        } else {
            Err(anyhow::Error::msg("Cannot parse response"))
        };
        //info!("Hashmap res: {:#?}", res);
        res
    }

    async fn subscribe_finalized_heads(&self) -> Result<Subscription<Value>, anyhow::Error> {
        let client = self
            .chain_adapter
            .json_rpc_client
            .as_ref()
            .ok_or(anyhow::Error::msg("None chain client"))?;
        let subscribe_finalized_heads = client
            .subscribe::<Value>(
                "chain_subscribeFinalizedHeads",
                rpc_params![],
                "chain_unsubscribeFinalizedHeads",
            )
            .await;
        info!("subscribe_finalized_heads: {:?}", subscribe_finalized_heads);
        Ok(subscribe_finalized_heads?)
    }

    async fn loop_get_request_number(
        &self,
        mut subscribe: Subscription<Value>,
    ) -> Result<(), anyhow::Error> {
        // wait for new block
        let mut last_count_block: isize = -1;

        loop {
            let res = subscribe.next().await;
            if let Some(Ok(res)) = res {
                info!("received {:?}", res);
                if let Some(block_number) = res.get("number") {
                    let block_number = isize::from_str_radix(
                        block_number.as_str().unwrap().trim_start_matches("0x"),
                        16,
                    )?;
                    info!("block_number {:?}", block_number);
                    if last_count_block == -1 {
                        last_count_block = block_number;
                        continue;
                    } else {
                        if block_number - last_count_block >= NUMBER_BLOCK_FOR_COUNTING {
                            // Set new count block
                            let current_count_block = last_count_block + NUMBER_BLOCK_FOR_COUNTING;
                            // Fixme: need decode block for getting timestamp. For now use system time.
                            let current_count_block_timestamp =
                                (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
                                    as TimeStamp;

                            // Get request number
                            match self
                                .get_request_number(
                                    "filter",
                                    PROJECT_FILTER,
                                    DATA_NAME,
                                    current_count_block_timestamp,
                                )
                                .await
                            {
                                Ok(projects_request) => {
                                    info!("projects_request: {:?}", projects_request);
                                    let project_quota = self.projects.clone();
                                    if let Err(e) = self
                                        .chain_adapter
                                        .submit_projects_usage(project_quota, projects_request)
                                        .await
                                    {
                                        info!("Submit projects usage error: {:?}", e)
                                    }
                                }

                                Err(e) => info!("get_request_number error: {}", e),
                            }
                            last_count_block = current_count_block;
                        }
                    }
                }
            }
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        // info!("test_sign_extrinsic...");
        // self.chain_adapter.test_sign_extrinsic();

        let projects = self.projects.clone();
        let chain_adapter = self.chain_adapter.clone();
        let list_project_url = self.list_project_url.clone();

        // Update quota list
        task::spawn(async move {
            loop {
                let res =
                    Self::get_projects_quota_list(projects.clone(), &list_project_url, "staked")
                        .await;
                if res.is_err() {
                    info!("Get projects quota error: {:?}", res);
                }
                sleep(Duration::from_secs(UPDATE_PROJECT_QUOTA_INTERVAL)).await;
            }
        });
        let projects = self.projects.clone();
        // Update quota list
        task::spawn(async move {
            info!("Subscribe_event");
            chain_adapter
                .subscribe_event_update_quota(projects.clone())
                .await;
        });

        // subscribe finalized header
        let subscribe_finalized_heads = self.subscribe_finalized_heads().await?;

        // Get request number over time to submit to chain
        self.loop_get_request_number(subscribe_finalized_heads)
            .await?;

        Ok(())
    }
}
impl StatsBuilder {
    pub async fn with_config_uri(mut self, path: String) -> StatsBuilder {
        self.inner.config_data_uri = path;

        let config_data: Option<ConfigData> = self.get_config_data().await;
        self.inner.config_data = config_data;
        self
    }

    async fn get_config_data(&self) -> Option<ConfigData> {
        // todo!()
        None
    }
    async fn get_prometheus_client(&self, url: &String) -> anyhow::Result<PrometheusClient> {
        let client = PrometheusClient::try_from(url.as_str())
            .map_err(|err| anyhow::Error::msg(format!("{:?}", err)));
        client
    }

    pub async fn with_prometheus_gateway_url(mut self, path: String) -> StatsBuilder {
        self.inner.prometheus_gateway_url = path;

        self.inner.gateway_adapter = Arc::new(DataCollectionAdapter {
            //data: Default::default(),
            client: self
                .get_prometheus_client(&self.inner.prometheus_gateway_url.to_string())
                .await
                .ok(),
        });
        self
    }

    pub async fn with_prometheus_node_url(mut self, path: String) -> StatsBuilder {
        self.inner.prometheus_node_url = path;

        self.inner.node_adapter = Arc::new(DataCollectionAdapter {
            client: self
                .get_prometheus_client(&self.inner.prometheus_node_url.to_string())
                .await
                .ok(),
        });
        self
    }

    pub async fn with_mvp_url(mut self, path: String) -> StatsBuilder {
        self.inner.mvp_url = path.clone();
        // RPSee client for subscribe new block
        let client = WsClientBuilder::default().build(&path).await;
        info!("chain client: {:?}", client);

        // substrate_api_client for send extrinsic and subscribe event
        //let (signer,seed) = Pair::from_phrase(self.inner.signer_phrase.as_str(),None).expect("Wrong signer-phrase");
        // Fixme: find Ferdie Pair from phrase
        let signer = AccountKeyring::Ferdie.pair();

        let ws_client = WsRpcClient::new(&self.inner.mvp_url);
        let chain_adapter = ChainAdapter {
            json_rpc_client: client.ok(),
            ws_rpc_client: Some(ws_client.clone()),
            api: Api::new(ws_client.clone())
                .map(|api| api.set_signer(signer))
                .ok(),
        };
        self.inner.chain_adapter = Arc::new(chain_adapter);
        self
    }
    pub fn with_list_project_url(mut self, path: String) -> StatsBuilder {
        self.inner.list_project_url = path;
        self
    }
    pub fn with_signer_phrase(mut self, signer_phrase: String) -> Self {
        self.inner.signer_phrase = signer_phrase;
        self
    }
    pub fn build(self) -> ComponentStats {
        self.inner
    }
}
