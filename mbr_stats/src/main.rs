use clap::{Arg, Command};
use log::info;
use logger::core::init_logger;
use mbr_stats::component_stats::ComponentStats;
use mbr_stats::SIGNER_PHRASE;

#[tokio::main]
async fn main() {
    // Load env file
    dotenv::dotenv().ok();

    info!("Start mbr stats");
    init_logger(&String::from("ComponentStats"));

    let matches = Command::new("mbr-check-component")
        .version("0.1")
        .about("mbr-check-component")
        .subcommand(create_component_stats())
        .get_matches();
    if let Some(ref matches) = matches.subcommand_matches("update-stats") {
        let config_data = matches.value_of("config-data").unwrap_or("");
        let prometheus_url = matches.value_of("prometheus-url").unwrap();
        let mvp_url = matches
            .value_of("mvp-url")
            .unwrap_or("wss://dev.verification.massbit.io");
        let list_project_url = matches
            .value_of("list-project-url")
            .unwrap_or("https://portal.massbitroute.dev/mbr/d-apis/project/list/verify");

        let mut component_stats = ComponentStats::builder()
            .with_config_uri(config_data.to_string())
            .await
            .with_signer_phrase(SIGNER_PHRASE.to_string())
            .with_prometheus_url(prometheus_url.to_string())
            .await
            .with_mvp_url(mvp_url.to_string())
            .await
            .with_list_project_url(list_project_url.to_string())
            .build();
        log::debug!("check_component: {:?}", component_stats);
        let _ = component_stats.run().await;
    }
}

fn create_component_stats() -> Command<'static> {
    Command::new("update-stats")
        .about("get stats from prometheus server and update to chain")
        .arg(
            Arg::new("prometheus-url")
                .short('p')
                .long("prometheus-url")
                .value_name("prometheus-url")
                .help("Input prometheus-url")
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
            Arg::new("list-project-url")
                .long("list-project-url")
                .value_name("list-project-url")
                .help("Input list-project-url")
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
            Arg::new("config-data")
                .short('c')
                .long("config-data")
                .value_name("config-data")
                .help("Input config-data")
                .takes_value(true),
        )
}
