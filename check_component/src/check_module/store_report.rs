use crate::check_module::check_module::{CheckMkReport, ComponentInfo, ComponentType};
use crate::CONFIG;
use anyhow::Error;
use log::{debug, info};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use wrap_wrk::{WrkBenchmark, WrkReport};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct StoreReport {
    pub reporter: String,
    pub reporter_role: ReporterRole,
    #[serde(skip_deserializing)]
    pub domain: String,
    #[serde(skip_deserializing)]
    pub authorization: String,
    #[serde(skip_deserializing)]
    pub provider_id: String,
    pub average_latency: f32,
    pub total_req: usize,
    pub total_duration: f32,
    pub total_read_byte: u64,
    pub non_2xx_3xx_req: usize,
    pub percent_low_latency: f32,
    pub is_data_correct: bool,
    pub provider_type: ComponentType,
    pub report_time: u128,
    pub status_detail: String,
    pub report_type: ReportType,

    pub request_rate: f32,
    pub transfer_rate: f32,
    pub histogram_90: f32,
    pub histogram_95: f32,
    pub histogram_99: f32,
    pub stdev_latency: f32,
    pub max_latency: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReporterRole {
    Fisherman,
    Verification,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReportType {
    ReportProvider,
    Benchmark,
}

impl Default for ReportType {
    fn default() -> Self {
        ReportType::Benchmark
    }
}

pub enum SendPurpose {
    Verify,
    Store,
}

impl Default for ReporterRole {
    fn default() -> Self {
        ReporterRole::Fisherman
    }
}

impl StoreReport {
    pub fn build(
        reporter: &String,
        reporter_role: ReporterRole,
        authorization: &String,
        domain: &String,
    ) -> StoreReport {
        StoreReport {
            reporter: reporter.clone(),
            reporter_role,
            authorization: authorization.clone(),
            domain: domain.clone(),
            ..Default::default()
        }
    }

    pub fn set_report_type(&mut self, component: &ComponentInfo, report_type: ReportType) {
        self.report_type = report_type;
        self.provider_id = component.id.clone();
        self.provider_type = component.component_type.clone();
    }

    pub fn set_report_data(
        &mut self,
        wrk_report: &WrkReport,
        check_mk_report: &CheckMkReport,
        component: &ComponentInfo,
        report_type: ReportType,
    ) {
        self.is_data_correct = check_mk_report.is_component_status_ok();
        self.non_2xx_3xx_req = wrk_report.non_2xx_3xx_req;
        self.average_latency = wrk_report.latency.avg.unwrap_or_default().as_millis() as f32;
        self.total_req = wrk_report.total_req;
        self.percent_low_latency = wrk_report.percent_low_latency;
        self.total_duration = wrk_report.total_duration.as_millis() as f32;
        self.total_read_byte = wrk_report.total_read.as_u64();
        if wrk_report.timestamp == 0 {}
        self.report_time = if wrk_report.timestamp == 0 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        } else {
            wrk_report.timestamp
        };
        self.provider_id = component.id.clone();
        self.provider_type = component.component_type.clone();
        self.status_detail = check_mk_report.status_detail.clone();
        self.report_type = report_type;

        self.request_rate = wrk_report.req_per_sec;
        self.transfer_rate = wrk_report.tran_per_sec.as_u64() as f32;
        self.histogram_90 = wrk_report.histogram_90;
        self.histogram_95 = wrk_report.histogram_95;
        self.histogram_99 = wrk_report.histogram_99;
        self.stdev_latency = wrk_report.latency.stdev.unwrap_or_default().as_millis() as f32;
        self.max_latency = wrk_report.latency.max.unwrap_or_default().as_millis() as f32;
    }

    fn create_body(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self)?)
    }

    fn get_url(&self, send_purpose: SendPurpose) -> String {
        match send_purpose {
            SendPurpose::Verify => {
                format!(
                    "https://portal.{}/mbr/verify/{}",
                    self.domain, self.provider_id
                )
            }
            SendPurpose::Store => {
                format!(
                    "https://portal.{}/mbr/benchmark/{}",
                    self.domain, self.provider_id
                )
            }
        }
    }

    pub async fn send_data(&self, send_purpose: SendPurpose) -> Result<Response, Error> {
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        // create body
        let body = self.create_body()?;
        info!("body: {:?}", body);
        // get url
        let url = self.get_url(send_purpose);

        let request_builder = client
            .post(url)
            .header("content-type", "application/json")
            .header("Authorization", &self.authorization)
            .body(body);
        debug!("request_builder: {:?}", request_builder);
        let response = request_builder.send().await?;
        Ok(response)
    }
}
