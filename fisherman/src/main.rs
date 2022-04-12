use clap::{Arg, Command};
use logger;
use logger::core::init_logger;
use mbr_check_component::check_module::check_module::CheckComponent;
use mbr_fisherman::fisherman_service::FishermanService;
use mbr_fisherman::{FISHERMAN_ENDPOINT, NUMBER_OF_SAMPLES, SAMPLE_INTERVAL_MS};

#[tokio::main]
async fn main() {
    let res = init_logger(&String::from("Fisherman"));
    println!("Log output: {}", res); // Print log output type

    let matches = Command::new("mbr-fisherman")
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
        let mvp_url = matches
            .value_of("mvp-url")
            .unwrap_or("wss://dev.verification.massbit.io");
        let signer_phrase = matches
            .value_of("signer-phrase")
            .unwrap_or("bottom drive obey lake curtain smoke basket hold race lonely fit walk"); //Alice
        let domain = matches.value_of("domain").unwrap_or("massbitroute.dev");
        let no_report_mode = matches.is_present("no-report-mode");

        let check_component = CheckComponent::builder()
            .with_list_node_id_file(list_node_id_file.to_string(), Some("staked".to_string()))
            .await
            .with_list_gateway_id_file(list_gateway_id_file.to_string(), Some("staked".to_string()))
            .await
            .with_list_dapi_id_file(list_dapi_id_file.to_string())
            .await
            .with_list_user_file(list_user_file.to_string())
            .await
            .with_domain(domain.to_string())
            .with_check_flow_file(check_flow_file.to_string())
            .with_base_endpoint_file(base_endpoint_file.to_string())
            .build();
        log::debug!("check_component: {:?}", check_component);
        let socket_addr = FISHERMAN_ENDPOINT.as_str();
        // let fisherman_service = FishermanService {
        //     number_of_sample: *NUMBER_OF_SAMPLES,
        //     sample_interval_ms: *SAMPLE_INTERVAL_MS,
        //     entry_point: socket_addr.to_string(),
        //     check_component_service: Arc::new(check_component),
        //     mvp_url: "".to_string(),
        //     signer_phrase: "".to_string(),
        //     chain_adapter: Arc::new(Default::default())
        // };

        let fisherman_service = FishermanService::builder()
            .with_number_of_sample(NUMBER_OF_SAMPLES)
            .with_sample_interval_ms(SAMPLE_INTERVAL_MS)
            .with_entry_point(socket_addr.to_string())
            .with_check_component_service(check_component)
            .with_signer_phrase(signer_phrase.to_string())
            .with_mvp_url(mvp_url.to_string())
            .await
            .with_no_report(no_report_mode)
            .build();

        // Check component
        fisherman_service.loop_check_component().await;

        // info!("Run service");
        // let access_control = AccessControl::default();
        // server.serve(access_control).await;
    }
}
fn create_run_fisherman() -> Command<'static> {
    Command::new("run-fisherman")
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
            Arg::new("signer-phrase")
                .long("signer-phrase")
                .value_name("signer-phrase")
                .help("Input signer-phrase")
                .takes_value(true),
        )
        .arg(
            Arg::new("mvp-url")
                .short('m')
                .long("mvp-url")
                .value_name("mvp-url")
                .help("Input mvp-url")
                .takes_value(true),
        )
        .arg(
            Arg::new("domain")
                .long("domain")
                .value_name("domain")
                .help("domain name")
                .takes_value(true),
        )
        .arg(
            Arg::new("no-report-mode")
                .long("no-report-mode")
                .value_name("no-report-mode")
                .help("enable no-report-mode")
                .takes_value(false),
        )
}
