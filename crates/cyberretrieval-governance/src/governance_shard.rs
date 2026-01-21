#![forbid(unsafe_code)]

use std::time::{Duration, SystemTime};

use crate::prompt::{PromptEnvelope, GovernanceActionKind};
use crate::router::NeurorightsBoundEnvelope;
use governance_core::roles::{GovernanceRole, RoleAssignment, StakeSnapshot};
use governance_core::policy::{AlnGovernanceShard, RoleThresholds};
use governance_core::eligibility::{compute_eligibility, GovernanceSafetyProfile, GovernanceIndices};
use neurorights_core::{NeurorightsBound, NeurorightsEnvelope};
use neurorights_firewall::audit::{Authorship, EvidenceStamp};

/// Tiered continuity outcome for a single governance action.
#[derive(Clone, Debug)]
pub struct ContinuityResolution {
    pub acting_role: Option<GovernanceRole>,
    pub fallback_role: Option<GovernanceRole>,
    pub disqualified_roles: Vec<GovernanceRole>,
    pub new_assignments: Vec<RoleAssignment>,
    pub indices: GovernanceIndices,
}

/// Minimal neurorights‑bound governance input.
#[derive(Clone, Debug)]
pub struct GovernanceContext {
    pub holder_did: String,
    pub aln_scope: String,
    pub bostrom_address: String,
    pub stake: StakeSnapshot,
    pub current_time: SystemTime,
    pub action: GovernanceActionKind,
    pub aln_governance_shard: AlnGovernanceShard,
}

/// Deterministic succession rule, mirroring `successionrules` from
/// `governance.totem.superposition.v1`.
#[derive(Clone, Debug)]
pub struct SuccessionRules {
    pub interimlimit_days: u64,
    pub on_resign: GovernanceRole,      // e.g. Council leads election
    pub on_misconduct: GovernanceRole,  // e.g. Council as interim
}

impl SuccessionRules {
    pub fn default_superchair_rules() -> Self {
        Self {
            interimlimit_days: 14,
            on_resign: GovernanceRole::Council,
            on_misconduct: GovernanceRole::Council,
        }
    }
}

/// Neurorights‑bound audit record for this continuity step.
#[derive(Clone, Debug)]
pub struct GovernanceAuditHop {
    pub envelope_hex_stamp: String,
    pub authorship: Authorship,
    pub evidence: EvidenceStamp,
    pub continuity: ContinuityResolution,
}

/// High‑level continuity dataplan:
/// 1. Compute eligible roles from ALN thresholds.
/// 2. Apply auto‑disqualification: stake / term / neurorights.
/// 3. Choose acting role in order: Superchair → Council → Proposer.
/// 4. If Superchair lost, apply successionrules deterministically.
/// 5. Emit a GovernanceAuditHop with hex‑stamped PromptEnvelope context.
pub fn resolve_continuity(
    bound_env: NeurorightsBoundEnvelope,
    ctx: GovernanceContext,
    succession: &SuccessionRules,
    safety: &GovernanceSafetyProfile,
) -> (ContinuityResolution, GovernanceAuditHop) {
    // 0. Neurorights firewall: if this compiled and we have a NeurorightsBound,
    //    then noscorefrominnerstate / noneurocoercion / revocability are already satisfied.
    let inner: &NeurorightsBound<PromptEnvelope, NeurorightsEnvelope> = &bound_env.bound;

    // 1. Eligibility from ALN shard + stake.
    let (eligible_roles, indices) =
        compute_eligibility(&ctx.stake, &ctx.aln_governance_shard, safety);

    // 2. Auto‑disqualification predicates (stake/term/profile) at runtime boundary.
    let mut disqualified = Vec::new();
    let mut still_eligible: Vec<GovernanceRole> = Vec::new();

    for role in eligible_roles.iter() {
        if !is_role_valid(role, &ctx, inner) {
            disqualified.push(role.clone());
        } else {
            still_eligible.push(role.clone());
        }
    }

    // 3. Continuity order: Superchair → Council → Proposer.
    let acting_role = pick_acting_role(&still_eligible);
    let mut fallback_role: Option<GovernanceRole> = None;
    let mut new_assignments: Vec<RoleAssignment> = Vec::new();

    // 4. Succession logic for lost Superchair.
    if disqualified.contains(&GovernanceRole::Superchair) {
        fallback_role = Some(succession.on_misconduct.clone());
        if let Some(role) = &fallback_role {
            let assignment = RoleAssignment {
                role: role.clone(),
                holder_did: ctx.holder_did.clone(),
                aln_scope: ctx.aln_scope.clone(),
                bostrom_address: ctx.bostrom_address.clone(),
                term_start: ctx.current_time,
                term_end: Some(ctx.current_time + Duration::from_secs(
                    succession.interimlimit_days * 24 * 3600,
                )),
                hex_stamp: inner.envelope().hex_stamp.clone(),
            };
            new_assignments.push(assignment);
        }
    }

    let continuity = ContinuityResolution {
        acting_role,
        fallback_role,
        disqualified_roles: disqualified,
        new_assignments,
        indices: indices.clone(),
    };

    let audit = build_audit_hop(inner, &ctx, &continuity);

    (continuity, audit)
}

/// Check disqualification predicates for a role:
/// - Stake / contribution from ALN thresholds are already enforced in compute_eligibility.
/// - Here we add term expiry and neurorights profile validity.
fn is_role_valid(
    role: &GovernanceRole,
    ctx: &GovernanceContext,
    inner: &NeurorightsBound<PromptEnvelope, NeurorightsEnvelope>,
) -> bool {
    // Term validity: if PromptEnvelope encodes a term_end for this role, enforce it.
    if let Some(term) = inner.envelope().role_term_end(role) {
        if ctx.current_time > term {
            return false;
        }
    }

    // Neurorights profile validity: delegate to neurorights_envelope guard.
    if !inner.neurorights_envelope().profile_is_strong_enough(role) {
        return false;
    }

    true
}

/// Prioritize acting role in fixed order: Superchair > Council > Proposer.
fn pick_acting_role(roles: &[GovernanceRole]) -> Option<GovernanceRole> {
    if roles.contains(&GovernanceRole::Superchair) {
        return Some(GovernanceRole::Superchair);
    }
    if roles.contains(&GovernanceRole::Council) {
        return Some(GovernanceRole::Council);
    }
    if roles.contains(&GovernanceRole::Proposer) {
        return Some(GovernanceRole::Proposer);
    }
    None
}

/// Build a neurorights‑bound audit hop for the neural rope.
fn build_audit_hop(
    bound: &NeurorightsBound<PromptEnvelope, NeurorightsEnvelope>,
    ctx: &GovernanceContext,
    continuity: &ContinuityResolution,
) -> GovernanceAuditHop {
    let env = bound.envelope();
    let authorship = Authorship {
        userdid: ctx.holder_did.clone(),
        aln: ctx.aln_scope.clone(),
        bostromaddress: ctx.bostrom_address.clone(),
        eibonlabel: env.eibon_label.clone(),
        neurorightsversion: bound.neurorights_envelope().policyversion.to_string(),
    };

    let evidence = EvidenceStamp::default_hex();

    GovernanceAuditHop {
        envelope_hex_stamp: env.hex_stamp.clone(),
        authorship,
        evidence,
        continuity: continuity.clone(),
    }
}
