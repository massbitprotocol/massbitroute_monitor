use crate::check_module::check_module::{CheckComponent, ComponentInfo};
use crate::server_config::AccessControl;
use std::collections::VecDeque;

use log::{debug, info};
use serde::{Deserialize, Serialize};

use serde_json::Value;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use warp::http::{HeaderMap, Method};

use warp::reply::Json;
use warp::{http::StatusCode, Filter, Rejection, Reply};

pub const MAX_JSON_BODY_SIZE: u64 = 1024 * 1024;

#[derive(Default)]
pub struct ServerBuilder {
    entry_point: String,
}

pub struct CheckComponentServer {
    entry_point: String,
    pub check_component_service: Arc<CheckComponent>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeployParam {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimpleResponse {
    success: bool,
}

impl CheckComponentServer {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }
    pub async fn serve(&self, access_control: AccessControl, sender: Sender<ComponentInfo>) {
        let allow_headers: Vec<String> = access_control.get_access_control_allow_headers();
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
            .create_get_status(self.check_component_service.clone(), sender)
            .await
            .with(&cors)
            .or(self.create_ping().with(&cors))
            .recover(handle_rejection);
        let socket_addr: SocketAddr = self.entry_point.parse().unwrap();

        warp::serve(router).run(socket_addr).await;
    }
    /// Get status of component
    async fn create_get_status(
        &self,
        service: Arc<CheckComponent>,
        sender: Sender<ComponentInfo>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let sender_clone = sender.clone();
        warp::path!("get_status")
            .and(CheckComponentServer::log_headers())
            .and(warp::post())
            .and(warp::body::content_length_limit(MAX_JSON_BODY_SIZE).and(warp::body::json()))
            .and_then(move |body: Value| {
                info!("#### Received request body ####");
                info!("{}", body);
                let component_info: ComponentInfo = serde_json::from_value(body).unwrap();
                let sender_another_clone = sender_clone.clone();
                async move {
                    let res = sender_another_clone.send(component_info).await;
                    Self::simple_response(true).await
                }
            })
    }

    /// Ping API
    fn create_ping(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("ping")
            .and(warp::get())
            .and_then(move || async move {
                info!("Receive ping request");
                Self::simple_response(true).await
            })
    }
    pub(crate) async fn simple_response(success: bool) -> Result<impl Reply, Rejection> {
        let res = SimpleResponse { success };
        Ok(warp::reply::json(&res))
    }

    fn log_headers() -> impl Filter<Extract = (), Error = Infallible> + Copy {
        warp::header::headers_cloned()
            .map(|headers: HeaderMap| {
                debug!("#### Received request header ####");
                for (k, v) in headers.iter() {
                    // Error from `to_str` should be handled properly
                    debug!(
                        "{}: {}",
                        k,
                        v.to_str().expect("Failed to print header value")
                    )
                }
            })
            .untuple_one()
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
