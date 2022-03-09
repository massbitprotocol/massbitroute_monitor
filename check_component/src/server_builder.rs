use crate::check_module::check_module::CheckComponent;
use crate::config::AccessControl;
use futures::lock::Mutex;
use log::info;
use serde::{Deserialize, Serialize};
use slog::Logger;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use warp::http::Method;
use warp::{http::StatusCode, multipart::FormData, Filter, Rejection, Reply};

#[derive(Default)]
pub struct ServerBuilder {
    entry_point: String,
    logger: Option<Logger>,
}
pub struct CheckComponentServer<'a> {
    entry_point: String,
    indexer_service: Arc<CheckComponent<'a>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeployParam {
    pub id: String,
}

impl<'a> CheckComponentServer<'a> {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }
    pub async fn serve(&self, access_control: AccessControl) {
        let mut allow_headers: Vec<String> = access_control.get_access_control_allow_headers();
        info!("allow_headers: {:?}", allow_headers);
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(allow_headers)
            .allow_methods(&[
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
                Method::HEAD,
            ]);

        let router = self
            .create_route_indexer_cli_deploy(self.indexer_service.clone())
            .with(&cors)
            .recover(handle_rejection);
        let socket_addr: SocketAddr = self.entry_point.parse().unwrap();

        warp::serve(router).run(socket_addr).await;
    }
    /// Indexer deploy from cli api
    fn create_route_indexer_cli_deploy(
        &self,
        service: Arc<CheckComponent<'a>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'a {
        warp::path!("get_status")
            .and(warp::post())
            .and(warp::multipart::form())
            .and_then(move |form: FormData| {
                let clone_service = service.clone();
                async move { clone_service.get_components_status(form) }
            })
    }
}
impl ServerBuilder {
    pub fn with_entry_point(mut self, entry_point: &str) -> Self {
        self.entry_point = String::from(entry_point);
        self
    }
    pub fn with_logger(mut self, logger: Logger) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn build(&self) -> CheckComponentServer {
        CheckComponentServer {
            entry_point: self.entry_point.clone(),
            indexer_service: Arc::new(CheckComponent {
                list_node_id_file: "",
                list_gateway_id_file: "",
                list_dapi_id_file: "",
                list_user_file: "",
                check_flow_file: "",
                base_endpoint_file: "",
                output_file: "",
                list_nodes: vec![],
                list_gateways: vec![],
                list_dapis: vec![],
                list_users: vec![],
                base_nodes: Default::default(),
                check_flows: Default::default(),
                is_loop_check: false,
                is_write_to_file: false,
            }),
        }
    }
}

async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::BAD_REQUEST,
            format!("Authorization error, {:?}", err),
        )
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}
