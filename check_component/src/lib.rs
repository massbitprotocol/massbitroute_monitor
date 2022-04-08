pub mod check_module;
pub mod config;
pub mod server_builder;
use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref CHECK_COMPONENT_ENDPOINT: String =
        env::var("CHECK_COMPONENT_ENDPOINT").unwrap_or(String::from("0.0.0.0:3030"));
    pub static ref BASE_ENDPOINT_JSON: String = env::var("BASE_ENDPOINT_JSON").unwrap();
    pub static ref CHECK_INTERVAL_MS: u64 = 3000;
    pub static ref CHECK_TASK_LIST: Vec<String> = vec![
        "checking_chain_type".to_string(),
        "checking_chain_sync".to_string(),
    ];
}
