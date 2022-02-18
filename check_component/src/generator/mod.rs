use chrono::Duration;
use clap::StructOpt;
use minifier::json::minify;
use serde::{Deserialize, Serialize};
use serde_json::{to_string, Number, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ptr::hash;
use std::thread;
use timer::Timer;

// use futures_util::future::try_future::TryFutureExt;

type BlockChainType = String;
type UrlType = String;
type TaskType = String;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct NodeInfo {
    pub blockchain: BlockChainType,
    pub network: String,
    pub gateway_id: String,
    pub user_id: String,
    pub ip: String,
    pub zone: String,
    pub country_code: String,
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckComponent<'a> {
    // input file
    pub list_id_file: &'a str,
    pub check_flow_file: &'a str,
    pub base_endpoint_file: &'a str,
    // The output file
    pub output_file: &'a str,
    // inner data
    pub list_nodes: Vec<NodeInfo>,
    pub base_nodes: HashMap<BlockChainType, UrlType>,
    pub check_flows: CheckFlows,
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
    Node(NodeInfo),
    Gateway(GatewayInfo),
    DApi(DApiInfo),
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
    metric: CheckMkMetric,
    status_detail: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
struct CheckMkMetric {
    name: String,
    values: Vec<String>,
}

impl ToString for CheckMkReport {
    fn to_string(&self) -> String {
        let values: String = self.metric.values.join(";");
        format!(
            r#"{status} "{service_name}" {value_name}={values} {status_detail}"#,
            status = &self.status,
            service_name = &self.service_name,
            value_name = self.metric.name,
            values = values,
            status_detail = self.status_detail
        )
    }
}

impl<'a> CheckComponent<'a> {
    pub fn builder() -> GeneratorBuilder<'a> {
        GeneratorBuilder::default()
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
                    //println!("check_flow.blockchain: {},blockchain: {}\ncheck_flow.component:{},component:{}",&check_flow.blockchain,&blockchain,check_flow.component,component);
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

    fn run_check_steps(
        &self,
        steps: &Vec<CheckStep>,
        node: &NodeInfo,
    ) -> Result<CheckMkReport, anyhow::Error> {
        // Todo: add code
        for step in steps {
            println!("step: {:?}", step);
        }

        Ok(CheckMkReport::default())
    }

    pub fn check_components(&self) -> Result<Vec<CheckMkReport>, anyhow::Error> {
        println!("Check component");
        // Call node
        //http://cf242b49-907f-49ce-8621-4b7655be6bb8.node.mbr.massbitroute.com
        //header 'x-api-key: vnihqf14qk5km71aatvfr7c3djiej9l6mppd5k20uhs62p0b1cm79bfkmcubal9ug44e8cu2c74m29jpusokv6ft6r01o5bnv5v4gb8='
        let mut reports: Vec<CheckMkReport> = Vec::new();

        // Check node
        let nodes = &self.list_nodes;
        // Check each node
        for node in nodes {
            let steps = self.get_check_steps(
                &node.blockchain,
                &"node".to_string(),
                &"checking_chain_type".to_string(),
            );

            let report = match steps {
                Ok(steps) => {
                    //println!("steps:{:#?}", steps);
                    println!("Do the check steps");
                    self.run_check_steps(&steps, node)
                }
                Err(err) => {
                    println!("There are no check steps");
                    Err(anyhow::Error::msg("There are no check steps"))
                }
            };

            if let Ok(report) = report {
                println!("add report: {:?}", &report);
                reports.push(report);
            }
        }

        Ok(vec![CheckMkReport::default(), CheckMkReport::default()])
    }
    pub fn run_check(&self) -> Result<(), anyhow::Error> {
        // Begin run check
        println!("run_check");
        // Infinity run check loop
        loop {
            let reports = self.check_components();

            // Write reports to file
            match reports {
                Ok(reports) => {
                    let report_content: String = reports
                        .iter()
                        .map(|report| report.to_string())
                        .collect::<Vec<String>>()
                        .join("\n");
                    std::fs::write(self.output_file, report_content).expect("Unable to write file");
                }
                Err(err) => (),
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
                list_id_file: "",
                check_flow_file: "",
                base_endpoint_file: "",
                output_file: "",
                list_nodes: vec![],
                base_nodes: Default::default(),
                check_flows: Default::default(),
            },
        }
    }
}

impl<'a> GeneratorBuilder<'a> {
    pub async fn with_list_id_file(mut self, path: &'a str) -> GeneratorBuilder<'a> {
        self.inner.list_id_file = path;
        let list_id_file = std::fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("Unable to read `{}`: {}", path, err));

        let list_nodes: Vec<NodeInfo> = self
            .get_list_nodes()
            .await
            .unwrap_or_else(|err| panic!("Cannot parse `{}` as JSON: {}", path, err));
        self.inner.list_nodes = list_nodes;
        self
    }

    async fn get_list_nodes(&self) -> Result<Vec<NodeInfo>, anyhow::Error> {
        let mut nodes: Vec<NodeInfo> = Vec::new();
        let prefix_url = "https://dapi.massbit.io/deploy/gateway".to_string();
        println!("get_list_nodes");

        println!("----------Create list of nodes info details----------");
        let file = File::open(self.inner.list_id_file)?;
        let reader = BufReader::new(file);
        for line in reader.lines().into_iter() {
            let data: Vec<String> = line
                .unwrap()
                .split(' ')
                .map(|piece| piece.to_string())
                .collect();
            let node = NodeInfo {
                blockchain: data[2].clone(),
                network: data[3].clone(),
                gateway_id: data[0].clone(),
                user_id: data[1].clone(),
                ip: data[4].clone(),
                zone: data[5].clone(),
                country_code: data[6].clone(),
                token: data[7].clone(),
            };
            println!("node: {:#?}", &node);
            nodes.push(node);

            //d39ba1ce-d1fd-4a85-b25c-0ec0f4f0517c 7dd6caf2-7dc1-45ee-8fe3-744259fabf81 eth mainnet 34.88.228.190 EU FI
            //https://dapi.massbit.io/deploy/gateway/eth/mainnet/EU/FI/7dd6caf2-7dc1-45ee-8fe3-744259fabf81/d39ba1ce-d1fd-4a85-b25c-0ec0f4f0517c
            // let url = format!("{}/{}/{}/{}/{}/{}/{}",prefix_url,node.blockchain,node.network,node.zone,node.country_code,node.user_id,node.gateway_id);
            // println!("url:{}",&url);
            // let node_data = reqwest::get(url.as_str())
            //     .await?
            //     .text()
            //     .await?;
            // println!("node_data: {:#?}", &node_data);
            // let token = Self::get_token(node_data)?;
            // node.token = token;
        }

        return Ok(nodes);
    }

    // pub fn get_token(node_data: String) -> Result<String,anyhow::Error> {
    //     let data: Value = serde_json::from_str(&minify(&node_data))?;
    //     let token: Option<String> = data.get("token").and_then(|value| Some(value.to_string()));
    //
    //     println!("data: {:#?}",data);
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
