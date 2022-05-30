// #![feature(fmt_internals)]
// #![feature(print_internals)]

#[path = "chain_adapter.rs"]
pub mod chain_adapter;
pub mod component_stats;
use dotenv;
use lazy_static::lazy_static;
use std::env;
lazy_static! {
    pub static ref SIGNER_PHRASE: String =
        env::var("SIGNER_PHRASE").expect("There is no env var SIGNER_PHRASE");
    pub static ref PORTAL_AUTHORIZATION: String =
        env::var("PORTAL_AUTHORIZATION").expect("There is no env var PORTAL_AUTHORIZATION");
}
