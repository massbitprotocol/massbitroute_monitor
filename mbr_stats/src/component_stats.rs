use std::collections::HashMap;
use anyhow::{anyhow, Error};
use chrono::Duration;
use clap::StructOpt;
use futures::pin_mut;
use futures_util::future::{err, join_all};
use minifier::json::minify;
use reqwest::{Body, Response};
use serde::{forward_to_deserialize_any_helper, Deserialize, Serialize};

use log::{debug, error, info, log_enabled, Level};
use serde_json::{to_string, Number, Value};
use prometheus_http_query::{Aggregate, Client as PrometheusClient, InstantVector, Selector};
use prometheus_http_query::aggregations::{sum, topk};
use std::convert::TryInto;
use std::fmt::Formatter;
use jsonrpsee::core::client::Client as JsonRpcClient;



#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ConfigData ;

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
    pub data_collection_adapter: DataCollectionAdapter,
    // For submit data
    pub mvp_adapter: MvpAdapter,

}

#[derive(Clone, Default)]
pub struct DataCollectionAdapter{
    data: HashMap<String,QueryData>,
    client: Option<PrometheusClient>,
}

impl std::fmt::Debug for DataCollectionAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataCollectionAdapter<{:?}>",&self.data)
    }
}

#[derive(Default)]
pub struct MvpAdapter{
    data: HashMap<String,SubmitData>,
    client: Option<JsonRpcClient>,
}

impl std::fmt::Debug for MvpAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MvpAdapter<{:?}>",&self.data)
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
    value:  isize,
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
                prometheus_gateway_url: "",
                mvp_url: "",
                data_collection_adapter: Default::default(),
                mvp_adapter: Default::default()
            },
        }
    }
}



impl<'a> ComponentStats<'a> {
    pub fn builder() -> StatsBuilder<'a> {
        StatsBuilder::default()
    }
    pub async fn run(&self) -> anyhow::Result<()>{

        // use std::convert::TryFrom;
        // let client = Client::try_from("34.88.224.156:443").unwrap();
        // let vector: InstantVector = Selector::new()
        //     .metric("nginx_vts_filter_requests_total")
        //     .try_into()?;
        // let q = topk(vector, Some(Aggregate::By(&["code"])), 5);
        //
        // let response = client.query(q, None, None).await?;
        //
        // assert!(response.as_instant().is_some());
        //
        // if let Some(result) = response.as_instant() {
        //     let first = result.get(0).unwrap();
        //     println!("Received a total of {} HTTP requests", first.sample().value());
        // };

        // Get data from prometheus


        Ok(())
    }
}
impl<'a> StatsBuilder<'a> {
    pub async fn with_config_uri(mut self, path: &'a str) -> StatsBuilder<'a> {
        self.inner.config_data_uri = path;

        let config_data: Option<ConfigData> = self
            .get_config_data()
            .await;
        self.inner.config_data = config_data;
        self
    }


    async fn get_config_data(&self) -> Option<ConfigData>{
        // todo!()
        None
    }
    async fn get_prometheus_client(&self,url: &String) -> anyhow::Result<PrometheusClient>{
        // todo!()
        Ok(PrometheusClient::default())
    }

    pub async fn with_prometheus_gateway_url(mut self, path: &'a str) -> StatsBuilder<'a> {
        self.inner.prometheus_gateway_url = path;

        self.inner.data_collection_adapter = DataCollectionAdapter{
            data: Default::default(),
            client: Some(self.get_prometheus_client(&self.inner.prometheus_gateway_url.to_string())?)
        } ;
        self
    }

    pub async fn with_prometheus_node_url(mut self, path: &'a str) -> StatsBuilder<'a> {
        self.inner.prometheus_node_url = path;

        self.inner.data_collection_adapter = DataCollectionAdapter{
            data: Default::default(),
            client: Some(self.get_prometheus_client(&self.inner.prometheus_node_url.to_string())?)
        } ;
        self
    }


    pub fn build(self) -> ComponentStats<'a> {
        self.inner
    }
}
