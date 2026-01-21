#![forbid(unsafe_code)]

pub mod roles;
pub mod risk;
pub mod handlers;

include!(concat!(env!("OUT_DIR"), "/aln_generated.rs"));
