use anyhow::Error;
use bytesize::ByteSize;
use log::info;
use logger::core::init_logger;
use std::env::current_dir;
use std::process::Command;
use std::str::FromStr;
use std::time::Duration;
use wrap_wrk::WrkBenchmark;

fn main() {
    let _res = init_logger(&String::from("CheckComponent"));
    //println!("Log output: {}", res); // Print log output type

    let thread = 20;
    let connection = 20;
    let duration = "20s";
    let rate = 10;
    // let dapi_url = "http://34.101.81.225:8545";
    let dapi_url = "https://34.142.136.135";
    let token = "LV_gHNH0MDepQ83IWRX16A";
    let host = "5bd9f624-338e-4f41-b89c-65e9bb9bf3c2.node.mbr.massbitroute.dev";
    let script = "../scripts/benchmark/massbit.lua";
    let wrk_path = "../scripts/benchmark/wrk";
    let wrk_dir = "./";
    let latency_threshold_ms = 500f32;
    let mut wrk = WrkBenchmark::build(
        thread,
        connection,
        duration.to_string(),
        rate,
        dapi_url.to_string(),
        token.to_string(),
        host.to_string(),
        script.to_string(),
        wrk_path.to_string(),
        wrk_dir.to_string(),
        latency_threshold_ms,
    );
    let report = wrk.run();

    info!("report: {:?}", report)

    //assert!(output.status.success());
}
