use clap::{Arg, Command};
use handlebars::Handlebars;
use logger::core::init_logger;
use mbr_stats::component_stats::ComponentStats;
use std::collections::BTreeMap;

#[tokio::main]
async fn main() {
    println!("Start mbr stats");
    let res = init_logger(&String::from("ComponentStats"));

    let matches = Command::new("mbr-check-component")
        .version("0.1")
        .about("mbr-check-component")
        .subcommand(create_component_stats())
        .get_matches();
    if let Some(ref matches) = matches.subcommand_matches("update-stats") {
        let config_data = matches.value_of("config-data").unwrap_or("");
        let prometheus_gateway_url = matches
            .value_of("prometheus-gateway-url")
            .unwrap_or("https://stat.mbr.massbitroute.com/__internal_prometheus_gw");
        let prometheus_node_url = matches
            .value_of("prometheus-node-url")
            .unwrap_or("https://stat.mbr.massbitroute.com/__internal_prometheus_node");
        let mvp_url = matches
            .value_of("mvp-url")
            .unwrap_or("wss://rpc.polkadot.io");

        let component_stats = ComponentStats::builder()
            .with_config_uri(config_data)
            .await
            .with_prometheus_gateway_url(prometheus_gateway_url)
            .await
            .with_prometheus_node_url(prometheus_node_url)
            .await
            .with_mvp_url(mvp_url)
            .await
            .build();
        log::debug!("check_component: {:?}", component_stats);
        let _ = component_stats.run().await;
    }
}

fn create_component_stats() -> Command<'static> {
    Command::new("update-stats")
        .about("get stats from prometheus server and update to chain")
        .arg(
            Arg::new("prometheus-gateway-url")
                .short('g')
                .long("prometheus-gateway-url")
                .value_name("prometheus-gateway-url")
                .help("Input prometheus-gateway-url")
                .takes_value(true),
        )
        .arg(
            Arg::new("prometheus-node-url")
                .short('n')
                .long("prometheus-node-url")
                .value_name("prometheus-node-url")
                .help("Input prometheus-node-url")
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
            Arg::new("config-data")
                .short('c')
                .long("config-data")
                .value_name("config-data")
                .help("Input config-data")
                .takes_value(true),
        )
}
