use std::collections::hash_map::Entry;
use std::fmt::Formatter;
// Massbit chain
use codec::Decode;
use jsonrpsee::core::client::{
    Client as JsonRpcClient,
};

use log::info;
use serde::{Deserialize, Serialize};
use sp_core::crypto::Pair as _;
use sp_core::sr25519::Pair;
use sp_core::Bytes;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::mpsc::channel;
use std::sync::Arc;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{compose_extrinsic, AccountId, Api, UncheckedExtrinsicV4, XtStatus};
use tokio::sync::RwLock;

pub const MVP_EXTRINSIC_DAPI: &str = "Dapi";
const MVP_EXTRINSIC_SUBMIT_PROJECT_USAGE: &str = "submit_project_usage";
const MVP_EVENT_PROJECT_REGISTERED: &str = "ProjectRegistered";

type ProjectIdString = String;
type Quota = String;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Projects(pub HashMap<ProjectIdString, Project>);

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    blockchain: String,
    network: String,
    quota: Quota,
    pub status: String,
}

#[derive(Default)]
pub struct ChainAdapter {
    pub json_rpc_client: Option<JsonRpcClient>,
    pub ws_rpc_client: Option<WsRpcClient>,
    pub api: Option<Api<Pair, WsRpcClient>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct SubmitData {
    consumer_id: String,
    requests_count: isize,
    from_block_number: isize,
    to_block_number: isize,
}

#[allow(dead_code)]
#[derive(Decode, Debug)]
struct ProjectRegisteredEventArgs {
    project_id: Bytes,
    account_id: AccountId,
    chain_id: Bytes,
    quota: u64,
}

impl ProjectRegisteredEventArgs {
    fn project_id_to_string(&self) -> String {
        String::from_utf8_lossy(&*self.project_id).to_string()
    }
    fn get_blockchain_and_network(&self) -> (String, String) {
        let chain_id = String::from_utf8_lossy(&*self.chain_id).to_string();
        let chain_id = chain_id.split(".").collect::<Vec<&str>>();
        (chain_id[0].to_string(), chain_id[1].to_string())
    }
}

impl ChainAdapter {
    pub fn submit_project_usage(
        &self,
        project_id: &String,
        usage: u128,
    ) -> Result<(), anyhow::Error> {
        // set the recipient
        let api = self.api.as_ref().unwrap().clone();
        let id: [u8; 36] = project_id.as_bytes().try_into()?;
        // the names are given as strings
        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
            api,
            MVP_EXTRINSIC_DAPI,
            MVP_EXTRINSIC_SUBMIT_PROJECT_USAGE,
            id,
            usage
        );

        info!("[+] Composed Extrinsic:\n {:?}\n", xt);

        // send and watch extrinsic until InBlock
        let tx_hash = self
            .api
            .as_ref()
            .unwrap()
            .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)?;
        info!("[+] Transaction got included. Hash: {:?}", tx_hash);
        Ok(())
    }
    pub async fn submit_projects_usage(
        &self,
        projects_quota: Arc<RwLock<Projects>>,
        projects_request: HashMap<String, usize>,
    ) -> Result<(), anyhow::Error> {
        let projects_quota_clone;
        {
            let lock_projects_quota = projects_quota.read().await;
            projects_quota_clone = lock_projects_quota.0.clone();
        }
        for (project_id, request_number) in projects_request {
            if let Some(project_quota) = projects_quota_clone.get(&project_id) {
                let quota = project_quota.quota.parse::<usize>()?;
                info!(
                    "Project {} has requests number: {} Quota: {}, ",
                    project_id, request_number, quota
                );
                if let Err(e) = self.submit_project_usage(&project_id, request_number as u128) {
                    info!("submit_project_usage error:{:?}", e);
                };
            } else {
                info!(
                    "Warning: Project {} has requests number: {} but do not has quota info!",
                    project_id, request_number
                );
            }
        }
        Ok(())
    }

    pub async fn subscribe_event_update_quota(&self, projects: Arc<RwLock<Projects>>) {
        let (events_in, events_out) = channel();
        let api = self.api.as_ref().unwrap();
        api.subscribe_events(events_in).unwrap();
        loop {
            let event: ProjectRegisteredEventArgs = api
                .wait_for_event(
                    MVP_EXTRINSIC_DAPI,
                    MVP_EVENT_PROJECT_REGISTERED,
                    None,
                    &events_out,
                )
                .unwrap();
            info!("Got event: {:?}", event);
            {
                let mut projects_lock = projects.write().await;
                match projects_lock.0.entry(event.project_id_to_string()) {
                    Entry::Occupied(o) => {
                        let project = o.into_mut();
                        project.quota = event.quota.to_string();
                    }
                    Entry::Vacant(v) => {
                        let (blockchain, network) = event.get_blockchain_and_network();
                        v.insert(Project {
                            blockchain,
                            network,
                            quota: event.quota.to_string(),
                            status: "staked".to_string(),
                        });
                    }
                };
                info!("projects quota update by event: {:?}", projects_lock);
            }
        }
    }
}

impl std::fmt::Debug for ChainAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MvpAdapter")
    }
}
