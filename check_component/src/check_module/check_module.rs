use futures::pin_mut;
use futures_util::future::join_all;
use minifier::json::minify;

use serde::{Deserialize, Serialize};

use handlebars::Handlebars;
use log::{debug, info, warn};
use serde_json::Value;

use std::collections::HashMap;

use std::fs::File;
use std::io::{BufRead, BufReader};

use anyhow::Error;
use reqwest::RequestBuilder;
use std::time::Instant;
use std::{thread, usize};

use crate::check_module::check_module::CheckMkStatus::{Unknown, Warning};
use crate::check_module::check_module::ComponentType::Gateway;
use crate::check_module::store_report::ReportType::ReportProvider;
use crate::check_module::store_report::{ReportType, ReporterRole, SendPurpose, StoreReport};
use crate::{BASE_ENDPOINT_JSON, BENCHMARK_WRK_PATH, CONFIG, LOCAL_IP, PORTAL_AUTHORIZATION};
use std::str::FromStr;
use strum_macros::EnumString;
use warp::{Rejection, Reply};
pub use wrap_wrk::{WrkBenchmark, WrkReport};

type BlockChainType = String;
type UrlType = String;
type TaskType = String;
type StepResult = HashMap<String, String>;
type ComponentId = String;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Hash, Eq)]
pub enum Zone {
    // Asia
    AS,
    // Europe
    EU,
    // North America
    NA,
    // South america
    SA,
    // Africa
    AF,
    // Oceania
    OC,
    // Global
    GB,
}

impl FromStr for Zone {
    type Err = ();

    fn from_str(input: &str) -> Result<Zone, Self::Err> {
        match input {
            "AS" => Ok(Zone::AS),
            "EU" => Ok(Zone::EU),
            "NA" => Ok(Zone::NA),
            "SA" => Ok(Zone::SA),
            "AF" => Ok(Zone::AF),
            "OC" => Ok(Zone::OC),
            "GB" => Ok(Zone::GB),
            _ => Err(()),
        }
    }
}

impl Default for Zone {
    fn default() -> Self {
        Zone::GB
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ActionCall {
    action_type: String,
    is_base_node: bool,
    request_type: String,
    header: HashMap<String, String>,
    body: String,
    time_out: usize,
    return_fields: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ActionCompare {
    action_type: String,
    operator_items: OperatorCompare,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct OperatorCompare {
    operator_type: String,
    params: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, Hash, PartialEq, Eq)]
pub struct ComponentInfo {
    pub blockchain: BlockChainType,
    pub network: String,
    pub id: ComponentId,
    #[serde(rename = "userId", default)]
    pub user_id: String,
    pub ip: String,
    #[serde(default)]
    pub zone: Zone,
    #[serde(rename = "countryCode", default)]
    pub country_code: String,
    #[serde(rename = "appKey", default)]
    pub token: String,
    #[serde(rename = "componentType", default)]
    pub component_type: ComponentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct UserInfo {
    pub name: String,
    pub id: String,
    pub email: String,
    pub verified: bool,
}

impl ComponentInfo {
    fn get_url(&self) -> UrlType {
        format!("https://{}", self.ip)
    }

    pub fn get_host_header(&self, domain: &String) -> String {
        match self.component_type {
            ComponentType::Node => {
                format!("{}.node.mbr.{}", self.id, domain)
            }
            ComponentType::Gateway => {
                format!("{}.gw.mbr.{}", self.id, domain)
            }
            ComponentType::DApi => String::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckComponent {
    // input
    pub list_node_id_file: String,
    pub list_gateway_id_file: String,
    pub list_dapi_id_file: String,
    pub list_user_file: String,
    pub check_flow_file: String,
    pub base_endpoint_file: String,
    // MBR Domain
    pub domain: String,
    // The output file
    pub output_file: String,
    // inner data
    pub list_nodes: Vec<ComponentInfo>,
    pub list_gateways: Vec<ComponentInfo>,
    pub list_dapis: Vec<ComponentInfo>,
    pub list_users: Vec<UserInfo>,
    pub base_nodes: HashMap<BlockChainType, Vec<EndpointInfo>>,
    pub check_flows: CheckFlows,
    pub is_loop_check: bool,
    pub is_write_to_file: bool,
}

type CheckFlows = HashMap<TaskType, Vec<CheckFlow>>;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct EndpointInfo {
    url: UrlType,
    #[serde(default, rename = "X-Api-Key")]
    x_api_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckFlow {
    #[serde(default)]
    blockchain: BlockChainType,
    #[serde(default)]
    component: String,
    #[serde(default)]
    check_steps: Vec<CheckStep>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckStep {
    #[serde(default)]
    action: Value,
    #[serde(default)]
    return_name: String,
    #[serde(default)]
    failed_case: FailedCase,
}

// struct GatewayInfo;
// struct DApiInfo;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash, Eq)]
pub enum ComponentType {
    Node,
    Gateway,
    DApi,
}
impl ToString for ComponentType {
    fn to_string(&self) -> String {
        match self {
            ComponentType::Node => "node".to_string(),
            ComponentType::Gateway => "gateway".to_string(),
            ComponentType::DApi => "dapi".to_string(),
        }
    }
}

impl std::default::Default for ComponentType {
    fn default() -> Self {
        ComponentType::Node
    }
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub enum ActionConclude {
//     Passed,
//     Failed,
//     Unknown,
// }
// impl Default for ActionConclude {
//     fn default() -> Self {
//         ActionConclude::Unknown
//     }
// }

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct FailedCase {
    #[serde(default)]
    critical: bool,
    #[serde(default)]
    message: String,
    #[serde(default)]
    conclude: CheckMkStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckMkReport {
    pub status: u8,
    pub service_name: String,
    pub metric: CheckMkMetric,
    pub status_detail: String,
    pub success: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckMkMetric {
    pub metric: HashMap<String, Value>,
}

impl ToString for CheckMkMetric {
    fn to_string(&self) -> String {
        if self.metric.is_empty() {
            format!("-")
        } else {
            self.metric
                .iter()
                .map(|(key, val)| format!("{}={}", key, val))
                .collect::<Vec<String>>()
                .join("|")
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ActionResponse {
    success: bool,
    conclude: CheckMkStatus,
    return_name: String,
    result: HashMap<String, String>,
    message: String,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum CheckMkStatus {
    Ok = 0,
    Warning = 1,
    Critical = 2,
    Unknown = 3,
}

impl Default for CheckMkStatus {
    fn default() -> Self {
        CheckMkStatus::Unknown
    }
}

impl ToString for CheckMkReport {
    fn to_string(&self) -> String {
        format!(
            r#"{status} {service_name} {metric} {status_detail}"#,
            status = &self.status,
            service_name = &self.service_name,
            metric = self.metric.to_string(),
            status_detail = self.status_detail
        )
    }
}

impl CheckMkReport {
    fn from_wrk_report(
        wrk_report: WrkReport,
        success_percent_threshold: u32,
        response_time_threshold_ms: f32,
        accepted_percent_low_latency: f32,
    ) -> Self {
        let mut status = CheckMkStatus::Ok as u8;
        let mut message = String::new();
        let latency = wrk_report
            .latency
            .avg
            .and_then(|avg| Some(avg.as_millis()))
            .unwrap_or(u128::MAX);
        let success_percent = wrk_report.get_success_percent().unwrap();
        message.push_str(
            format!(
                "Percent of requests that latency lower than {}ms: {}%, average latency: {}ms, request success percent: {}%.",
                response_time_threshold_ms, wrk_report.percent_low_latency*100f32, latency, success_percent
            )
            .as_str(),
        );

        if accepted_percent_low_latency > wrk_report.percent_low_latency {
            status = CheckMkStatus::Critical as u8;
            message.push_str(
                format!(
                    "False because percent low latency request is too low: {}% (required percent {}%). ",
                    wrk_report.percent_low_latency*100f32, accepted_percent_low_latency*100f32
                )
                .as_str(),
            );
        }

        if success_percent < success_percent_threshold {
            status = CheckMkStatus::Critical as u8;
            message.push_str(
                format!(
                    "False because of request success percent is too low: {}% (required percent {}%). ",
                    success_percent, success_percent_threshold
                )
                .as_str(),
            );
        }

        CheckMkReport {
            status,
            service_name: "benchmark".to_string(),
            metric: Default::default(),
            status_detail: message,
            success: true,
        }
    }
}

impl CheckMkReport {
    fn new_failed_report(msg: String) -> Self {
        let mut resp = CheckMkReport::default();
        resp.success = false;
        resp.status_detail = msg.to_string();
        resp
    }
    pub fn is_component_status_ok(&self) -> bool {
        //Fixme: currently count unknown status as success call. It should separate into 2 cases.
        ((self.success == true) && (self.status == 0)) || (self.status == 4)
    }
    pub fn combine_report(logic_check: &CheckMkReport, benchmark_check: &CheckMkReport) -> Self {
        let mut status = CheckMkStatus::Ok;
        if logic_check.status == CheckMkStatus::Critical as u8
            || benchmark_check.status == CheckMkStatus::Critical as u8
        {
            status = CheckMkStatus::Critical
        } else if logic_check.status == CheckMkStatus::Unknown as u8
            || logic_check.status == CheckMkStatus::Unknown as u8
        {
            status = CheckMkStatus::Unknown
        } else if logic_check.status == CheckMkStatus::Warning as u8
            || logic_check.status == CheckMkStatus::Warning as u8
        {
            status = CheckMkStatus::Warning
        }

        // assert_eq!(logic_check.service_name, benchmark_check.service_name);
        let service_name = logic_check.service_name.clone();

        let mut metric = logic_check.metric.clone();
        metric.metric.extend(benchmark_check.metric.metric.clone());

        let mut status_detail = format!(
            "Logic check:{} Benchmark check:{}",
            logic_check.status_detail, benchmark_check.status_detail
        );
        let success = logic_check.success && benchmark_check.success;
        CheckMkReport {
            status: status as u8,
            service_name,
            metric,
            status_detail,
            success,
        }
    }
}

impl CheckComponent {
    pub fn builder() -> GeneratorBuilder {
        GeneratorBuilder::default()
    }

    fn get_user(&self, user_id: &String) -> Option<&UserInfo> {
        self.list_users.iter().find(|user| &user.id == user_id)
    }

    pub async fn reload_components_list(
        &mut self,
        filter_status: Option<&String>,
        filter_zone: &Zone,
    ) -> Result<(), anyhow::Error> {
        // Get nodes
        let url = &self.list_node_id_file;
        debug!("list_node_id url:{}", url);
        let res_data = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .get(url)
            .header("Authorization", PORTAL_AUTHORIZATION.as_str())
            .send()
            .await?
            .text()
            .await?;
        debug!("res_data Node: {:?}", res_data);
        let mut components: Vec<ComponentInfo> = serde_json::from_str(res_data.as_str())?;
        debug!("components Node: {:?}", components);
        for component in components.iter_mut() {
            component.component_type = ComponentType::Node;
        }
        self.list_nodes = components;

        //Get gateway
        let url = &self.list_gateway_id_file;
        debug!("list_gateway_id url:{}", url);
        let res_data = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .get(url)
            .header("Authorization", PORTAL_AUTHORIZATION.as_str())
            .send()
            .await?
            .text()
            .await?;
        debug!("res_data Gateway: {:?}", res_data);
        let mut components: Vec<ComponentInfo> = serde_json::from_str(res_data.as_str()).unwrap();
        debug!("components Gateway: {:?}", components);
        for component in components.iter_mut() {
            component.component_type = ComponentType::Gateway;
        }
        self.list_gateways = components;

        //Filter components
        if let Some(status) = filter_status {
            self.list_nodes
                .retain(|component| &component.status == status);
            self.list_gateways
                .retain(|component| &component.status == status);
        }

        //Filter zone
        //info!("Zone:{:?}", filter_zone);
        if *filter_zone != Zone::GB {
            self.list_nodes
                .retain(|component| component.zone == *filter_zone);
            self.list_gateways
                .retain(|component| component.zone == *filter_zone);
        }
        Ok(())
    }

    pub fn get_check_steps(
        &self,
        blockchain: &String,
        component_type: &String,
        tasks: &Vec<TaskType>,
    ) -> Result<Vec<CheckStep>, anyhow::Error> {
        // info!("blockchain:{:?}", blockchain);
        // info!("component_type:{:?}", component_type);
        // info!("self.check_flows:{:?}", self.check_flows);
        let mut check_steps = Vec::new();
        for task in tasks {
            match self.check_flows.get(task.as_str()) {
                Some(check_flows) => {
                    for check_flow in check_flows {
                        if &check_flow.blockchain == blockchain
                            && &check_flow.component == component_type
                        {
                            check_steps.extend(check_flow.check_steps.clone());
                        }
                    }
                }
                None => {
                    warn!("Cannot find suitable task: {} in check-flow.json", task);
                }
            }
        }
        Ok(check_steps)
    }

    fn do_compare(
        operator: &OperatorCompare,
        step_result: &StepResult,
    ) -> Result<bool, anyhow::Error> {
        match operator.operator_type.as_str() {
            "and" => {
                let sub_actions: Vec<OperatorCompare> =
                    serde_json::from_value(operator.params.clone())?;
                let mut result = true;
                for sub_operator in sub_actions {
                    result = result && CheckComponent::do_compare(&sub_operator, step_result)?;
                }
                return Ok(result);
            }
            "eq" => {
                let items: Vec<String> = serde_json::from_value(operator.params.clone())?;
                let mut item_values = Vec::new();
                for mut item in items {
                    if item.starts_with("#") {
                        item.remove(0);
                        item_values.push(item.clone());
                    } else {
                        let item_value = step_result
                            .get(item.as_str())
                            .ok_or(anyhow::Error::msg("Cannot find key"))?;
                        item_values.push(item_value.clone());
                    }
                }
                debug!("item_values: {:?}", item_values);
                if item_values.len() > 1 {
                    let first = &item_values[0];
                    for item_value in item_values.iter() {
                        if item_value != first {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
            _ => return Err(anyhow::Error::msg("Cannot find key")),
        }
    }

    fn compare_action(
        &self,
        action: &ActionCompare,
        _node: &ComponentInfo,
        return_name: &String,
        step_result: &StepResult,
    ) -> Result<ActionResponse, anyhow::Error> {
        debug!(
            "Run Compare Action: {:?}, step_result: {:?}",
            action, step_result
        );

        let success = CheckComponent::do_compare(&action.operator_items, step_result)?;

        debug!("Run Compare Action success: {}", success);

        let mut result: HashMap<String, String> = HashMap::new();
        result.insert(return_name.clone(), success.to_string());
        return Ok(ActionResponse {
            success: true,
            conclude: match success {
                true => CheckMkStatus::Ok,
                false => CheckMkStatus::Unknown,
            },
            return_name: return_name.clone(),
            result,
            message: format!("compare items: {:?}", action.operator_items),
        });
    }

    fn replace_string(org: String, step_result: &StepResult) -> Result<String, anyhow::Error> {
        let handlebars = Handlebars::new();
        handlebars
            .render_template(org.as_str(), step_result)
            .map_err(|err| {
                println!("err:{:?}", err);
                anyhow::Error::msg(format!("{}", err))
            })
    }

    pub async fn call_action(
        &self,
        component: &ComponentInfo,
        step_result: &StepResult,
        step: &CheckStep,
    ) -> Result<ActionResponse, anyhow::Error> {
        let action: ActionCall = serde_json::from_value(step.action.clone()).unwrap();
        debug!("call action: {:?}", &action);

        if action.is_base_node {
            // Get base_endpoints
            let mut report = Err(anyhow::Error::msg("Cannot found working base node"));
            let base_endpoints =
                self.base_nodes
                    .get(&component.blockchain)
                    .ok_or(anyhow::Error::msg(format!(
                        "Cannot found base node for chain {:?}",
                        &component.blockchain
                    )))?;
            // Calling to Base node retry if failed
            for endpoint in base_endpoints {
                debug!("try endpoint:{:?}", endpoint);
                let res = self
                    .call_action_base_node(&action, &step.return_name, &step_result, endpoint)
                    .await;
                if res.is_ok() {
                    report = res;
                    debug!("endpoint {:?} success return: {:?}", endpoint, report);
                    break;
                }
            }
            report
        } else {
            // Calling check node only runs once
            self.call_action_check_node(&action, component, &step.return_name, &step_result)
                .await
        }
    }

    async fn call_action_check_node(
        &self,
        action: &ActionCall,
        node: &ComponentInfo,
        return_name: &String,
        step_result: &StepResult,
    ) -> Result<ActionResponse, anyhow::Error> {
        // prepare rpc call
        let node_url = node.get_url();
        let url = &node_url;

        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        // Replace body for transport result of previous step
        let body = Self::replace_string(action.body.clone(), step_result)?;

        debug!("body: {:?}", body);
        let request_builder = client
            .post(url)
            .header("content-type", "application/json")
            .header("x-api-key", node.token.as_str())
            .header("host", node.get_host_header(&self.domain))
            .body(body);

        debug!("request_builder: {:?}", request_builder);

        let sender = request_builder.send();
        pin_mut!(sender);

        // Call rpc
        // Start clock to meansure call time
        let now = Instant::now();
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(action.time_out as u64),
            &mut sender,
        )
        .await;
        //End clock
        let response_time_ms = now.elapsed().as_millis();

        let str_resp = res??.text().await?;
        debug!("response call: {:?}", str_resp);
        Self::prepare_result(&str_resp, response_time_ms, action, return_name)
    }

    fn prepare_result(
        str_resp: &String,
        response_time_ms: u128,
        action: &ActionCall,
        return_name: &String,
    ) -> Result<ActionResponse, anyhow::Error> {
        let mut str_resp_short = str_resp.clone();
        str_resp_short.truncate(CONFIG.max_length_report_detail);

        let resp: Value = serde_json::from_str(&str_resp).map_err(|e| {
            anyhow::Error::msg(format!(
                "Err {} when parsing response: {} ",
                e, str_resp_short,
            ))
        })?;

        // get result
        let mut result: HashMap<String, String> = HashMap::new();

        // Add response_time
        result.insert(
            CONFIG.response_time_key.to_string(),
            response_time_ms.to_string(),
        );

        for (name, path) in action.return_fields.clone() {
            let mut value = resp.clone();
            let path: Vec<String> = path.split("/").map(|s| s.to_string()).collect();
            //debug!("path: {:?}", path);
            for key in path.into_iter() {
                //debug!("key: {:?}", key);
                value = value
                    .get(key.clone())
                    .ok_or(anyhow::Error::msg(format!(
                        "cannot find key {} in result: {:?} ",
                        &key, resp
                    )))?
                    .clone();
                //debug!("value: {:?}", value);
            }
            // Fixme: check other type
            let str_value: String = match value.as_bool() {
                Some(value) => value.to_string(),
                None => value
                    .as_str()
                    .ok_or(anyhow::Error::msg(format!(
                        "value {:?} cannot parse to string",
                        value
                    )))?
                    .to_string(),
            };

            result.insert(name, str_value);
        }

        let action_resp = ActionResponse {
            success: true,
            conclude: CheckMkStatus::Ok,
            return_name: return_name.clone(),
            result,
            message: format!("call {}: {}", return_name.clone(), true),
        };

        debug!("action_resp: {:#?}", action_resp);

        Ok(action_resp)
    }

    async fn call_action_base_node(
        &self,
        action: &ActionCall,
        return_name: &String,
        step_result: &StepResult,
        base_endpoint: &EndpointInfo,
    ) -> Result<ActionResponse, anyhow::Error> {
        let url = &base_endpoint.url;

        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        // Replace body for transport result of previous step
        let body = Self::replace_string(action.body.clone(), step_result)?;

        debug!("body: {:?}", body);
        let request_builder = if base_endpoint.x_api_key.is_empty() {
            client
                .post(url)
                .header("content-type", "application/json")
                .body(body)
        } else {
            client
                .post(url)
                .header("content-type", "application/json")
                .header("x-api-key", base_endpoint.x_api_key.as_str())
                .body(body)
        };

        debug!("request_builder: {:?}", request_builder);

        let sender = request_builder.send();
        pin_mut!(sender);

        // Call rpc
        // Start clock to meansure call time
        let now = Instant::now();
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(action.time_out as u64),
            &mut sender,
        )
        .await;
        //End clock
        let response_time_ms = now.elapsed().as_millis();

        let str_resp = res??.text().await?;
        debug!("response call: {:?}", str_resp);
        // Prepare return result
        Self::prepare_result(&str_resp, response_time_ms, action, return_name)
    }

    pub async fn run_check_steps(
        &self,
        steps: Vec<CheckStep>,
        component: &ComponentInfo,
    ) -> Result<CheckMkReport, anyhow::Error> {
        let mut step_result: StepResult = HashMap::new();
        let mut status = CheckMkStatus::Ok;
        let mut message = String::new();
        let step_number = steps.len();
        let mut metric: HashMap<String, Value> = HashMap::new();

        for step in steps {
            debug!("step: {:?}", step);
            let report = match step
                .action
                .get("action_type")
                .unwrap()
                .as_str()
                .unwrap_or_default()
            {
                "call" => self.call_action(component, &step_result, &step).await,
                "compare" => {
                    let action: ActionCompare =
                        serde_json::from_value(step.action.clone()).unwrap();
                    debug!("compare action: {:?}", &action);
                    self.compare_action(&action, component, &step.return_name, &step_result)
                }
                _ => Err(anyhow::Error::msg("not support action")),
            };

            // Handle report
            match report {
                Ok(report) => {
                    let resp_time = if let Some(response_time_ms) =
                        report.result.get(&*CONFIG.response_time_key)
                    {
                        Some(response_time_ms.parse::<i64>().unwrap_or_default())
                    } else {
                        None
                    };
                    if let Some(resp_time) = resp_time {
                        let metric_name =
                            format!("{}_{}", report.return_name, CONFIG.response_time_key);
                        metric.insert(metric_name, resp_time.into());
                    }

                    match report.success {
                        true => {
                            debug!(
                                "Success step: {:?}, report: {:?}",
                                &step.return_name, report
                            );
                            for (key, value) in &report.result {
                                step_result.insert(
                                    format!("{}_{}", &report.return_name, key),
                                    value.clone(),
                                );
                            }
                        }
                        false => {
                            status = step.failed_case.conclude;
                            if step.failed_case.critical {
                                message.push_str(&format!(
                                    "Failed at step {} due to critical error, message: {}",
                                    &step.return_name, report.message
                                ));
                                break;
                            } else {
                                message.push_str(&format!(
                                    "Failed at step {}, message: {}",
                                    &step.return_name, report.message
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    message.push_str(&format!(
                        "Failed at step {}, err: {}.",
                        &step.return_name, e
                    ));
                    status = step.failed_case.conclude;
                    if step.failed_case.critical {
                        break;
                    }
                }
            }
        }
        let user = self.get_user(&component.user_id);
        let user_info = match user {
            None => "".to_string(),
            Some(user) => {
                format!("id:{},{},email:{}", user.id, user.name, user.email)
            }
        };
        if status == CheckMkStatus::Ok {
            message.push_str(format!("Succeed {} steps. ", step_number).as_str());
        }

        message.push_str(&user_info);
        Ok(CheckMkReport {
            status: status as u8,
            service_name: format!(
                "{}-http-{}-{}-{}-{}",
                component.component_type.to_string(),
                component.blockchain,
                component.network,
                component.id,
                component.ip
            ),
            metric: CheckMkMetric { metric },
            status_detail: message,
            success: true,
        })
    }

    pub async fn get_report_component(
        &self,
        component_info: &ComponentInfo,
    ) -> Result<(CheckMkReport, WrkReport), Error> {
        // Get logic report
        let mut check_mk_report = CheckMkReport::default();
        let mut wrk_report = WrkReport::default();
        debug!("component_info:{:?}", component_info);
        let check_steps = self
            .get_check_steps(
                &component_info.blockchain,
                &component_info.component_type.to_string(),
                &CONFIG.check_task_list_all,
            )
            .unwrap_or_default();
        debug!("check_steps:{:?}", check_steps);
        if check_steps.is_empty() {
            check_mk_report = CheckMkReport::new_failed_report("check_steps is empty".to_string())
        }
        let res_check_data = self.run_check_steps(check_steps, &component_info).await;
        info!("res:{:?}", res_check_data);

        let response_time_threshold = if component_info.component_type == ComponentType::Gateway {
            CONFIG.node_response_time_threshold_ms
        } else {
            CONFIG.gateway_response_time_threshold_ms
        };

        match res_check_data {
            Err(err) => check_mk_report = CheckMkReport::new_failed_report(format!("{:?}", err)),
            Ok(res_check_data) => {
                if res_check_data.success == true && res_check_data.status == 0 {
                    // logic report is ok and not skip_benchmark run benchmark
                    let res_benchmark = match CONFIG.skip_benchmark {
                        true => CheckMkReport {
                            status: 0,
                            service_name: "Skip benchmark".to_string(),
                            metric: Default::default(),
                            status_detail: "Skip benchmark".to_string(),
                            success: true,
                        },
                        false => {
                            wrk_report = self
                                .run_benchmark(response_time_threshold, &component_info)
                                .await?;

                            let res_benchmark = CheckMkReport::from_wrk_report(
                                wrk_report.clone(),
                                CONFIG.success_percent_threshold,
                                response_time_threshold,
                                CONFIG.accepted_low_latency_percent,
                            );
                            res_benchmark
                        }
                    };

                    check_mk_report =
                        CheckMkReport::combine_report(&res_check_data, &res_benchmark);
                } else {
                    check_mk_report = res_check_data;
                }
            }
        }
        Ok((check_mk_report, wrk_report))
    }

    fn get_benchmark_url(component: &ComponentInfo) -> String {
        match component.component_type {
            ComponentType::Node => {
                format!("https://{}", component.ip)
            }
            ComponentType::Gateway => {
                format!("https://{}", component.ip)
            }
            ComponentType::DApi => Default::default(),
        }
    }

    pub async fn run_benchmark(
        &self,
        response_time_threshold: f32,
        component: &ComponentInfo,
    ) -> Result<WrkReport, anyhow::Error> {
        let dapi_url = Self::get_benchmark_url(component);
        let mut host = Default::default();
        let mut path = Default::default();
        match component.component_type {
            ComponentType::Node => {
                host = format!("{}.node.mbr.{}", component.id, self.domain);
                path = "/".to_string();
            }
            ComponentType::Gateway => {
                host = format!("{}.gw.mbr.{}", component.id, self.domain);
                path = "/_test_20k".to_string();
            }
            ComponentType::DApi => {}
        };

        let mut benchmark = WrkBenchmark::build(
            CONFIG.benchmark_thread,
            CONFIG.benchmark_connection,
            CONFIG.benchmark_duration.to_string(),
            CONFIG.benchmark_rate,
            dapi_url,
            component.token.clone(),
            host,
            CONFIG.benchmark_script.to_string(),
            CONFIG.benchmark_wrk_path.to_string(),
            BENCHMARK_WRK_PATH.clone().to_string(),
            response_time_threshold,
        );
        benchmark.run(
            &component.component_type.to_string(),
            &path,
            &component.blockchain,
        )
    }

    //Using in fisherman service
    pub async fn check_components(
        &self,
        tasks: &Vec<TaskType>,
        components: &Vec<ComponentInfo>,
    ) -> Result<Vec<(ComponentInfo, CheckMkReport)>, anyhow::Error> {
        // Call node
        //http://cf242b49-907f-49ce-8621-4b7655be6bb8.node.mbr.massbitroute.com
        //header 'x-api-key: vnihqf14qk5km71aatvfr7c3djiej9l6mppd5k20uhs62p0b1cm79bfkmcubal9ug44e8cu2c74m29jpusokv6ft6r01o5bnv5v4gb8='
        // Check node
        let mut reports = Vec::new();
        for component in components {
            match self.get_report_component(&component).await {
                Ok((check_mk_report, wrk_report)) => {
                    // Store reports
                    reports.push((component.clone(), check_mk_report));
                }
                Err(e) => {
                    info!(
                        "Cannot get report for component {:?}, error: {:?}",
                        component, e
                    );
                }
            }
        }
        Ok(reports)
    }
}

pub struct GeneratorBuilder {
    inner: CheckComponent,
}

impl Default for GeneratorBuilder {
    fn default() -> Self {
        Self {
            inner: CheckComponent {
                list_node_id_file: "".to_string(),
                list_gateway_id_file: "".to_string(),
                list_dapi_id_file: "".to_string(),
                list_user_file: "".to_string(),
                check_flow_file: "".to_string(),
                base_endpoint_file: "".to_string(),
                domain: "".to_string(),
                output_file: "".to_string(),
                list_nodes: vec![],
                list_gateways: vec![],
                list_dapis: vec![],
                list_users: vec![],
                base_nodes: Default::default(),
                check_flows: Default::default(),
                is_loop_check: false,
                is_write_to_file: false,
            },
        }
    }
}

impl GeneratorBuilder {
    pub async fn with_list_node_id_file(
        mut self,
        path: String,
        status: Option<String>,
    ) -> GeneratorBuilder {
        self.inner.list_node_id_file = path.clone();
        self
    }

    pub async fn with_list_gateway_id_file(
        mut self,
        path: String,
        status: Option<String>,
    ) -> GeneratorBuilder {
        self.inner.list_gateway_id_file = path.clone();
        self
    }
    pub async fn with_list_dapi_id_file(mut self, path: String) -> GeneratorBuilder {
        self.inner.list_dapi_id_file = path.clone();
        self
    }
    pub async fn with_list_user_file(mut self, path: String) -> GeneratorBuilder {
        self.inner.list_user_file = path.clone();

        let list_users: Vec<UserInfo> = self.get_list_user().await.unwrap_or_default();
        debug!("list users: {:?}", &list_users);
        self.inner.list_users = list_users;
        self
    }

    async fn get_list_user(&self) -> Result<Vec<UserInfo>, anyhow::Error> {
        let mut users: Vec<UserInfo> = Vec::new();
        debug!("----------Create list of users info details----------");
        let list_id_file = &self.inner.list_user_file;
        let lines: Vec<String> = match list_id_file.starts_with("http") {
            true => {
                let url = list_id_file;
                debug!("url:{}", url);
                let node_data = reqwest::get(url).await?.text().await?;
                debug!("node_data: {}", node_data);
                let lines: Vec<String> = node_data.split("\n").map(|s| s.to_string()).collect();
                lines
            }
            false => {
                let file = File::open(list_id_file)?;
                let reader = BufReader::new(file);
                reader.lines().into_iter().filter_map(|s| s.ok()).collect()
            }
        };
        for line in lines {
            if !line.is_empty() {
                //println!("line: {}", &line);
                //debug!("line: {:?}", &line);
                let data: Vec<String> = line.split(' ').map(|piece| piece.to_string()).collect();
                //debug!("data: {:?}", &data);

                let user =
                    // 298eef2b-5fa2-4a3d-b00c-fe95b01e237c zhangpanyi@live.com zhangpanyi@live.com true
                    UserInfo {
                        name: data[1].clone(),
                        id: data[0].clone(),
                        email: data[2].clone(),
                        verified: data[3].clone().parse::<bool>().unwrap_or_default()
                    };

                users.push(user);
            }
        }

        return Ok(users);
    }

    pub fn with_check_flow_file(mut self, path: String) -> Self {
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("Unable to read `{}`: {}", path, err));
        debug!("json: {:#?}", json);
        let test_flow: CheckFlows = serde_json::from_str(&minify(&json)).unwrap();
        debug!("test_flow: {:#?}", test_flow);
        self.inner.check_flow_file = path;
        self.inner.check_flows = test_flow;
        self
    }
    pub fn with_domain(mut self, path: String) -> Self {
        self.inner.domain = path;
        self
    }
    pub fn with_base_endpoint_file(mut self, path: String) -> Self {
        let json = if !path.is_empty() {
            std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("Unable to read `{}`: {}", path, err))
        } else {
            println!("Load base endpoint from env: \n{:?}", *BASE_ENDPOINT_JSON);
            BASE_ENDPOINT_JSON.clone()
        };

        let base_nodes: HashMap<BlockChainType, Vec<EndpointInfo>> =
            serde_json::from_str(&minify(&json)).unwrap_or_default();
        self.inner.base_endpoint_file = path;
        self.inner.base_nodes = base_nodes;
        self
    }
    pub fn with_output_file(mut self, output_file: String) -> Self {
        self.inner.output_file = output_file;
        self
    }

    pub fn build(self) -> CheckComponent {
        self.inner
    }
}
