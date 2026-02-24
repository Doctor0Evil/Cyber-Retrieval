use super::{NeuromorphicState, InvariantSnapshot, GovernanceVerdict, SovereignEndpoint};
use phoenix_aln_particles::{ALNComplianceParticle, ComplianceVerdict};
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

/// A candidate action decoded from neuromorphic state (e.g. intent vector → motor profile).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandidateAction {
    /// Host-local identifier for routing, logging, and replay.
    pub action_id: String,
    /// Raw payload (stim pattern, cursor command, API request, etc.).
    pub payload: Vec<u8>,
}

/// Policy engine that evaluates Cp(S(t), a(t)) over ALN rules and envelopes.
pub trait PolicyEngine {
    fn evaluate_policy(
        &self,
        state: &NeuromorphicState,
        action: &CandidateAction,
    ) -> ComplianceVerdict;

    fn build_aln_particle(
        &self,
        state: &NeuromorphicState,
        action: &CandidateAction,
        verdict: &ComplianceVerdict,
    ) -> ALNComplianceParticle;
}

/// Compute χ(t) and, if compliant, emit a SovereignEndpoint ready for audit and export.
pub fn gate_action_to_endpoint(
    policy: &dyn PolicyEngine,
    invariants: InvariantSnapshot,
    state: &NeuromorphicState,
    action: CandidateAction,
    state_hash: [u8; 32],
) -> Option<SovereignEndpoint> {
    let verdict = policy.evaluate_policy(state, &action);
    let aln_particle = policy.build_aln_particle(state, &action, &verdict);

    let compliance_predicate_pass = verdict.is_compliant && invariants.energy_ok
        && invariants.lyapunov_ok
        && invariants.plasticity_ok;

    let compliance_bit = compliance_predicate_pass;

    let governance = GovernanceVerdict {
        compliance_predicate_pass,
        compliance_bit,
        aln_particle,
        invariants,
    };

    if governance.compliance_bit {
        Some(SovereignEndpoint {
            endpoint_id: action.action_id.clone(),
            action_payload: action.payload,
            governance,
            state_hash,
            emitted_at: SystemTime::now(),
        })
    } else {
        None
    }
}
