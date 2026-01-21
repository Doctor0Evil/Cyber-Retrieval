#![forbid(unsafe_code)]

use governance_core::eligibility::{compute_eligibility, GovernanceIndices};
use governance_core::policy::{GovernanceSafetyProfile, RoleThresholds};
use governance_core::roles::{GovernanceRole, StakeSnapshot};
use neurorights_core::{NeurorightsBound, NeurorightsEnvelope};
use neurorights_firewall::router::wrap_prompt;
use crate::audit::GovernanceDecisionLog;

/// Minimal view of the incoming prompt for governance ops.
#[derive(Clone, Debug)]
pub struct GovernanceQuery {
    pub holder_did: String,
    pub aln_scope: String,
    pub bostrom_address: String,
    pub stake: StakeSnapshot,
}

/// Governance result that can be written to logs / registry by another layer.
#[derive(Clone, Debug)]
pub struct GovernanceDecision {
    pub eligible_roles: Vec<GovernanceRole>,
    pub indices: GovernanceIndices,
}

pub fn handle_governance_query(
    bound: NeurorightsBound<crate::PromptEnvelope, NeurorightsEnvelope>,
    query: GovernanceQuery,
) -> (GovernanceDecision, GovernanceDecisionLog) {
    // PromptEnvelope has already passed neurorights checks via NeurorightsBound.
    let _env = bound.inner(); // safe, retrieval-only

    let thresholds = RoleThresholds::default_thresholds();
    let safety = GovernanceSafetyProfile::default_profile();
    let (roles, indices) = compute_eligibility(&query.stake, &thresholds, &safety);

    let decision = GovernanceDecision {
        eligible_roles: roles,
        indices,
    };

    let log = GovernanceDecisionLog::from_query_and_decision(&bound, &query, &decision);

    (decision, log)
}
