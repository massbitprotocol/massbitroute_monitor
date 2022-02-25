use clap::{App, Arg};
use logger::core::init_logger;
use mbr_check_component::check_module::check_module::CheckComponent;
// use regex::Regex;
use handlebars::Handlebars;
use std::collections::BTreeMap;

#[tokio::main]
async fn main() {
    // let mut handlebars = Handlebars::new();
    // let source = r#"{ "jsonrpc": "2.0",  "method": "eth_getBlockByNumber", "params": ["{{base_return:block_number}}",  true],"id": 1}"#;
    //
    // let mut data = BTreeMap::new();
    // data.insert(
    //     "base_return:block_number".to_string(),
    //     "\"0xd9b51c\"".to_string(),
    // );
    // let res = handlebars.render_template(source, &data);
    // println!("res: {:?}", res);

    let res = init_logger(&String::from("CheckComponent"));
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
        let output = matches
            .value_of("output")
            .unwrap_or("src/example/output.json");
        let check_component = CheckComponent::builder()
            .with_list_node_id_file(list_node_id_file)
            .await
            .with_list_gateway_id_file(list_gateway_id_file)
            .await
            .with_list_dapi_id_file(list_dapi_id_file)
            .await
            .with_list_user_file(list_user_file)
            .await
            .with_check_flow_file(check_flow_file)
            .with_base_endpoint_file(base_endpoint_file)
            .with_output_file(output)
            .build();
        log::debug!("check_component: {:?}", check_component);
        let _ = check_component.run_check().await;
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
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("output")
                .help("Output file")
                .takes_value(true),
        )
}
