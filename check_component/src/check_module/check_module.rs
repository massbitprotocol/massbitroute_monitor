use anyhow::{anyhow, Error};
use chrono::Duration;
use clap::StructOpt;
use futures::pin_mut;
use futures_util::future::{err, join_all};
use minifier::json::minify;
use reqwest::{Body, Response};
use serde::{forward_to_deserialize_any_helper, Deserialize, Serialize};

use crate::check_module::check_module::ComponentType::Node;
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
use std::thread;
use timer::Timer;
use tokio::time::error::Elapsed;

// use futures_util::future::try_future::TryFutureExt;

type BlockChainType = String;
type UrlType = String;
type TaskType = String;
type StepResult = HashMap<String, String>;

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

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ComponentInfo {
    pub blockchain: BlockChainType,
    pub network: String,
    pub id: String,
    pub user_id: String,
    pub ip: String,
    pub zone: String,
    pub country_code: String,
    pub token: String,
    pub component_type: ComponentType,
    pub endpoint: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct UserInfo {
    pub name: String,
    pub id: String,
    pub email: String,
    pub verified: bool,
}

impl ComponentInfo {
    fn get_node_url(&self) -> UrlType {
        format!("http://{}.node.mbr.massbitroute.com", self.id)
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

    fn get_gateway_url(&self, fix_dapi: &ComponentInfo) -> UrlType {
        // http://34.88.83.191/cb8a7cef-0ebd-4ce7-a39f-6c0d4ddd5f3a
        format!("http://{ip}/{dapi_id}", ip = self.ip, dapi_id = fix_dapi.id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckComponent<'a> {
    // input file
    pub list_node_id_file: &'a str,
    pub list_gateway_id_file: &'a str,
    pub list_dapi_id_file: &'a str,
    pub list_user_file: &'a str,
    pub check_flow_file: &'a str,
    pub base_endpoint_file: &'a str,
    // The output file
    pub output_file: &'a str,
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

struct GatewayInfo;
struct DApiInfo;

enum Component {
    Node(ComponentInfo),
    Gateway(GatewayInfo),
    DApi(DApiInfo),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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
    status: u8,
    service_name: String,
    metric: Option<CheckMkMetric>,
    status_detail: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
struct CheckMkMetric {
    metric: HashMap<String, Vec<String>>,
}

impl ToString for CheckMkMetric {
    fn to_string(&self) -> String {
        self.metric
            .iter()
            .map(|(key, val)| format!("{}={}", key, val.join(";")))
            .collect::<Vec<String>>()
            .join("|")
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
        match self.metric.clone() {
            None => format!(
                r#"{status} "{service_name}" - {status_detail}"#,
                status = &self.status,
                service_name = &self.service_name,
                status_detail = self.status_detail
            ),
            Some(metric) => format!(
                r#"{status} {service_name} {metric} {status_detail}"#,
                status = &self.status,
                service_name = &self.service_name,
                metric = metric.to_string(),
                status_detail = self.status_detail
            ),
        }
    }
}

impl<'a> CheckComponent<'a> {
    pub fn builder() -> GeneratorBuilder<'a> {
        GeneratorBuilder::default()
    }

    fn get_user(&self, user_id: &String) -> Option<&UserInfo> {
        self.list_users.iter().find(|user| &user.id == user_id)
    }

    fn get_check_steps(
        &self,
        blockchain: &String,
        component: &String,
        task: &TaskType,
    ) -> Result<Vec<CheckStep>, anyhow::Error> {
        match self.check_flows.get(task.as_str()) {
            Some(check_flows) => {
                for check_flow in check_flows {
                    //log::debug!("check_flow.blockchain: {},blockchain: {}\ncheck_flow.component:{},component:{}",&check_flow.blockchain,&blockchain,check_flow.component,component);
                    if &check_flow.blockchain == blockchain && &check_flow.component == component {
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
        // Get url
        //log::debug!("self.base_nodes: {:?}", self.base_nodes);

        // prepare rpc call
        let ref_dapi = node.get_component_same_chain(&self.list_dapis).unwrap();
        let node_url = match node.component_type {
            ComponentType::Node => node.get_node_url(),
            ComponentType::Gateway => {
                let url = node.get_gateway_url(&ref_dapi);
                //println!("gateway url: {}", url);
                url
            }
            ComponentType::DApi => String::default(),
        };

        let url = match action.is_base_node {
            true => self.base_nodes.get(node.blockchain.as_str()),
            false => Some(&node_url),
        }
        .ok_or(anyhow::Error::msg("Cannot get url"))?;
        //println!("org:  {}", action.body);
        //println!("step_result:{:?}", step_result);
        // Replace body for transport result of previous step
        let body = Self::replace_string(action.body.clone(), step_result)?;
        //println!("body: {}", body);

        let mut client = reqwest::Client::new()
            .post(url)
            .header("content-type", "application/json");
        client = match node.component_type {
            ComponentType::Node => client.header("X-Api-Key", node.token.as_str()).body(body),
            ComponentType::Gateway => client.header("Host", ref_dapi.endpoint.as_str()).body(body),
            ComponentType::DApi => client,
        };
        log::debug!("client: {:?}", client);

        let sender = client.send();
        pin_mut!(sender);

        // Call rpc
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(action.time_out as u64),
            &mut sender,
        )
        .await;
        let resp = res??.text().await?;
        //let resp = client.send().await?.text().await?;
        let resp: Value = serde_json::from_str(&resp)?;
        //log::debug!("resp: {:?}", resp);

        //get result
        let mut result: HashMap<String, String> = HashMap::new();
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

    async fn run_check_steps(
        &self,
        steps: Vec<CheckStep>,
        component: &ComponentInfo,
    ) -> Result<CheckMkReport, anyhow::Error> {
        let mut step_result: StepResult = HashMap::new();
        let mut status = CheckMkStatus::Ok;
        let mut message = String::new();

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
                Ok(report) => match report.success {
                    true => {
                        log::debug!(
                            "Success step: {:?}, report: {:?}",
                            &step.return_name,
                            report
                        );
                        for (key, value) in &report.result {
                            step_result
                                .insert(format!("{}_{}", &report.return_name, key), value.clone());
                        }
                    }
                    false => {
                        if step.failed_case.critical {
                            status = CheckMkStatus::Critical;
                            message.push_str(&format!(
                                "Stop at step {} due to critical error",
                                &step.return_name
                            ));
                            break;
                        } else {
                            status = CheckMkStatus::Warning;
                            message.push_str(&format!("Failed at step {}.", &step.return_name));
                        }
                    }
                },
                Err(e) => {
                    if step.failed_case.critical {
                        status = CheckMkStatus::Critical;
                        message.push_str(&format!("Failed at step {}.", &step.return_name));
                        break;
                    } else {
                        status = CheckMkStatus::Warning;
                        message.push_str(&format!("Failed at step {}.", &step.return_name));
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
            message.push_str("Succeed ");
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
            metric: None,
            status_detail: message,
        })
    }

    async fn check_components(&self) -> Result<Vec<CheckMkReport>, anyhow::Error> {
        log::debug!("Check component");
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
                    log::debug!("Do the check steps");
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
                    log::debug!("Do the check steps");
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

    pub async fn run_check(&self) -> Result<(), anyhow::Error> {
        // Begin run check
        log::debug!("run_check");
        // Infinity run check loop
        loop {
            let reports = self.check_components().await;

            // Write reports to file
            match reports {
                Ok(reports) => {
                    let report_content: String = reports
                        .iter()
                        .map(|report| report.to_string())
                        .collect::<Vec<String>>()
                        .join("\n");
                    if self.is_write_to_file {
                        std::fs::write(self.output_file, report_content)
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
            thread::sleep(::std::time::Duration::new(CHECK_INTERVAL, 0));
        }

        Ok(())
    }
}

pub struct GeneratorBuilder<'a> {
    inner: CheckComponent<'a>,
}

impl<'a> Default for GeneratorBuilder<'a> {
    fn default() -> Self {
        Self {
            inner: CheckComponent {
                list_node_id_file: "",
                list_gateway_id_file: "",
                list_dapi_id_file: "",
                list_user_file: "",
                check_flow_file: "",
                base_endpoint_file: "",
                output_file: "",
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

impl<'a> GeneratorBuilder<'a> {
    pub async fn with_list_node_id_file(mut self, path: &'a str) -> GeneratorBuilder<'a> {
        self.inner.list_node_id_file = path;

        let list_nodes: Vec<ComponentInfo> = self
            .get_list_component(ComponentType::Node)
            .await
            .unwrap_or_else(|err| panic!("Cannot parse `{}` as JSON: {}", path, err));
        self.inner.list_nodes = list_nodes;
        self
    }
    pub async fn with_list_gateway_id_file(mut self, path: &'a str) -> GeneratorBuilder<'a> {
        self.inner.list_gateway_id_file = path;

        let list_gateways: Vec<ComponentInfo> = self
            .get_list_component(ComponentType::Gateway)
            .await
            .unwrap_or_else(|err| panic!("Cannot parse `{}` as JSON: {}", path, err));
        self.inner.list_gateways = list_gateways;
        self
    }
    pub async fn with_list_dapi_id_file(mut self, path: &'a str) -> GeneratorBuilder<'a> {
        self.inner.list_dapi_id_file = path;

        let list_dapis: Vec<ComponentInfo> = self
            .get_list_component(ComponentType::DApi)
            .await
            .unwrap_or_else(|err| panic!("Cannot parse `{}` as JSON: {}", path, err));
        self.inner.list_dapis = list_dapis;
        self
    }
    pub async fn with_list_user_file(mut self, path: &'a str) -> GeneratorBuilder<'a> {
        self.inner.list_user_file = path;

        let list_users: Vec<UserInfo> = self
            .get_list_user()
            .await
            .unwrap_or_else(|err| panic!("Cannot parse `{}` as JSON: {}", path, err));
        log::debug!("list users: {:?}", &list_users);
        self.inner.list_users = list_users;
        self
    }

    async fn get_list_user(&self) -> Result<Vec<UserInfo>, anyhow::Error> {
        let mut users: Vec<UserInfo> = Vec::new();
        log::debug!("----------Create list of users info details----------");
        let list_id_file = self.inner.list_user_file;
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
    ) -> Result<Vec<ComponentInfo>, anyhow::Error> {
        let mut nodes: Vec<ComponentInfo> = Vec::new();
        log::debug!("----------Create list of nodes info details----------");
        let list_id_file = match component_type {
            ComponentType::Node => self.inner.list_node_id_file,
            ComponentType::Gateway => self.inner.list_gateway_id_file,
            ComponentType::DApi => self.inner.list_dapi_id_file,
        };
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
                        endpoint: data[4].clone(),
                    },
                };

                nodes.push(node);
            }
        }

        return Ok(nodes);
    }

    // pub fn get_token(node_data: String) -> Result<String,anyhow::Error> {
    //     let data: Value = serde_json::from_str(&minify(&node_data))?;
    //     let token: Option<String> = data.get("token").and_then(|value| Some(value.to_string()));
    //
    //     log::debug!("data: {:#?}",data);
    //     Ok(token.expect("cannot get token"))
    // }

    pub fn with_check_flow_file(mut self, path: &'a str) -> Self {
        self.inner.check_flow_file = path;
        let json = std::fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("Unable to read `{}`: {}", path, err));
        let test_flow: CheckFlows = serde_json::from_str(&minify(&json))
            .unwrap_or_else(|err| panic!("Cannot parse `{}` as JSON: {}", path, err));

        self.inner.check_flows = test_flow;
        self
    }

    pub fn with_base_endpoint_file(mut self, path: &'a str) -> Self {
        self.inner.check_flow_file = path;
        let json = std::fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("Unable to read `{}`: {}", path, err));
        let base_nodes: HashMap<BlockChainType, UrlType> = serde_json::from_str(&minify(&json))
            .unwrap_or_else(|err| panic!("Cannot parse `{}` as JSON: {}", path, err));
        self.inner.base_nodes = base_nodes;
        self
    }

    pub fn with_output_file(mut self, output_file: &'a str) -> Self {
        self.inner.output_file = output_file;
        self
    }

    pub fn build(self) -> CheckComponent<'a> {
        self.inner
    }
}
