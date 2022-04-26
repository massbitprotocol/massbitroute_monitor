use anyhow::Error;
use bytesize::ByteSize;
use log::info;
use std::env::current_dir;
use std::process::Command;
use std::str::FromStr;
use std::time::Duration;
use wrap_wrk::WrkBenchmark;

fn main() {
    let thread = 20;
    let connection = 20;
    let duration = "20s";
    let rate = 10;
    // let dapi_url = "http://34.101.81.225:8545";
    let dapi_url = "https://34.118.57.45";
    let token = "Yqwfgvme6d9mt6rRElHncA";
    let host = "d3d6fb54-dcb8-482c-8125-1cb28748b2dd.gw.mbr.massbitroute.dev";
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
