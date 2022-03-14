use clap::{App, Arg};
use logger::core::init_logger;
use mbr_check_component::check_module::check_module::{CheckComponent, CheckMkReport, ComponentInfo};
// use regex::Regex;
use handlebars::Handlebars;
use lazy_static::lazy_static;
use log::info;
use logger;
use mbr_check_component::config::AccessControl;
use mbr_check_component::server_builder::ServerBuilder;
use std::collections::{BTreeMap, HashMap};
use std::collections::hash_map::Entry;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize,Serialize};


lazy_static! {
    pub static ref FISHERMAN_ENDPOINT: String =
        env::var("FISHERMAN_ENDPOINT").unwrap_or(String::from("0.0.0.0:4040"));
    pub static ref NUMBER_OF_SAMPLES: u64 = 5;
    pub static ref SAMPLE_INTERVAL_MS: u64 = 1000;
    //Fixme: use better solution to get response time
    pub static ref RESPONSE_TIME_KEY_NAME: String = "checkCall_response_time_ms".to_string();
}

#[derive(Clone, Debug, Default)]
pub struct FishermanService {
    number_of_sample: u64,
    sample_interval_ms: u64,
    entry_point: String,
    check_component_service: Arc<CheckComponent>,
}

#[derive(Clone, Debug, Default)]
pub struct ComponentReport {
    check_number: u64,
    success_number: u64,
    response_time_ms: Option<u64>,
}

impl From<&Vec<CheckMkReport>> for ComponentReport{
    fn from(reports: &Vec<CheckMkReport>) -> Self {
        let check_number = *NUMBER_OF_SAMPLES;
        let mut success_number = 0;
        let response_times = reports.iter().filter_map(|report| {
            if report.is_component_status_ok() {
                success_number += 1;
            }
            if let Some(response_time) = report.metric.metric.get(&*RESPONSE_TIME_KEY_NAME) {
                Some(serde_json::from_value(response_time.clone()).unwrap())
            }else {
                None
            }
        }).collect::<Vec<u64>>();
        let number_of_call = response_times.len() as u64;
        let response_time_ms = match number_of_call==0 {
            true => {None}
            false => {Some(response_times.into_iter().sum::<u64>()/number_of_call)}
        };

        ComponentReport{
            check_number,
            success_number,
            response_time_ms
        }

    }
}

#[tokio::main]
async fn main() {
    let res = init_logger(&String::from("Fisherman"));
    //println!("Log output: {}", res); // Print log output type

    let matches = App::new("mbr-fisherman")
        .version("0.1")
        .about("mbr-fisherman")
        .subcommand(create_run_fisherman())
        .get_matches();
    if let Some(ref matches) = matches.subcommand_matches("run-fisherman") {
        let list_node_id_file = matches
            .value_of("list-node-id-file")
            .unwrap_or("https://dapi.massbit.io/deploy/info/node/listid");
        let list_gateway_id_file = matches
            .value_of("list-gateway-id-file")
            .unwrap_or("https://dapi.massbit.io/deploy/info/gateway/listid");
        let list_dapi_id_file = matches
            .value_of("list-dapi-id-file")
            .unwrap_or("https://dapi.massbit.io/deploy/info/dapi/listid");
        let list_user_file = matches
            .value_of("list-user-file")
            .unwrap_or("https://dapi.massbit.io/deploy/info/user/listid");
        let check_flow_file = matches
            .value_of("check-flow")
            .unwrap_or("src/example/check-flow.json");
        let base_endpoint_file = matches
            .value_of("base-endpoint")
            .unwrap_or("src/example/base-endpoint.json");

        let check_component = CheckComponent::builder()
            .with_list_node_id_file(list_node_id_file.to_string())
            .await
            .with_list_gateway_id_file(list_gateway_id_file.to_string())
            .await
            .with_list_dapi_id_file(list_dapi_id_file.to_string())
            .await
            .with_list_user_file(list_user_file.to_string())
            .await
            .with_check_flow_file(check_flow_file.to_string())
            .with_base_endpoint_file(base_endpoint_file.to_string())
            .build();
        log::debug!("check_component: {:?}", check_component);
        let socket_addr = FISHERMAN_ENDPOINT.as_str();
        let service = FishermanService {
            number_of_sample: *NUMBER_OF_SAMPLES,
            sample_interval_ms: *SAMPLE_INTERVAL_MS,
            entry_point: socket_addr.to_string(),
            check_component_service: Arc::new(check_component)
        };


        let task = tokio::spawn(async move {
            info!("Run check component");
            let mut average_reports: HashMap<ComponentInfo,ComponentReport> = HashMap::new();
            let mut collect_reports : HashMap<ComponentInfo,Vec<CheckMkReport>> = HashMap::new();
            for n in 0..service.number_of_sample {
                info!("Run {} times",n+1);
                if let Ok(reports) = service.check_component_service.check_components().await {
                    for (component,report) in reports {
                        // with each component collect reports in to vector
                        match collect_reports.entry(component) {
                            Entry::Occupied(o) => {
                                o.into_mut().push(report);
                            },
                            Entry::Vacant(v) => {
                                v.insert(vec![report]);
                            }
                        }
                    }
                };

                tokio::time::sleep(Duration::from_millis(service.sample_interval_ms));

            }

            info!("collect_reports: {:#?}",collect_reports);
            for (component,report) in collect_reports.iter(){
                info!("component:{:?}", component.id);
                average_reports.insert(component.clone(), ComponentReport::from(report));
            };



            info!("average_reports: {:#?}",average_reports);

        });

        task.await;
        // info!("Run service");
        // let access_control = AccessControl::default();
        // server.serve(access_control).await;
    }
}
fn create_run_fisherman() -> App<'static> {
    App::new("run-fisherman")
        .about("check node kind is correct")
        .arg(
            Arg::new("list-node-id-file")
                .short('n')
                .long("list-node-id-file")
                .value_name("list-node-id-file")
                .help("Input list-node-id file")
                .takes_value(true),
        )
        .arg(
            Arg::new("list-gateway-id-file")
                .short('g')
                .long("list-gateway-id-file")
                .value_name("list-gateway-id-file")
                .help("Input list-gateway-id file")
                .takes_value(true),
        )
        .arg(
            Arg::new("list-dapi-id-file")
                .short('d')
                .long("list-dapi-id-file")
                .value_name("list-dapi-id-file")
                .help("Input list-dapi-id file")
                .takes_value(true),
        )
        .arg(
            Arg::new("list-user-file")
                .short('u')
                .long("list-user-file")
                .value_name("list-user-file")
                .help("Input list-user file")
                .takes_value(true),
        )
        .arg(
            Arg::new("check-flow")
                .short('c')
                .long("check-flow")
                .value_name("check-flow")
                .help("Input check-flow file")
                .takes_value(true),
        )
        .arg(
            Arg::new("base-endpoint")
                .short('b')
                .long("base-endpoint")
                .value_name("base-endpoint")
                .help("Input base-endpoint file")
                .takes_value(true),
        )

}
