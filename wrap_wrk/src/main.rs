use anyhow::Error;
use bytesize::ByteSize;
use std::process::Command;
use std::str::FromStr;
use std::time::Duration;

impl WrkBenchmark {
    fn build(
        thread: i32,
        connection: i32,
        duration: String,
        rate: i32,
        dapiURL: String,
        token: String,
        host: String,
        script: String,
        wrk_path: String,
    ) -> Self {
        WrkBenchmark {
            thread,
            connection,
            duration,
            rate,
            dapiURL,
            token,
            host,
            script,
            wrk_path,
        }
    }
    fn run(&mut self) -> Result<WrkReport, Error> {
        let output = Command::new(&self.wrk_path)
            .arg(format!("-t{}", self.thread))
            .arg(format!("-c{}", self.connection))
            .arg(format!("-d{}", self.duration))
            .arg(format!("-R{}", self.rate))
            .arg(format!("-s"))
            .arg(format!("{}", self.script))
            .arg(format!("{}", self.dapiURL))
            .arg(format!("--"))
            .arg(format!("{}", self.token))
            .arg(format!("{}", self.host))
            .output()
            .expect("failed to execute process");
        let status = output.status;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("status: {}", status);
        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);

        //assert!(output.status.success());
        Self::get_report(&stdout)
    }

    fn parse_string_duration(time: &String) -> Option<Duration> {
        if time.contains("-nan") {
            return None;
        }
        if time.contains("ms") {
            Some(Duration::from_secs_f32(
                time.strip_suffix("ms").unwrap().parse::<f32>().unwrap() / 1000f32,
            ))
        } else {
            Some(Duration::from_secs_f32(
                time.strip_suffix("s").unwrap().parse::<f32>().unwrap(),
            ))
        }
    }

    fn get_report(stdout: &String) -> Result<WrkReport, Error> {
        let tmp: Vec<String> = stdout
            .split("Latency")
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let tmp = tmp[1].clone();
        let arr: Vec<String> = tmp
            .split_whitespace()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        println!("{:?}", arr);
        let latency = ValueMetric::<Duration> {
            avg: Self::parse_string_duration(&arr[0]),
            stdev: Self::parse_string_duration(&arr[1]),
            max: Self::parse_string_duration(&arr[2]),
            stdev_percent: arr[3].strip_suffix("%").unwrap().parse::<f32>().ok(),
        };
        let success_req_per_sec = ValueMetric::<f32> {
            avg: arr[5].parse::<f32>().ok(),
            stdev: arr[6].parse::<f32>().ok(),
            max: arr[7].parse::<f32>().ok(),
            stdev_percent: arr[8]
                .strip_suffix("%")
                .unwrap()
                .clone()
                .parse::<f32>()
                .ok(),
        };
        println!("latency:{:?}", latency);
        println!("success_req_per_sec:{:?}", success_req_per_sec);
        let total_req = arr[9].parse::<usize>().unwrap();
        let total_duration =
            Self::parse_string_duration(&arr[12].strip_suffix(",").unwrap().to_string()).unwrap();
        let total_read = ByteSize::from_str(&arr[13]).unwrap();

        let mut socket_error = None;
        if tmp.contains("Socket error") {
            socket_error = Some(SocketError {
                connect: arr[18].strip_suffix(",").unwrap().parse::<usize>().unwrap(),
                read: arr[20].strip_suffix(",").unwrap().parse::<usize>().unwrap(),
                write: arr[22].strip_suffix(",").unwrap().parse::<usize>().unwrap(),
                timeout: arr[24].parse::<usize>().unwrap(),
            });
        }

        let tmp: Vec<String> = tmp
            .split("Requests/sec:")
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let tmp = tmp[1].clone();
        let arr: Vec<String> = tmp
            .split_whitespace()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let req_per_sec = arr[0].parse::<f32>().unwrap();
        let tran_per_sec = ByteSize::from_str(&arr[2]).unwrap();
        let report = Ok(WrkReport {
            latency,
            success_req_per_sec,
            total_req,
            total_duration,
            total_read,
            req_per_sec,
            tran_per_sec,
            socket_error,
        });

        report
    }
}

#[derive(Clone, Debug, Default)]
pub struct WrkBenchmark {
    thread: i32,
    connection: i32,
    duration: String,
    rate: i32,
    dapiURL: String,
    token: String,
    host: String,
    script: String,
    wrk_path: String,
}

#[derive(Clone, Debug, Default)]
pub struct WrkReport {
    latency: ValueMetric<Duration>,
    success_req_per_sec: ValueMetric<f32>,
    total_req: usize,
    total_duration: Duration,
    total_read: ByteSize,
    req_per_sec: f32,
    tran_per_sec: ByteSize,
    socket_error: Option<SocketError>,
}

#[derive(Clone, Debug, Default)]
pub struct ValueMetric<T> {
    avg: Option<T>,
    stdev: Option<T>,
    max: Option<T>,
    stdev_percent: Option<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct SocketError {
    connect: usize,
    read: usize,
    write: usize,
    timeout: usize,
}

fn main() {
    let thread = 20;
    let connection = 20;
    let duration = "20s";
    let rate = 10;
    // let dapiURL = "http://34.101.81.225:8545";
    let dapiURL = "http://34.101.82.118";
    let token = "SJs5XDqPiU5MPx3h_C2qrA";
    let host = "a0f7d53f-b5ff-4ab5-8c5e-a239d81bdaa1.node.mbr.massbitroute.dev";
    let script = "../scripts/benchmark/massbit.lua";
    let wrk_path = "../scripts/benchmark/wrk";

    let mut wrk = WrkBenchmark::build(
        thread,
        connection,
        duration.to_string(),
        rate,
        dapiURL.to_string(),
        token.to_string(),
        host.to_string(),
        script.to_string(),
        wrk_path.to_string(),
    );
    let report = wrk.run();

    println!("{:?}", report)

    //assert!(output.status.success());
}
