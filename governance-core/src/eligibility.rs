#![forbid(unsafe_code)]

use crate::policy::{GovernanceSafetyProfile, RoleThresholds};
use crate::roles::{GovernanceRole, RoleAssignment, StakeSnapshot};
use std::time::SystemTime;

/// Simple Knowledge / Risk / Cybostate annotation.
#[derive(Clone, Debug)]
pub struct GovernanceIndices {
    pub knowledge_factor: f32,
    pub risk_of_harm: f32,
    pub cybostate_factor: f32,
    pub hex_stamp: String,
}

impl GovernanceIndices {
    pub fn governance_module_profile() -> Self {
        Self {
            knowledge_factor: 0.9,
            risk_of_harm: 0.08,
            cybostate_factor: 0.87,
            hex_stamp: "0x4F91C7AB39D62E11".to_string(),
        }
    }
}

/// Pure, retrieval-only eligibility computation.
pub fn compute_eligibility(
    stake: &StakeSnapshot,
    thresholds: &RoleThresholds,
    safety: &GovernanceSafetyProfile,
) -> (Vec<GovernanceRole>, GovernanceIndices) {
    // Enforce global Risk-of-Harm ceiling structurally.
    assert!(safety.risk_of_harm_ceiling <= 0.3);

    let roles = thresholds.eligible_roles(stake);
    (roles, GovernanceIndices::governance_module_profile())
}

/// Build a RoleAssignment without side effects (no writes, no network).
pub fn assign_role_snapshot(
    role: GovernanceRole,
    holder_did: String,
    aln_scope: String,
    bostrom_address: String,
) -> RoleAssignment {
    RoleAssignment {
        role,
        holder_did,
        aln_scope,
        bostrom_address,
        term_start: SystemTime::now(),
        term_end: None,
        hex_stamp: "0x6AF08C5D3B917E24D0C42EB1F39A8C72".to_string(),
    }
}
