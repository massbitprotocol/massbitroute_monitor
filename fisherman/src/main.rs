use clap::{App, Arg};
use logger::core::init_logger;
use mbr_check_component::check_module::check_module::CheckComponent;
// use regex::Regex;
use handlebars::Handlebars;
use lazy_static::lazy_static;
use log::info;
use logger;
use mbr_check_component::config::AccessControl;
use mbr_check_component::server_builder::ServerBuilder;
use std::collections::BTreeMap;
use std::env;

lazy_static! {
    pub static ref FISHERMAN_ENDPOINT: String =
        env::var("FISHERMAN_ENDPOINT").unwrap_or(String::from("0.0.0.0:4040"));
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
        let server = ServerBuilder::default()
            .with_entry_point(socket_addr)
            .build(check_component);


        let task = tokio::spawn(async move {
            info!("Run check component");
            let _ = server.check_component_service.run_check().await;
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
