#![forbid(unsafe_code)]

use std::time::SystemTime;

/// Stake-bearing roles derived from ALN shards:
/// - asset.chat.stake.v1
/// - governance.chat.website.v1
/// - governance.totem.superposition.v1
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GovernanceRole {
    Superchair,
    Council,
    Proposer,
}

/// Minimal stake + contribution snapshot used for eligibility checks.
#[derive(Clone, Debug)]
pub struct StakeSnapshot {
    pub chat: u128,
    pub cy: u128,
    pub zen: u128,
    pub lifeforce: u128,
    pub contribution_index: u32,
}

/// Immutable role assignment with authorship and time bounds.
#[derive(Clone, Debug)]
pub struct RoleAssignment {
    pub role: GovernanceRole,
    pub holder_did: String,
    pub aln_scope: String,
    pub bostrom_address: String,
    pub term_start: SystemTime,
    pub term_end: Option<SystemTime>,
    pub hex_stamp: String,
}
