use crate::check_module::check_module::{CheckComponent, ComponentInfo};
use crate::config::AccessControl;
use futures::lock::Mutex;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use slog::Logger;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use warp::http::Method;
use warp::{http::StatusCode, multipart::FormData, Filter, Rejection, Reply};

pub const MAX_JSON_BODY_SIZE: u64 = 1024 * 1024;

#[derive(Default)]
pub struct ServerBuilder {
    entry_point: String,
    logger: Option<Logger>,
}
pub struct CheckComponentServer {
    entry_point: String,
    pub check_component_service: Arc<CheckComponent>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeployParam {
    pub id: String,
}

impl CheckComponentServer {
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
        info!("cors: {:?}", cors);
        let router = self
            .create_get_status(self.check_component_service.clone())
            .with(&cors)
            .recover(handle_rejection);
        let socket_addr: SocketAddr = self.entry_point.parse().unwrap();

        warp::serve(router).run(socket_addr).await;
    }
    /// Indexer deploy from cli api
    fn create_get_status(
        &self,
        service: Arc<CheckComponent>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("get_status")
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |component_info: ComponentInfo| {
                let clone_service = service.clone();
                async move { clone_service.get_components_status(component_info).await }
            })
    }
}
impl ServerBuilder {
    pub fn with_entry_point(mut self, entry_point: &str) -> Self {
        self.entry_point = String::from(entry_point);
        self
    }

    pub fn build(&self, check_component: CheckComponent) -> CheckComponentServer {
        CheckComponentServer {
            entry_point: self.entry_point.clone(),
            check_component_service: Arc::new(check_component),
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
