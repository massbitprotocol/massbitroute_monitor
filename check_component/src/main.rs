use anyhow::Error;
use clap::{App, Arg};
use futures_util::future::join;
use logger::core::init_logger;
use mbr_check_component::check_module::check_module::{
    CheckComponent, CheckMkReport, ComponentInfo,
};
use std::sync::Arc;
use std::thread;

use log::{info, warn};
use logger;
use mbr_check_component::check_module::store_report::{ReporterRole, SendPurpose, StoreReport};
use mbr_check_component::server_builder::ServerBuilder;
use mbr_check_component::server_config::AccessControl;
use mbr_check_component::{CHECK_COMPONENT_ENDPOINT, LOCAL_IP, PORTAL_AUTHORIZATION};
use reqwest::Response;
use slog::debug;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use wrap_wrk::WrkReport;

#[tokio::main]
async fn main() {
    // Load env file
    dotenv::dotenv().ok();

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
        let base_endpoint_file = matches.value_of("base-endpoint").unwrap_or_default();
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
        // Create job queue
        let (sender, mut receiver): (Sender<ComponentInfo>, Receiver<ComponentInfo>) =
            channel(1024);

        let socket_addr = CHECK_COMPONENT_ENDPOINT.as_str();

        let server = ServerBuilder::default()
            .with_entry_point(socket_addr)
            .build(check_component);

        let check_component = server.check_component_service.clone();

        // Run thread verify
        let task_job = tokio::spawn(async move {
            info!("spawn thread!");
            // Process each socket concurrently.
            loop {
                let component = receiver.recv().await;

                if let Some(component) = component {
                    info!("Verify component:{:?}", component);
                    let res = check_component.get_report_component(&component).await;
                    match res {
                        Ok((check_mk_report, wrk_report)) => {
                            // Send to store
                            let mut store_report = StoreReport::build(
                                &*LOCAL_IP,
                                ReporterRole::Verification,
                                &*PORTAL_AUTHORIZATION,
                                &check_component.domain,
                            );

                            store_report.set_report_data(&wrk_report, &check_mk_report, &component);
                            // Send report to verify
                            let res = store_report.send_data(SendPurpose::Verify).await;
                            match res {
                                Ok(res) => {
                                    info!("Send verify res: {:?}", res.text().await);
                                }
                                Err(err) => {
                                    info!("Send verify error: {}", err);
                                }
                            }
                        }
                        Err(err) => {}
                    }
                } else {
                    info!("RError:{:?}", component);
                }
            }
            warn!("Job queue is dead!");
        });

        let access_control = AccessControl::default();

        // info!("Run check component");
        // let _ = server
        //     .check_component_service
        //     .run_check(*CHECK_INTERVAL_MS)
        //     .await;

        info!("Run service ");
        let task_serve = server.serve(access_control, sender);
        join(task_job, task_serve).await;
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
