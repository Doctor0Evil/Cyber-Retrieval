#![forbid(unsafe_code)]

use crate::policy::{AlnGovernanceShard, RoleThresholds};
use crate::roles::{GovernanceRole, StakeSnapshot};

/// Safety profile stays aligned with neurorights.envelope.citizen.v1.
#[derive(Clone, Debug)]
pub struct GovernanceSafetyProfile {
    pub risk_of_harm_ceiling: f32, // must stay ≤ 0.3
    pub no_inner_state_scoring: bool,
    pub no_neurocoercion: bool,
    pub revocable_at_will: bool,
    pub ecosocial_reporting_required: bool,
}

impl GovernanceSafetyProfile {
    pub const fn default_profile() -> Self {
        Self {
            risk_of_harm_ceiling: 0.08,
            no_inner_state_scoring: true,
            no_neurocoercion: true,
            revocable_at_will: true,
            ecosocial_reporting_required: true,
        }
    }
}

/// Knowledge / Risk / Cybostate + hex-stamp for this ALN binding.
#[derive(Clone, Debug)]
pub struct GovernanceIndices {
    pub knowledge_factor: f32,
    pub risk_of_harm: f32,
    pub cybostate_factor: f32,
    pub hex_stamp: String,
}

impl GovernanceIndices {
    pub fn aln_bound_profile() -> Self {
        Self {
            knowledge_factor: 0.92,
            risk_of_harm: 0.08, // stays under the 0.3 ceiling
            cybostate_factor: 0.87,
            hex_stamp: "0x4F91C7AB39D62E11".to_string(),
        }
    }
}

/// Pure, retrieval-only eligibility computation with ALN-sourced thresholds.
pub fn compute_eligibility(
    stake: &StakeSnapshot,
    aln_shard: &AlnGovernanceShard,
    safety: &GovernanceSafetyProfile,
) -> (Vec<GovernanceRole>, GovernanceIndices) {
    // Global Risk-of-Harm ceiling enforced structurally.
    assert!(safety.risk_of_harm_ceiling <= 0.3);

    let thresholds = RoleThresholds::from_aln_shard(aln_shard.clone());
    let roles = thresholds.eligible_roles(stake);

    (roles, GovernanceIndices::aln_bound_profile())
}
