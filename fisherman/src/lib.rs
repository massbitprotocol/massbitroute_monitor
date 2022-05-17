use lazy_static::lazy_static;
use mbr_check_component::check_module::check_module::Zone;
use serde::Deserialize;
use std::env;
use std::str::FromStr;
pub mod data_check;
pub mod fisherman_service;
pub mod ping_pong;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub response_time_key_name: String,
    pub number_of_samples: u64,
    pub sample_interval_ms: u64,
    pub delay_between_check_loop_ms: u64,
    pub success_percent_threshold: u32,
    pub node_response_time_threshold: u32,
    pub gateway_response_time_threshold: u32,
    pub node_response_failed_number: i32,
    pub gateway_response_failed_number: i32,
    pub reports_history_queue_length_max: usize,
    pub check_task_list_fisherman: Vec<String>,
    pub checking_component_status: String,

    // for submit report
    pub mvp_extrinsic_submit_provider_report: String,
    pub mvp_extrinsic_dapi: String,
    pub mvp_extrinsic_submit_project_usage: String,
    pub mvp_event_project_registered: String,

    // for ping pong check
    pub ping_parallel_requests: usize,
    pub ping_success_ratio_threshold: f32,
    pub ping_sample_number: u64,
    pub ping_request_response: String,
    pub ping_timeout_ms: u64,

    pub check_ping_pong_interval: u64,
    pub check_logic_interval: u64,
    pub check_benchmark_interval: u64,
    pub update_provider_list_interval: u64,
}
const CONFIG_FILE: &str = "config_fisherman.json";
lazy_static! {
    pub static ref FISHERMAN_ENDPOINT: String =
        env::var("FISHERMAN_ENDPOINT").unwrap_or(String::from("0.0.0.0:4040"));
    pub static ref ZONE: Zone = {
        let zone = &env::var("ZONE").unwrap().to_uppercase();
        Zone::from_str(zone.as_str()).unwrap()
    };
    pub static ref CONFIG: Config = get_config();
}

fn get_config() -> Config {
    let json = std::fs::read_to_string(CONFIG_FILE)
        .unwrap_or_else(|err| panic!("Unable to read config file `{}`: {}", CONFIG_FILE, err));
    let config: Config = serde_json::from_str(&*json).unwrap();
    config
}
