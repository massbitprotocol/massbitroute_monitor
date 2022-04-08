use lazy_static::lazy_static;
use std::env;
pub mod fisherman_service;

lazy_static! {
    pub static ref FISHERMAN_ENDPOINT: String =
        env::var("FISHERMAN_ENDPOINT").unwrap_or(String::from("0.0.0.0:4040"));
}
//Fixme: use better solution to get response time
pub const RESPONSE_TIME_KEY_NAME: &str = "checkCall_response_time_ms";
pub const NUMBER_OF_SAMPLES: u64 = 5;
pub const SAMPLE_INTERVAL_MS: u64 = 200;
pub const DELAY_BETWEEN_CHECK_LOOP_MS: u64 = 1000;
// Good health response
const SUCCESS_PERCENT_THRESHOLD: u32 = 50;

const NODE_RESPONSE_TIME_THRESHOLD: u32 = 3000;
const GATEWAY_RESPONSE_TIME_THRESHOLD: u32 = 4000;

const NODE_RESPONSE_FAILED_NUMBER: i32 = 1;
const GATEWAY_RESPONSE_FAILED_NUMBER: i32 = 2;

const MVP_EXTRINSIC_SUBMIT_PROVIDER_REPORT: &str = "submit_provider_report";
const REPORTS_HISTORY_QUEUE_LENGTH_MAX: usize = 3;
