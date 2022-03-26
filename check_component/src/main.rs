use clap::{App, Arg};
use logger::core::init_logger;
use mbr_check_component::check_module::check_module::CheckComponent;
// use regex::Regex;

use lazy_static::lazy_static;
use log::info;
use logger;
use mbr_check_component::config::AccessControl;
use mbr_check_component::server_builder::ServerBuilder;

use std::env;

lazy_static! {
    pub static ref CHECK_COMPONENT_ENDPOINT: String =
        env::var("CHECK_COMPONENT_ENDPOINT").unwrap_or(String::from("0.0.0.0:3030"));
    pub static ref CHECK_INTERVAL_MS: u64 = 3000;
}

// pub fn get_node_url_from_cli() -> String {
//     let yml = load_yaml!("../../src/examples/cli.yml");
//     let matches = App::from_yaml(yml).get_matches();
//
//     let node_ip = matches.value_of("node-server").unwrap_or("ws://127.0.0.1");
//     let node_port = matches.value_of("node-port").unwrap_or("9944");
//     let url = format!("{}:{}", node_ip, node_port);
//     println!("Interacting with node on {}\n", url);
//     url
// }

#[tokio::main]
async fn main() {
    let _res = init_logger(&String::from("CheckComponent"));
    //println!("Log output: {}", res); // Print log output type

    let matches = App::new("mbr-check-component")
        .version("0.1")
        .about("mbr-check-component")
        .subcommand(create_check_component())
        .get_matches();
    if let Some(ref matches) = matches.subcommand_matches("check-kind") {
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
        let _massbit_chain_endpoint = matches
            .value_of("massbit-chain-endpoint")
            .unwrap_or("ws://127.0.0.1:9944");
        let domain = matches.value_of("domain").unwrap_or("massbitroute.dev");
        let _signer_phrase = matches.value_of("signer-phrase").unwrap_or("Bob");
        let output = matches
            .value_of("output")
            .unwrap_or("src/example/output.json");
        let check_component = CheckComponent::builder()
            .with_list_node_id_file(list_node_id_file.to_string(), None)
            .await
            .with_list_gateway_id_file(list_gateway_id_file.to_string(), None)
            .await
            .with_list_dapi_id_file(list_dapi_id_file.to_string())
            .await
            .with_list_user_file(list_user_file.to_string())
            .await
            .with_check_flow_file(check_flow_file.to_string())
            .with_base_endpoint_file(base_endpoint_file.to_string())
            .with_domain(domain.to_string())
            .with_output_file(output.to_string())
            .build();
        log::debug!("check_component: {:?}", check_component);
        let socket_addr = CHECK_COMPONENT_ENDPOINT.as_str();
        let server = ServerBuilder::default()
            .with_entry_point(socket_addr)
            .build(check_component);

        let access_control = AccessControl::default();

        // info!("Run check component");
        // let _ = server
        //     .check_component_service
        //     .run_check(*CHECK_INTERVAL_MS)
        //     .await;

        info!("Run service ");
        server.serve(access_control).await;
    }
}
fn create_check_component() -> App<'static> {
    App::new("check-kind")
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
        .arg(
            Arg::new("massbit-chain-endpoint")
                .short('m')
                .long("massbit-chain-endpoint")
                .value_name("massbit-chain-endpoint")
                .help("Input massbit-chain-endpoint")
                .takes_value(true),
        )
        .arg(
            Arg::new("signer-phrase")
                .short('s')
                .long("signer-phrase")
                .value_name("signer-phrase")
                .help("Input signer-phrase")
                .takes_value(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("output")
                .help("Output file")
                .takes_value(true),
        )
        .arg(
            Arg::new("domain")
                .long("domain")
                .value_name("domain")
                .help("domain name")
                .takes_value(true),
        )
}
