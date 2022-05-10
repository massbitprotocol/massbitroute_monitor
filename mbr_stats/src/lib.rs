// #![feature(fmt_internals)]
// #![feature(print_internals)]

pub mod component_stats;
#[path = "chain_adapter.rs"]
pub mod chain_adapter;
use lazy_static::lazy_static;

// lazy_static! {
//     static ref CHAIN_NETWORK_LIST: Vec<String> = vec!["eth-mainnet","dot-mainnet"];
// }