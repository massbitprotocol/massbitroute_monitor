use std::collections::hash_map::Entry;
use std::fmt::Formatter;
// Massbit chain
use sp_core::{Bytes, sr25519};
use sp_core::sr25519::Pair;
use sp_core::crypto::Pair as _;
use sp_keyring::AccountKeyring;
use std::convert::TryFrom;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{compose_extrinsic, Api, GenericAddress, Metadata, UncheckedExtrinsicV4, XtStatus, AccountId};
use std::sync::mpsc::channel;
use jsonrpsee::tracing::Event;
use sp_core::H256 as Hash;
use substrate_api_client::extrinsic::balances;
use codec::Decode;
use std::collections::HashMap;
use std::sync::Arc;
use serde::{forward_to_deserialize_any_helper, Deserialize, Serialize};
use jsonrpsee::core::client::{
    Client as JsonRpcClient, ClientT, Subscription, SubscriptionClientT,
};
//use sp_core::LogLevel::Debug;
use tokio::sync::RwLock;
use std::fmt::{self, Debug, Display};
use minifier::js::Keyword::Default;

const MVP_EXTRINSIC_DAPI: &str = "Dapi";
const MVP_EXTRINSIC_SUBMIT_PROJECT_USAGE: &str = "submit_project_usage";
const MVP_EVENT_PROJECT_REGISTERED: &str = "ProjectRegistered";


type ProjectId = Bytes;
type ProjectIdString = String;
type Quota = String;

#[derive(Default,Debug,Serialize, Deserialize,Clone)]
pub struct Projects(pub HashMap<ProjectIdString,Project>);

#[derive(Default,Debug,Serialize, Deserialize,Clone)]
pub struct Project{
    blockchain: String,
    network: String,
    quota: Quota,
}

// #[derive(Default,Debug)]
// pub struct Dapi{
//     id: String,
//     status: u8,
// }



#[derive(Default)]
pub struct ChainAdapter {
    pub(crate) json_rpc_client: Option<JsonRpcClient>,
    pub(crate) ws_rpc_client: Option<WsRpcClient>,
    pub(crate) api: Option<Api<Pair, WsRpcClient>>
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct SubmitData {
    consumer_id: String,
    requests_count: isize,
    from_block_number: isize,
    to_block_number: isize,
}


#[derive(Decode)]
struct TransferEventArgs {
    from: AccountId,
    to: AccountId,
    value: u128,
}

#[derive(Decode,Debug)]
struct ProjectRegisteredEventArgs {
    project_id: Bytes,
    account_id: AccountId,
    chain_id: Bytes,
    quota: u64
}

impl ProjectRegisteredEventArgs {
    fn project_id_to_string(&self) -> String{
        String::from_utf8_lossy(&*self.project_id).to_string()
    }
    fn get_blockchain_and_network(&self) -> (String,String){
        let chain_id = String::from_utf8_lossy(&*self.chain_id).to_string();
        let chain_id = chain_id.split(".").collect::<Vec<&str>>();
        (chain_id[0].to_string(),chain_id[1].to_string())
    }
}

// #[derive(Decode,Debug)]
// enum PalletDapiBlockChain{
//     Ethereum,
//     Polkadot,
// }
// impl fmt::Display for PalletDapiBlockChain {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             PalletDapiBlockChain::Ethereum => {
//                 write!(f, "eth")
//             }
//             PalletDapiBlockChain::Polkadot => {
//                 write!(f, "dot")
//             }
//         }
//
//     }
// }


impl ChainAdapter {
    pub fn submit_project_usage(&self,project_id: &String, usage:u128) -> Result<(),anyhow::Error>{
        // set the recipient
        let api = self.api.as_ref().unwrap().clone();
        // the names are given as strings
        #[allow(clippy::redundant_clone)]
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
            api,
            MVP_EXTRINSIC_DAPI,
            MVP_EXTRINSIC_SUBMIT_PROJECT_USAGE,
            project_id.as_bytes(),
            usage
        );

        println!("[+] Composed Extrinsic:\n {:?}\n", xt);

        // send and watch extrinsic until InBlock
        let tx_hash = self.api.as_ref().unwrap()
            .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
            .unwrap();
        println!("[+] Transaction got included. Hash: {:?}", tx_hash);
        Ok(())
    }
    pub async fn submit_projects_usage(&self,projects_quota: Arc<RwLock<Projects>>, projects_request: HashMap<String,usize>) -> Result<(),anyhow::Error>{
        let mut projects_quota_clone;
        {
            let lock_projects_quota = projects_quota.read().await;
            projects_quota_clone = lock_projects_quota.0.clone();
        }
        for (project_id,request_number) in projects_request {
            if let Some(project_quota)=projects_quota_clone.get(&project_id){
                let quota = project_quota.quota.parse::<usize>()?;
                if  quota < request_number {
                    println!("Project {} has requests number: {} larger than Quota: {}, ",project_id,request_number,quota);
                    if let Err(e) = self.submit_project_usage(&project_id, request_number as u128){
                        println!("submit_project_usage error:{:?}",e);
                    };
                }
            }
            else {
                println!("Warning: Project {} has requests number: {} but do not has quota info!",project_id,request_number);
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
                .wait_for_event(MVP_EXTRINSIC_DAPI, MVP_EVENT_PROJECT_REGISTERED, None, &events_out)
                .unwrap();
            println!("Got event: {:?}", event);
            {
                let mut projects_lock = projects.write().await;
                match projects_lock.0.entry(event.project_id_to_string()) {
                    Entry::Occupied(o) => {
                        let project = o.into_mut();
                        project.quota = event.quota.to_string();
                    },
                    Entry::Vacant(v) => {
                        let (blockchain,network) = event.get_blockchain_and_network();
                        v.insert(Project {
                            blockchain,
                            network,
                            quota: event.quota.to_string(),
                        });
                    }
                };
                println!("projects: {:?}", projects_lock);
            }

        }


    }
}

impl std::fmt::Debug for ChainAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MvpAdapter")
    }
}
