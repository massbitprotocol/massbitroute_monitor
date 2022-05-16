use anyhow::Error;
use clap::{Arg, Command};
use dotenv;
use futures_util::future::err;
use log::{debug, info};
use logger;
use logger::core::init_logger;
use mbr_check_component::check_module::check_module::{CheckComponent, ComponentInfo};
use mbr_check_component::SIGNER_PHRASE;
use mbr_fisherman::data_check::CheckDataCorrectness;
use mbr_fisherman::fisherman_service::{
    FishermanService, ProviderReportReason, SubmitProviderReport,
};
use mbr_fisherman::ping_pong::CheckPingPong;
use mbr_fisherman::FISHERMAN_ENDPOINT;
use mbr_fisherman::{CONFIG, ZONE};
use mbr_stats::chain_adapter::Projects;
use std::convert::TryInto;
use std::fs::read;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    // Load env file
    dotenv::dotenv().ok();

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
        let domain = matches.value_of("domain").unwrap_or("massbitroute.dev");
        let no_report_mode = matches.is_present("no-report-mode");
        let mut check_component = CheckComponent::builder()
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

        let mut fisherman_service_org = FishermanService::builder()
            .with_number_of_sample(CONFIG.number_of_samples)
            .with_sample_interval_ms(CONFIG.sample_interval_ms)
            .with_entry_point(socket_addr.to_string())
            .with_check_component_service(check_component)
            .with_signer_phrase(SIGNER_PHRASE.to_string())
            .with_mvp_url(mvp_url.to_string())
            .await
            .with_no_report(no_report_mode)
            .build();

        let list_providers_org = Arc::new(RwLock::new(
            fisherman_service_org.get_provider_list_from_portal().await,
        ));
        // Check component
        //fisherman_service.loop_check_component().await;
        let mut fisherman_service = fisherman_service_org.clone();
        let list_providers = list_providers_org.clone();

        // test ping pong
        task::spawn(async move {
            loop {
                let list_providers_clone = list_providers.clone();
                // Get list bad component
                let bad_components = fisherman_service
                    .check_ping_pong(
                        list_providers_clone,
                        fisherman_service.check_component_service.domain.clone(),
                    )
                    .await;

                match bad_components {
                    Ok(bad_components) => {
                        // Submit_reports bad components, get success report list
                        let bad_components =
                            fisherman_service.submit_reports(&bad_components).await;
                        // Remove from list
                        {
                            let mut list_providers_lock = list_providers.write().await;
                            list_providers_lock
                                .retain(|component| !bad_components.contains(component));
                        }
                    }
                    Err(err) => {
                        info!("check_ping_pong error: {}", err);
                    }
                }
                info!(
                    "check ping pong list_providers: {:?}",
                    list_providers.read().await
                );
                sleep(Duration::from_secs(CONFIG.check_ping_pong_interval)).await;
            }
        });

        let mut fisherman_service = fisherman_service_org.clone();
        let list_providers = list_providers_org.clone();
        // Check data correctness
        task::spawn(async move {
            loop {
                let list_providers_clone = list_providers.clone();
                // Get list bad component
                let bad_components = fisherman_service.check_data(list_providers_clone).await;

                match bad_components {
                    Ok(bad_components) => {
                        // Submit_reports bad components, get success report list
                        let bad_components =
                            fisherman_service.submit_reports(&bad_components).await;
                        // Remove from list
                        {
                            let mut list_providers_lock = list_providers.write().await;
                            list_providers_lock
                                .retain(|component| !bad_components.contains(component));
                        }
                    }
                    Err(err) => {
                        info!("check_ping_pong error: {}", err);
                    }
                }
                info!(
                    "check data correctness list_providers: {:?}",
                    list_providers.read().await
                );
                sleep(Duration::from_secs(CONFIG.check_logic_interval)).await;
            }
        });

        // let mut fisherman_service = fisherman_service_org.clone();
        // let list_providers = list_providers_org.clone();
        // // test benchmark
        // task::spawn(async move {
        //     loop {
        //         let list_providers_clone = list_providers.clone();
        //         let res = fisherman_service
        //             .check_benchmark(list_providers_clone)
        //             .await;
        //
        //         sleep(Duration::from_secs(CONFIG.check_benchmark_interval)).await;
        //     }
        // });

        // Update node/gw list
        loop {
            sleep(Duration::from_secs(CONFIG.update_provider_list_interval)).await;
            let new_list_providers = fisherman_service_org.get_provider_list_from_portal().await;
            {
                let mut list_providers_lock = list_providers_org.write().await;
                *list_providers_lock = new_list_providers;
            }
            info!(
                "Update list provider: {:?}",
                list_providers_org.read().await
            );
        }
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
