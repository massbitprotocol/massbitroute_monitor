pub mod check_module;
pub mod server_builder;
pub mod server_config;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;

pub const CONFIG_FILE: &str = "config_check_component.json";

#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    pub check_interval_ms: u64,
    pub check_task_list_node: Vec<String>,
    pub check_task_list_all: Vec<String>,
    pub check_task_list_gateway: Vec<String>,
    pub max_json_body_size: u64,
    pub response_time_key: String,
    pub max_length_report_detail: usize,
    pub benchmark_thread: i32,
    pub benchmark_connection: i32,
    pub benchmark_duration: String,
    pub benchmark_rate: i32,
    pub benchmark_script: String,
    pub benchmark_wrk_path: String,
    pub success_percent_threshold: u32,
    pub node_response_time_threshold_ms: f32,
    pub gateway_response_time_threshold_ms: f32,
    pub accepted_low_latency_percent: f32,
}

lazy_static! {
    pub static ref CHECK_COMPONENT_ENDPOINT: String =
        env::var("CHECK_COMPONENT_ENDPOINT").unwrap_or(String::from("0.0.0.0:3030"));
    pub static ref BASE_ENDPOINT_JSON: String = env::var("BASE_ENDPOINT_JSON").unwrap();
    pub static ref BENCHMARK_WRK_PATH: String = env::var("BENCHMARK_WRK_PATH").unwrap_or("./".to_string());
    // pub static ref CHECK_INTERVAL_MS: u64 = 3000;
    // pub static ref CHECK_TASK_LIST_NODE: Vec<String> = vec![
    //     "checking_chain_type".to_string(),
    //     "checking_chain_sync".to_string(),
    // ];
    // pub static ref CHECK_TASK_LIST_ALL: Vec<String> = vec![
    //     "checking_chain_type".to_string(),
    //     "checking_chain_sync".to_string(),
    // ];
    // pub static ref CHECK_TASK_LIST_GATEWAY: Vec<String> = vec!["checking_chain_type".to_string(),];
    pub(crate) static ref CONFIG: Config = get_config();
}

fn get_config() -> Config {
    let json = std::fs::read_to_string(CONFIG_FILE)
        .unwrap_or_else(|err| panic!("Unable to read config file `{}`: {}", CONFIG_FILE, err));
    serde_json::from_str::<Config>(&*json).unwrap()
}
