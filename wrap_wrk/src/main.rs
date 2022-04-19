use anyhow::Error;
use bytesize::ByteSize;
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
    let dapi_url = "http://34.101.82.118";
    let token = "SJs5XDqPiU5MPx3h_C2qrA";
    let host = "a0f7d53f-b5ff-4ab5-8c5e-a239d81bdaa1.node.mbr.massbitroute.dev";
    let script = "../scripts/benchmark/massbit.lua";
    let wrk_path = "../scripts/benchmark/wrk";
    let wrk_dir = "./";
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
    );
    let report = wrk.run();

    println!("{:?}", report)

    //assert!(output.status.success());
}
