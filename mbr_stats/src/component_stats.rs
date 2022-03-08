use anyhow::{anyhow, Error};
use chrono::Duration;
use clap::StructOpt;
use futures::pin_mut;
use futures_util::future::{err, join_all};
use minifier::json::minify;
use reqwest::{Body, Response};
use serde::{forward_to_deserialize_any_helper, Deserialize, Serialize};
use std::collections::HashMap;

use jsonrpsee::core::client::{
    Client as JsonRpcClient, ClientT, Subscription, SubscriptionClientT,
};
use jsonrpsee::types::ParamsSer;
use jsonrpsee::ws_client::WsClientBuilder;
use jsonrpsee::{rpc_params, tracing};
use log::{debug, error, info, log_enabled, Level};
use prometheus_http_query::aggregations::{count_values, sum, topk};
use prometheus_http_query::functions::round;
use prometheus_http_query::{Aggregate, Client as PrometheusClient, InstantVector, Selector};
use serde_json::{to_string, Number, Value};
use std::convert::TryInto;
use std::fmt::Formatter;
use std::os::unix::raw::time_t;
use std::time::{SystemTime, UNIX_EPOCH};

type TimeStamp = i64;

const NUMBER_BLOCK_FOR_COUNTING: isize = 3;
const DATA_NAME: &str = "nginx_vts_filter_requests_total";

#[derive(Debug)]
pub enum ComponentType {
    Node,
    Dapi,
    Gateway,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ConfigData;

#[derive(Debug, Default)]
pub struct ComponentStats<'a> {
    // input file
    pub config_data_uri: &'a str,
    // config data
    pub config_data: Option<ConfigData>,
    // prometheus url
    pub prometheus_gateway_url: &'a str,
    pub prometheus_node_url: &'a str,
    // Massbit verification protocol chain url
    pub mvp_url: &'a str,

    // For collecting data
    pub gateway_adapter: DataCollectionAdapter,
    pub node_adapter: DataCollectionAdapter,
    // For collecting data

    // For submit data
    pub chain_adapter: ChainAdapter,
}

#[derive(Clone, Default)]
pub struct DataCollectionAdapter {
    //data: HashMap<String, QueryData>,
    client: Option<PrometheusClient>,
}

impl std::fmt::Debug for DataCollectionAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataCollectionAdapter<>")
    }
}

#[derive(Default)]
pub struct ChainAdapter {
    data: HashMap<String, SubmitData>,
    client: Option<JsonRpcClient>,
}

impl std::fmt::Debug for ChainAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MvpAdapter<{:?}>", &self.data)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct SubmitData {
    consumer_id: String,
    requests_count: isize,
    from_block_number: isize,
    to_block_number: isize,
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

pub struct StatsBuilder<'a> {
    inner: ComponentStats<'a>,
}

impl<'a> Default for StatsBuilder<'a> {
    fn default() -> Self {
        Self {
            inner: ComponentStats {
                config_data_uri: "",
                config_data: None,
                prometheus_gateway_url: "",
                prometheus_node_url: "",
                mvp_url: "",
                gateway_adapter: Default::default(),
                node_adapter: Default::default(),
                chain_adapter: Default::default(),
            },
        }
    }
}

impl<'a> ComponentStats<'a> {
    pub fn builder() -> StatsBuilder<'a> {
        StatsBuilder::default()
    }

    async fn get_request_number(
        &self,
        filter: &str,
        value: &str,
        data_name: &str,
        time: TimeStamp,
    ) -> anyhow::Result<HashMap<String, usize>> {
        let res: HashMap<String, usize> = HashMap::new();
        let q: InstantVector = Selector::new()
            .metric(data_name)
            .regex_match(filter, value)
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
                        Some((filter.to_string(), value))
                    })
                })
                .collect::<HashMap<String, usize>>())
        } else {
            Err(anyhow::Error::msg("Cannot parse response"))
        };
        //println!("Hashmap res: {:#?}", res);
        res

    }

    async fn get_request_number_in_duration(
        &self,
        start_time: TimeStamp,
        end_time: TimeStamp,
        data_name: &str,
    ) -> anyhow::Result<HashMap<String,usize>> {
        let dapi_id: &str = ".*gw::api_method.*|.*node::api_method.*|.*dapi::api_method.*";
        let start_req_number = self
            .get_request_number("filter", dapi_id, data_name,start_time)
            .await?;
        let end_req_number = self.get_request_number("filter", dapi_id, data_name,end_time).await?;
        let res = start_req_number.iter().filter_map(|(name,start_value)|{
            if let Some(end_value) = end_req_number.get(name) {
                Some((name.to_string(),end_value-start_value))
            }
            else {
                None
            }
        }).collect::<HashMap<String,usize>>();


        Ok(res)
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        // subscribe finalized header
        let client = self
            .chain_adapter
            .client
            .as_ref()
            .ok_or(anyhow::Error::msg("None chain client"))?;
        let mut subscribe_finalized_heads = client
            .subscribe::<Value>(
                "chain_subscribeFinalizedHeads",
                rpc_params![],
                "chain_unsubscribeFinalizedHeads",
            )
            .await;
        println!("subscribe_finalized_heads: {:?}", subscribe_finalized_heads);
        let mut subscribe_finalized_heads = subscribe_finalized_heads?;
        let mut i = 0;
        // wait for new block
        let mut last_count_block: isize = -1;
        let mut last_count_block_timestamp: TimeStamp = -1;

        loop {
            let res = subscribe_finalized_heads.next().await;
            if let Some(Ok(res)) = res {
                println!("received {:?}", res);
                if let Some(block_number) = res.get("number") {
                    let block_number = isize::from_str_radix(
                        block_number.as_str().unwrap().trim_start_matches("0x"),
                        16,
                    )?;
                    println!("block_number {:?}", block_number);
                    if last_count_block == -1 {
                        last_count_block = block_number;
                        // Fixme: need decode block for getting timestamp. For now use system time.
                        last_count_block_timestamp =
                            (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
                                as TimeStamp;
                        continue;
                    } else {
                        if block_number - last_count_block >= NUMBER_BLOCK_FOR_COUNTING {
                            // Set new count block
                            let current_count_block = last_count_block + NUMBER_BLOCK_FOR_COUNTING;
                            // Fixme: need decode block for getting timestamp. For now use system time.
                            let current_count_block_timestamp =
                                (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
                                    as TimeStamp;
                            // Get number of request in between 2 time
                            // Collect data
                            if let Ok(res) = self
                                .get_request_number_in_duration(
                                    last_count_block_timestamp,
                                    current_count_block_timestamp,
                                    DATA_NAME
                                )
                                .await
                            {
                                println!(
                                    "Number of requests from {} to {}: {:#?}",
                                    last_count_block_timestamp, current_count_block_timestamp, res
                                );
                            }

                            last_count_block = current_count_block;
                            last_count_block_timestamp = current_count_block_timestamp;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
impl<'a> StatsBuilder<'a> {
    pub async fn with_config_uri(mut self, path: &'a str) -> StatsBuilder<'a> {
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
        use std::convert::TryFrom;
        let client = PrometheusClient::try_from(url.as_str())
            .map_err(|err| anyhow::Error::msg(format!("{:?}", err)));

        client
    }

    pub async fn with_prometheus_gateway_url(mut self, path: &'a str) -> StatsBuilder<'a> {
        self.inner.prometheus_gateway_url = path;

        self.inner.gateway_adapter = DataCollectionAdapter {
            //data: Default::default(),
            client: self
                .get_prometheus_client(&self.inner.prometheus_gateway_url.to_string())
                .await
                .ok(),
        };
        self
    }

    pub async fn with_prometheus_node_url(mut self, path: &'a str) -> StatsBuilder<'a> {
        self.inner.prometheus_node_url = path;

        self.inner.node_adapter = DataCollectionAdapter {
            client: self
                .get_prometheus_client(&self.inner.prometheus_node_url.to_string())
                .await
                .ok(),
        };
        self
    }

    pub async fn with_mvp_url(mut self, path: &'a str) -> StatsBuilder<'a> {
        self.inner.mvp_url = path;
        let client = WsClientBuilder::default().build(&path).await;
        println!("chain client: {:?}", client);
        self.inner.chain_adapter.client = client.ok();
        self
    }

    pub fn build(self) -> ComponentStats<'a> {
        self.inner
    }
}
