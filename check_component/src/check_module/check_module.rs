use anyhow::{anyhow, Error};
use chrono::Duration;
use clap::StructOpt;
use futures::pin_mut;
use futures_util::future::{err, join_all};
use minifier::json::minify;
use reqwest::{Body, Response};
use serde::{forward_to_deserialize_any_helper, Deserialize, Serialize};

use handlebars::{Handlebars, RenderError};
use log::{debug, error, info, log_enabled, Level};
use serde_json::{to_string, Number, Value};
use std::any::Any;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::format;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ptr::hash;
use std::time::Instant;
use std::{thread, usize};
use timer::Timer;
use tokio::time::error::Elapsed;
use warp::multipart::FormData;
use warp::reject::custom;
use warp::{Rejection, Reply};

type BlockChainType = String;
type UrlType = String;
type TaskType = String;
type StepResult = HashMap<String, String>;
type ComponentId = String;

const RESPONSE_TIME_KEY: &str = "response_time_ms";

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
    #[serde(rename = "userId")]
    pub user_id: String,
    pub ip: String,
    pub zone: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    #[serde(skip_serializing, default)]
    pub token: String,
    #[serde(skip_serializing, default)]
    pub component_type: ComponentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
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
    // fn get_node_url(&self) -> UrlType {
    //     format!("http://{}.node.mbr.massbitroute.com", self.id)
    // }

    fn get_url(&self) -> UrlType {
        format!("https://{}", self.ip)
    }

    fn get_host_header(&self, domain: &String) -> String {
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

    fn get_component_same_chain(
        &self,
        list_components: &Vec<ComponentInfo>,
    ) -> Option<ComponentInfo> {
        let mut result = None;

        for component in list_components {
            if (component.blockchain == self.blockchain) && (component.network == self.network) {
                result = Some(component.clone());
                break;
            }
        }
        result
    }

    // fn get_gateway_url(&self, fix_dapi: &ComponentInfo) -> UrlType {
    //     // http://34.88.83.191/cb8a7cef-0ebd-4ce7-a39f-6c0d4ddd5f3a
    //     format!("http://{ip}/{dapi_id}", ip = self.ip, dapi_id = fix_dapi.id)
    // }
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
    pub base_nodes: HashMap<BlockChainType, UrlType>,
    pub check_flows: CheckFlows,
    pub is_loop_check: bool,
    pub is_write_to_file: bool,
}

type CheckFlows = HashMap<TaskType, Vec<CheckFlow>>;
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

enum Component {
    Node(ComponentInfo),
    Gateway(ComponentInfo),
    DApi(ComponentInfo),
}

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

const CHECK_INTERVAL: u64 = 2; // sec unit

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct FailedCase {
    #[serde(default)]
    critical: bool,
    #[serde(default)]
    message: String,
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
struct ActionResponse {
    success: bool,
    return_name: String,
    result: HashMap<String, String>,
    message: String,
}

#[derive(PartialEq)]
enum CheckMkStatus {
    Ok = 0,
    Warning = 1,
    Critical = 2,
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
    fn to_json(&self) -> String {
        format!(
            r#"{{"status":{status},"service_name":{service_name},"metric":{metric},"status_detail":{status_detail}}}"#,
            status = &self.status,
            service_name = &self.service_name,
            metric = self.metric.to_string(),
            status_detail = self.status_detail
        )
    }
    fn new_failed_report(msg: String) -> Self {
        let mut resp = CheckMkReport::default();
        resp.success = false;
        resp.status_detail = msg.to_string();
        resp
    }
    pub fn is_component_status_ok(&self) -> bool {
        (self.success == true) && (self.status == 0)
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
        status: Option<String>,
    ) -> Result<(), anyhow::Error> {
        let url = &self.list_node_id_file;
        log::debug!("url:{}", url);
        let res_data = reqwest::get(url).await?.text().await?;
        info!("res_data: {:?}", res_data);
        self.list_nodes = serde_json::from_str(res_data.as_str()).unwrap();

        let url = &self.list_gateway_id_file;
        log::debug!("url:{}", url);
        let res_data = reqwest::get(url).await?.text().await?;
        info!("res_data: {:?}", res_data);
        self.list_gateways = serde_json::from_str(res_data.as_str()).unwrap();

        //Filter components
        if let Some(status) = status {
            self.list_nodes
                .retain(|component| *component.status == status);
            self.list_gateways
                .retain(|component| *component.status == status);
        }

        Ok(())
    }

    pub fn get_check_steps(
        &self,
        blockchain: &String,
        component_type: &String,
        task: &TaskType,
    ) -> Result<Vec<CheckStep>, anyhow::Error> {
        // log::info!("blockchain:{:?}", blockchain);
        // log::info!("component_type:{:?}", component_type);
        // log::info!("self.check_flows:{:?}", self.check_flows);
        match self.check_flows.get(task.as_str()) {
            Some(check_flows) => {
                for check_flow in check_flows {
                    if &check_flow.blockchain == blockchain
                        && &check_flow.component == component_type
                    {
                        return Ok(check_flow.check_steps.clone());
                    }
                }
                Err(anyhow::Error::msg(
                    "Cannot find suitable check_flow in Task",
                ))
            }
            None => Err(anyhow::Error::msg(
                "Cannot find suitable task in check-flow.json",
            )),
        }
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
                for item in items {
                    let item_value = step_result
                        .get(item.as_str())
                        .ok_or(anyhow::Error::msg("Cannot find key"))?;
                    item_values.push(item_value);
                }
                log::debug!("item_values: {:?}", item_values);
                if item_values.len() > 1 {
                    let first = item_values[0];
                    for item_value in item_values {
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
        node: &ComponentInfo,
        return_name: &String,
        step_result: &StepResult,
    ) -> Result<ActionResponse, anyhow::Error> {
        log::debug!(
            "Run Compare Action: {:?}, step_result: {:?}",
            action,
            step_result
        );

        let success = CheckComponent::do_compare(&action.operator_items, step_result)?;

        log::debug!("Run Compare Action success: {}", success);

        let mut result: HashMap<String, String> = HashMap::new();
        result.insert(return_name.clone(), success.to_string());
        return Ok(ActionResponse {
            success,
            return_name: return_name.clone(),
            result,
            message: format!("compare items: {:?}", action.operator_items),
        });
    }

    fn replace_string(org: String, step_result: &StepResult) -> Result<String, anyhow::Error> {
        let mut handlebars = Handlebars::new();
        handlebars
            .render_template(org.as_str(), step_result)
            .map_err(|err| {
                println!("err:{:?}", err);
                anyhow::Error::msg(format!("{}", err))
            })
    }

    async fn call_action(
        &self,
        action: &ActionCall,
        node: &ComponentInfo,
        return_name: &String,
        step_result: &StepResult,
    ) -> Result<ActionResponse, anyhow::Error> {
        // prepare rpc call
        //let ref_dapi = node.get_component_same_chain(&self.list_dapis).unwrap();
        let node_url = node.get_url();

        let url = match action.is_base_node {
            true => self.base_nodes.get(node.blockchain.as_str()),
            false => Some(&node_url),
        }
        .ok_or(anyhow::Error::msg("Cannot get url"))?;
        let mut client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        // Replace body for transport result of previous step
        let body = Self::replace_string(action.body.clone(), step_result)?;

        let mut request_builder = client
            .post(url)
            .header("content-type", "application/json")
            .header("x-api-key", node.token.as_str())
            .header("host", node.get_host_header(&self.domain))
            .body(body);
        log::debug!("request_builder: {:?}", request_builder);

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

        let resp = res??.text().await?;

        let resp: Value = serde_json::from_str(&resp)?;
        log::debug!("response call: {:?}", resp);

        // get result
        let mut result: HashMap<String, String> = HashMap::new();

        // Add response_time
        result.insert(RESPONSE_TIME_KEY.to_string(), response_time_ms.to_string());

        for (name, path) in action.return_fields.clone() {
            let mut value = resp.clone();
            let path: Vec<String> = path.split("/").map(|s| s.to_string()).collect();
            //log::debug!("path: {:?}", path);
            for key in path.into_iter() {
                //log::debug!("key: {:?}", key);
                value = value
                    .get(key.clone())
                    .ok_or(anyhow::Error::msg(format!(
                        "cannot find key {} in result ",
                        &key
                    )))?
                    .clone();
                //log::debug!("value: {:?}", value);
            }
            result.insert(name, value.as_str().unwrap().to_string());
        }

        let action_resp = ActionResponse {
            success: true,
            return_name: return_name.clone(),
            result,
            message: format!("call {}: {}", return_name.clone(), true),
        };

        log::debug!("action_resp: {:#?}", action_resp);

        Ok(action_resp)
    }

    pub async fn run_check_steps(
        &self,
        steps: Vec<CheckStep>,
        component: &ComponentInfo,
    ) -> Result<(ComponentInfo, CheckMkReport), anyhow::Error> {
        let mut step_result: StepResult = HashMap::new();
        let mut status = CheckMkStatus::Ok;
        let mut message = String::new();
        let step_number = steps.len();
        let mut metric: HashMap<String, Value> = HashMap::new();

        for step in steps {
            log::debug!("step: {:?}", step);
            let report = match step
                .action
                .get("action_type")
                .unwrap()
                .as_str()
                .unwrap_or_default()
            {
                "call" => {
                    let action: ActionCall = serde_json::from_value(step.action.clone()).unwrap();
                    log::debug!("call action: {:?}", &action);
                    self.call_action(&action, component, &step.return_name, &step_result)
                        .await
                }
                "compare" => {
                    let action: ActionCompare =
                        serde_json::from_value(step.action.clone()).unwrap();
                    log::debug!("compare action: {:?}", &action);
                    self.compare_action(&action, component, &step.return_name, &step_result)
                }
                _ => Err(anyhow::Error::msg("not support action")),
            };

            //log::debug!("report: {:?}", report);

            // Handle report
            match report {
                Ok(report) => {
                    let resp_time =
                        if let Some(response_time_ms) = report.result.get(RESPONSE_TIME_KEY) {
                            Some(response_time_ms.parse::<i64>().unwrap_or_default())
                        } else {
                            None
                        };
                    if let Some(resp_time) = resp_time {
                        let metric_name = format!("{}_{}", report.return_name, RESPONSE_TIME_KEY);
                        metric.insert(metric_name, resp_time.into());
                    }

                    match report.success {
                        true => {
                            log::debug!(
                                "Success step: {:?}, report: {:?}",
                                &step.return_name,
                                report
                            );
                            for (key, value) in &report.result {
                                step_result.insert(
                                    format!("{}_{}", &report.return_name, key),
                                    value.clone(),
                                );
                            }
                        }
                        false => {
                            if step.failed_case.critical {
                                status = CheckMkStatus::Critical;
                                message.push_str(&format!(
                                    "Failed at step {} due to critical error, message: {}",
                                    &step.return_name, report.message
                                ));
                                break;
                            } else {
                                status = CheckMkStatus::Warning;
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
                    if step.failed_case.critical {
                        status = CheckMkStatus::Critical;
                        break;
                    } else {
                        status = CheckMkStatus::Warning;
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
        Ok((
            component.clone(),
            CheckMkReport {
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
            },
        ))
    }

    pub(crate) async fn get_components_status(
        &self,
        component_info: ComponentInfo,
    ) -> Result<impl Reply, Rejection> {
        //-> HashMap<String, CheckMkStatus>
        info!("component_info:{:?}", component_info);

        let check_steps = self
            .get_check_steps(
                &component_info.blockchain,
                &component_info.component_type.to_string(),
                &"checking_chain_type".to_string(),
            )
            .unwrap_or_default();
        info!("check_steps:{:?}", check_steps);
        if check_steps.is_empty() {
            return Ok(warp::reply::json(&CheckMkReport::new_failed_report(
                "check_steps is empty".to_string(),
            )));
        }
        let res = self.run_check_steps(check_steps, &component_info).await;
        info!("res:{:?}", res);
        match res {
            Ok(res) => Ok(warp::reply::json(&res.1)),
            Err(err) => Ok(warp::reply::json(&CheckMkReport::new_failed_report(
                format!("{:?}", err),
            ))),
        }
    }

    pub async fn check_components(
        &self,
    ) -> Result<Vec<(ComponentInfo, CheckMkReport)>, anyhow::Error> {
        log::info!("Check components");
        // Call node
        //http://cf242b49-907f-49ce-8621-4b7655be6bb8.node.mbr.massbitroute.com
        //header 'x-api-key: vnihqf14qk5km71aatvfr7c3djiej9l6mppd5k20uhs62p0b1cm79bfkmcubal9ug44e8cu2c74m29jpusokv6ft6r01o5bnv5v4gb8='
        let mut reports: Vec<CheckMkReport> = Vec::new();

        // Check node
        let nodes = &self.list_nodes;
        let mut tasks = Vec::new();
        for node in nodes {
            let steps = self.get_check_steps(
                &node.blockchain,
                &"node".to_string(),
                &"checking_chain_type".to_string(),
            );

            match steps {
                Ok(steps) => {
                    //log::debug!("steps:{:#?}", steps);
                    //log::info!("Do the check steps");
                    tasks.push(self.run_check_steps(steps, node));
                }
                Err(err) => {
                    log::debug!("There are no check steps");
                }
            };
        }

        // Check Gateway
        let gateways = &self.list_gateways;
        for gateway in gateways {
            let steps = self.get_check_steps(
                &gateway.blockchain,
                &"gateway".to_string(),
                &"checking_chain_type".to_string(),
            );

            match steps {
                Ok(steps) => {
                    //log::debug!("steps:{:#?}", steps);
                    //log::info!("Do the check steps");
                    tasks.push(self.run_check_steps(steps, gateway));
                }
                Err(err) => {
                    log::debug!("There are no check steps");
                }
            };
        }

        let responses = join_all(tasks).await;
        let reports = responses
            .into_iter()
            .filter_map(|report| report.ok())
            .collect();

        Ok(reports)
    }

    pub async fn run_check(&self, check_interval_ms: u64) -> Result<(), anyhow::Error> {
        // Begin run check
        log::debug!("run_check");
        // Infinity run check loop
        loop {
            log::info!("check_components");
            let reports = self.check_components().await;
            log::info!("reports:{:?}", reports);
            // Write reports to file
            match reports {
                Ok(reports) => {
                    let report_content: String = reports
                        .iter()
                        .map(|(component, report)| report.to_string())
                        .collect::<Vec<String>>()
                        .join("\n");
                    if self.is_write_to_file {
                        std::fs::write(self.output_file.clone(), report_content)
                            .expect("Unable to write file");
                    } else {
                        println!("{}", report_content);
                    }
                }
                Err(err) => (),
            }
            if !self.is_loop_check {
                break;
            }
            // Repeat every CHECK_INTERVAL secs
            thread::sleep(::std::time::Duration::from_millis(check_interval_ms));
        }

        Ok(())
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

        let list_nodes: Vec<ComponentInfo> = self
            .get_list_component(ComponentType::Node, status)
            .await
            .unwrap_or_default();
        info!("list_nodes:{:?}", list_nodes);
        self.inner.list_nodes = list_nodes;
        self
    }

    pub async fn with_list_gateway_id_file(
        mut self,
        path: String,
        status: Option<String>,
    ) -> GeneratorBuilder {
        self.inner.list_gateway_id_file = path.clone();

        let list_gateways: Vec<ComponentInfo> = self
            .get_list_component(ComponentType::Gateway, status)
            .await
            .unwrap_or_default();
        info!("list_gateways:{:?}", list_gateways);
        self.inner.list_gateways = list_gateways;
        self
    }
    pub async fn with_list_dapi_id_file(mut self, path: String) -> GeneratorBuilder {
        self.inner.list_dapi_id_file = path.clone();

        // let list_dapis: Vec<ComponentInfo> = self
        //     .get_list_component(ComponentType::DApi)
        //     .await
        //     .unwrap_or_default();
        // self.inner.list_dapis = list_dapis;
        self
    }
    pub async fn with_list_user_file(mut self, path: String) -> GeneratorBuilder {
        self.inner.list_user_file = path.clone();

        let list_users: Vec<UserInfo> = self.get_list_user().await.unwrap_or_default();
        log::debug!("list users: {:?}", &list_users);
        self.inner.list_users = list_users;
        self
    }

    async fn get_list_user(&self) -> Result<Vec<UserInfo>, anyhow::Error> {
        let mut users: Vec<UserInfo> = Vec::new();
        log::debug!("----------Create list of users info details----------");
        let list_id_file = &self.inner.list_user_file;
        let lines: Vec<String> = match list_id_file.starts_with("http") {
            true => {
                let url = list_id_file;
                log::debug!("url:{}", url);
                let node_data = reqwest::get(url).await?.text().await?;
                log::debug!("node_data: {}", node_data);
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
                //log::debug!("line: {:?}", &line);
                let data: Vec<String> = line.split(' ').map(|piece| piece.to_string()).collect();
                //log::debug!("data: {:?}", &data);

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

    async fn get_list_component(
        &self,
        component_type: ComponentType,
        status: Option<String>,
    ) -> Result<Vec<ComponentInfo>, anyhow::Error> {
        let mut components: Vec<ComponentInfo> = Vec::new();
        log::debug!("----------Create list of nodes info details----------");
        let list_id_file = match component_type {
            ComponentType::Node => &self.inner.list_node_id_file,
            ComponentType::Gateway => &self.inner.list_gateway_id_file,
            ComponentType::DApi => &self.inner.list_dapi_id_file,
        };
        let mut lines: Vec<String> = Vec::new();
        match list_id_file.starts_with("http") {
            true => {
                let url = list_id_file;
                log::debug!("url:{}", url);
                let res_data = reqwest::get(url).await?.text().await?;
                info!("res_data: {:?}", res_data);
                components = serde_json::from_str(res_data.as_str()).unwrap();
            }
            false => {
                let file = File::open(list_id_file)?;
                let reader = BufReader::new(file);
                lines = reader.lines().into_iter().filter_map(|s| s.ok()).collect()
            }
        };
        for line in lines {
            if !line.is_empty() {
                //println!("line: {}", &line);
                //log::debug!("line: {:?}", &line);
                let data: Vec<String> = line.split(' ').map(|piece| piece.to_string()).collect();
                //log::debug!("data: {:?}", &data);

                let node = match component_type {
                    ComponentType::Node | ComponentType::Gateway => ComponentInfo {
                        blockchain: data[2].clone(),
                        network: data[3].clone(),
                        id: data[0].clone(),
                        user_id: data[1].clone(),
                        ip: data[4].clone(),
                        zone: data[5].clone(),
                        country_code: data[6].clone(),
                        token: data[7].clone(),
                        component_type: component_type.clone(),
                        endpoint: Default::default(),
                        status: "".to_string(),
                    },
                    //77762f16-4344-4c58-9851-9e2e4488a2c0 87f54452-18e7-4582-b699-110e061a6248 ftm mainnet cd0pfbm2tjq3.ftm-mainnet.massbitroute.com 77762f16-4344-4c58-9851-9e2e4488a2c0
                    ComponentType::DApi => ComponentInfo {
                        blockchain: data[2].clone(),
                        network: data[3].clone(),
                        id: data[0].clone(),
                        user_id: data[1].clone(),
                        ip: Default::default(),
                        zone: Default::default(),
                        country_code: Default::default(),
                        token: Default::default(),
                        component_type: component_type.clone(),
                        endpoint: Some(data[4].clone()),
                        status: "".to_string(),
                    },
                };

                components.push(node);
            }
        }

        //Filter components
        if let Some(status) = status {
            components.retain(|component| *component.status == status);
        }

        return Ok(components);
    }

    // pub fn get_token(node_data: String) -> Result<String,anyhow::Error> {
    //     let data: Value = serde_json::from_str(&minify(&node_data))?;
    //     let token: Option<String> = data.get("token").and_then(|value| Some(value.to_string()));
    //
    //     log::debug!("data: {:#?}",data);
    //     Ok(token.expect("cannot get token"))
    // }

    pub fn with_check_flow_file(mut self, path: String) -> Self {
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("Unable to read `{}`: {}", path, err));
        log::info!("json: {:#?}", json);
        let test_flow: CheckFlows = serde_json::from_str(&minify(&json)).unwrap_or_default();
        self.inner.check_flow_file = path;
        self.inner.check_flows = test_flow;
        self
    }
    pub fn with_domain(mut self, path: String) -> Self {
        self.inner.domain = path;
        self
    }
    pub fn with_base_endpoint_file(mut self, path: String) -> Self {
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("Unable to read `{}`: {}", path, err));
        let base_nodes: HashMap<BlockChainType, UrlType> =
            serde_json::from_str(&minify(&json)).unwrap_or_default();
        self.inner.check_flow_file = path;
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
