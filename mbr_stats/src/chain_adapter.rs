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
use tokio::sync::RwLock;

type ProjectId = Bytes;
type Quota = u64;

#[derive(Default,Debug)]
pub struct Projects(pub HashMap<ProjectId,Project>);

#[derive(Default,Debug)]
pub struct Project{
    project_hash: Hash,
    quota: Quota,
}

#[derive(Default)]
pub struct ChainAdapter {
    pub(crate) json_rpc_client: Option<JsonRpcClient>,
    pub(crate) ws_rpc_client: Option<WsRpcClient>,
    pub(crate) api: Option<Api<Pair, WsRpcClient>>
}

impl ChainAdapter {
    pub fn test_sign_extrinsic(&self) {
        ///////////// test substrate client
        // set the recipient
        let to = AccountKeyring::Alice.to_account_id();

        // call Balances::transfer
        // the names are given as strings
        #[allow(clippy::redundant_clone)]
            let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
            self.api.as_ref().unwrap().clone(),
            "Balances",
            "transfer",
            GenericAddress::Id(to),
            Compact(100_000_000_000_000_000_u128)
        );

        println!("[+] Composed Extrinsic:\n {:?}\n", xt);

        // send and watch extrinsic until InBlock
        let tx_hash = self.api.as_ref().unwrap()
            .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
            .unwrap();
        println!("[+] Transaction got included. Hash: {:?}", tx_hash);
        /////////////// end test substrate client
        loop {}
    }
    pub async fn subscribe_event_update_quota(&self, projects: Arc<RwLock<Projects>>) {
        let (events_in, events_out) = channel();
        let api = self.api.as_ref().unwrap();
        api.subscribe_events(events_in).unwrap();
        loop {
            let event: ProjectRegisteredEventArgs = api
                .wait_for_event("Dapi", "ProjectRegistered", None, &events_out)
                .unwrap();
            println!("Got event: {:?}", event);
            {
                let mut projects_lock = projects.write().await;
                projects_lock.0.insert(event.project_id,Project{
                    quota: event.quota,
                    project_hash: event.project_hash,
                });
            }
            //println!("projects: {:?}", projects);
        }


    }
}

impl std::fmt::Debug for ChainAdapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MvpAdapter")
    }
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
    project_hash: Hash,
    account_id: AccountId,
    blockchain: PalletDapiBlockChain,
    quota: u64
}

#[derive(Decode,Debug)]
enum PalletDapiBlockChain{
    Ethereum,
    Polkadot,
}
