use clap::{App, Arg};
//use logger::core::init_logger;
use mbr_check_component::check_module::check_module::CheckComponent;

#[tokio::main]
async fn main() {
    let matches = App::new("mbr-check-component")
        .version("0.1")
        .about("mbr-check-component")
        .subcommand(create_check_component())
        .get_matches();
    if let Some(ref matches) = matches.subcommand_matches("check-kind") {
        let list_node_id_file = matches
            .value_of("list-node-id-file")
            .unwrap_or("src/example/list-node-id");
        let list_gateway_id_file = matches
            .value_of("list-gateway-id-file")
            .unwrap_or("src/example/list-gateway-id");
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
            .with_list_id_file(list_node_id_file)
            .await
            .with_check_flow_file(check_flow_file)
            .with_base_endpoint_file(base_endpoint_file)
            .with_output_file(output)
            .build();
        println!("check_component: {:?}", check_component);
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
