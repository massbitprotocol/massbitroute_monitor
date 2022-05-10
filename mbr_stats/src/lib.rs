// #![feature(fmt_internals)]
// #![feature(print_internals)]

#[path = "chain_adapter.rs"]
pub mod chain_adapter;
pub mod component_stats;
use lazy_static::lazy_static;

lazy_static! {
    static ref SIGNER_DERIVE: String = "Ferdie".to_string();
}
