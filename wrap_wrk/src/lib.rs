use anyhow::Error;
use bytesize::ByteSize;
use log::{debug, info};
use regex::Regex;
use std::process::Command;
use std::str::FromStr;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct DetailedPercentileSpectrum {
    latency: f32,
    percent: f32,
    count: u64,
}

impl WrkBenchmark {
    pub fn build(
        thread: i32,
        connection: i32,
        duration: String,
        rate: i32,
        dapi_url: String,
        token: String,
        host: String,
        script: String,
        wrk_path: String,
        current_dir: String,
        latency_threshold_ms: f32,
    ) -> Self {
        WrkBenchmark {
            thread,
            connection,
            duration,
            rate,
            dapi_url,
            token,
            host,
            script,
            wrk_path,
            current_dir,
            latency_threshold_ms: latency_threshold_ms,
        }
    }
    pub fn run(&mut self) -> Result<WrkReport, Error> {
        info!("current_dir: {}", self.current_dir);
        info!("wrk_path: {}", self.wrk_path);
        info!("script: {}", self.script);
        let output = Command::new(&self.wrk_path)
            .current_dir(&self.current_dir)
            .arg(format!("--latency"))
            .arg(format!("-t{}", self.thread))
            .arg(format!("-c{}", self.connection))
            .arg(format!("-d{}", self.duration))
            .arg(format!("-R{}", self.rate))
            .arg(format!("-s"))
            .arg(format!("{}", self.script))
            .arg(format!("{}", self.dapi_url))
            .arg(format!("--"))
            .arg(format!("{}", self.token))
            .arg(format!("{}", self.host))
            .output()
            .expect("failed to execute process");
        let status = output.status;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);
        // info!("status: {}", status);
        // info!("stdout: {}", stdout);
        // info!("stderr: {}", stderr);

        //assert!(output.status.success());
        self.get_report(&stdout, 500f32)
    }

    fn parse_string_duration(time: &String) -> Option<Duration> {
        if time.contains("-nan") || time.contains("-nanus") {
            return None;
        }
        if time.contains("ms") {
            Some(Duration::from_secs_f32(
                time.strip_suffix("ms").unwrap().parse::<f32>().unwrap() / 1000f32,
            ))
        } else if time.contains("us") {
            Some(Duration::from_secs_f32(
                time.strip_suffix("us").unwrap().parse::<f32>().unwrap() / 1000_000f32,
            ))
        } else {
            Some(Duration::from_secs_f32(
                time.strip_suffix("s").unwrap().parse::<f32>().unwrap(),
            ))
        }
    }

    fn get_latency_table(&self, text: &String) -> Result<Vec<DetailedPercentileSpectrum>, Error> {
        let re = Regex::new(
            r"Value   Percentile   TotalCount 1/\(1-Percentile\)\s+(?P<table>[\d.\sinf]+)#",
        )?;
        let caps = re.captures(text).unwrap();
        let table = caps.name("table").unwrap().as_str();
        //info!("table:{}", table);

        let sorted_table: Vec<DetailedPercentileSpectrum> = table
            .split("\n")
            .filter_map(|line| {
                //info!("s:{}", line);
                let arr = line
                    .split_whitespace()
                    .map(|value| value.to_string())
                    .collect::<Vec<String>>();
                //info!("arr:{:?}", arr);
                if arr.len() == 4 {
                    Some(DetailedPercentileSpectrum {
                        latency: arr[0].parse::<f32>().unwrap_or(f32::MAX),
                        percent: arr[1].parse::<f32>().unwrap_or(f32::MAX),
                        count: arr[2].parse::<u64>().unwrap_or(u64::MAX),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(sorted_table)
    }

    fn get_percent_latency(&self, sorted_table: Vec<DetailedPercentileSpectrum>) -> f32 {
        let mut percent = 0f32;
        for line in sorted_table {
            if self.latency_threshold_ms > line.latency {
                percent = line.percent
            } else {
                break;
            }
        }
        percent
    }

    fn get_report(&self, stdout: &String, percent_pass_latency: f32) -> Result<WrkReport, Error> {
        //info!("{}", stdout);
        // Get percent_low_latency
        let sorted_table = self.get_latency_table(stdout)?;
        //info!("vec table:{:?}", sorted_table);
        let percent_low_latency = self.get_percent_latency(sorted_table);
        debug!("percent_low_latency:{:?}", percent_low_latency);
        //Get Non-2xx or 3xx responses
        let re = Regex::new(r"Non-2xx or 3xx responses: (?P<non_2xx_3xx_req>\d+)")?;
        let caps = re.captures(stdout);
        let non_2xx_3xx_req = caps
            .and_then(|caps| {
                Some(
                    caps.name("non_2xx_3xx_req")
                        .unwrap()
                        .as_str()
                        .parse::<usize>()
                        .unwrap_or(0),
                )
            })
            .unwrap_or(0);

        // Get total_req, total_duration, total_read:
        let re = Regex::new(
            r"(?P<total_req>\d+) requests in (?P<total_duration>\d+\.\d+\w+), (?P<total_read>\d+\.\d+\w+) read",
        )?;
        let caps = re.captures(stdout).unwrap();
        let total_req = caps
            .name("total_req")
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap();
        let total_duration = caps.name("total_duration").unwrap().as_str().to_string();
        let total_read = caps.name("total_read").unwrap().as_str();
        let total_duration = Self::parse_string_duration(&total_duration).unwrap();
        let total_read = ByteSize::from_str(&total_read).unwrap();

        // Get Requests/sec, Transfer/sec
        let re = Regex::new(
            r"Requests/sec:\s+(?P<req_per_sec>\d+\.\d+)\s+Transfer/sec:\s+(?P<tran_per_sec>\d+\.\d+\w+?)\s+",
        )?;
        let caps = re.captures(stdout).unwrap();
        let req_per_sec = caps
            .name("req_per_sec")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .unwrap();
        let tran_per_sec = caps.name("tran_per_sec").unwrap().as_str();
        debug!("tran_per_sec:{}", tran_per_sec);
        let tran_per_sec = ByteSize::from_str(&tran_per_sec).unwrap();

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

        //info!("arr: {:?}", arr);
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
        debug!("latency:{:?}", latency);
        debug!("success_req_per_sec:{:?}", success_req_per_sec);

        let mut socket_error = None;
        if tmp.contains("Socket error") {
            socket_error = Some(SocketError {
                connect: arr[18].strip_suffix(",").unwrap().parse::<usize>().unwrap(),
                read: arr[20].strip_suffix(",").unwrap().parse::<usize>().unwrap(),
                write: arr[22].strip_suffix(",").unwrap().parse::<usize>().unwrap(),
                timeout: arr[24].parse::<usize>().unwrap(),
            });
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let report = Ok(WrkReport {
            latency,
            success_req_per_sec,
            total_req,
            total_duration,
            total_read,
            req_per_sec,
            tran_per_sec,
            socket_error,
            non_2xx_3xx_req,
            percent_low_latency,
            timestamp,
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
    dapi_url: String,
    token: String,
    host: String,
    script: String,
    wrk_path: String,
    current_dir: String,
    latency_threshold_ms: f32,
}

#[derive(Clone, Debug, Default)]
pub struct WrkReport {
    pub latency: ValueMetric<Duration>,
    pub success_req_per_sec: ValueMetric<f32>,
    pub total_req: usize,
    pub total_duration: Duration,
    pub total_read: ByteSize,
    pub req_per_sec: f32,
    pub tran_per_sec: ByteSize,
    pub socket_error: Option<SocketError>,
    pub non_2xx_3xx_req: usize,
    pub percent_low_latency: f32,
    pub timestamp: u128,
}

impl WrkReport {
    pub fn get_success_percent(&self) -> Option<u32> {
        if self.total_req > 0 {
            Some(((self.total_req - self.non_2xx_3xx_req) * 100 / self.total_req) as u32)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ValueMetric<T> {
    pub avg: Option<T>,
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
