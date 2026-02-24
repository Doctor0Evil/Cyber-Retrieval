use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

use bioscale_upgrade_store::{
    EvidenceBundle, UpgradeDescriptor, UpgradeId,
};
use phoenix_aln_particles::ALNComplianceParticle;
use sha2::{Digest, Sha256};

/// Identifier for a specific frozen invariant spec (energy, Lyapunov, policy).
/// In practice this is derived from the UnifiedMasterConfig document hash.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InvariantSpecId(pub String);

/// Execution mode: pure mathematical reference or device‑trusted runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    ReferenceModel,
    DeviceTrustedRuntime,
}

/// Canonical, append‑only record of an evaluated trace under a given invariant spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionAuditRecord {
    /// Monotonic sequence number for this trace within a code version.
    pub seq: u64,
    /// Git SHA or semantic version of the code that produced this record.
    pub code_version: String,
    /// Frozen invariant spec identifier (hash of UnifiedMasterConfig core fields).
    pub invariant_spec_id: InvariantSpecId,
    /// Upgrade being exercised (if any).
    pub upgrade_id: Option<UpgradeId>,
    /// Execution mode (reference vs hardware).
    pub mode: ExecutionMode,
    /// Millisecond‑resolution wall‑clock window for this evaluation.
    pub started_at: SystemTime,
    pub finished_at: SystemTime,
    /// Raw input trace identifier (hash over biosignal + context).
    pub input_trace_hash: String,
    /// Hash of the ordered state‑transition log.
    pub state_log_hash: String,
    /// Hash of the ordered invariant‑evaluation log.
    pub invariant_log_hash: String,
    /// Hash of the ordered compliance‑decision log.
    pub decision_log_hash: String,
    /// Combined digest over the three log hashes plus invariant spec.
    pub composite_hash: String,
    /// ALN particle that authorized this execution (sovereign consent).
    pub aln_particle: ALNComplianceParticle,
    /// Evidence bundle used to justify biophysical corridors for this run.
    pub evidence: EvidenceBundle,
    /// True if this evaluation was performed on a TPM/TEE‑attested runtime
    /// and the attestation report is embedded in `runtime_attestation`.
    pub hardware_attested: bool,
    /// Opaque TPM/TEE attestation artifact (CBOR, JSON, etc.); optional in reference mode.
    pub runtime_attestation: Option<Vec<u8>>,
}

/// Result of an equivalence check between two versions for a given trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquivalenceStatus {
    /// All hashes match; new version is behaviorally indistinguishable.
    Equivalent,
    /// Invariants and decisions match, but internal state log differs;
    /// acceptable only if policy allows internal refactors.
    InternalsDiffer,
    /// Any invariant or decision hash differs; upgrade must be rejected
    /// unless explicit breaking‑change consent is recorded.
    NonEquivalent,
}

/// Frozen mathematical core that must never change without explicit
/// “breaking‑change” governance. This is where your energy balance,
/// Lyapunov functional, and policy predicates live.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenInvariantSpec {
    /// Version label for human tracking (e.g., "neuromath-core-1.0").
    pub label: String,
    /// SHA‑256 over a canonical serialization of the invariant math
    /// (e.g., UnifiedMasterConfig subset).
    pub spec_hash: String,
    /// If true, any change to this spec requires explicit renewed consent
    /// by the host (via ALNComplianceParticle) and cannot be silently
    /// rolled forward by OTA processes.
    pub requires_explicit_consent: bool,
}

impl FrozenInvariantSpec {
    /// Deterministically derive the InvariantSpecId used in audit records.
    pub fn id(&self) -> InvariantSpecId {
        InvariantSpecId(self.spec_hash.clone())
    }

    /// Compute a spec hash from a canonical JSON representation of the math
    /// (energy balance, Lyapunov functional, policy predicates).
    pub fn from_canonical_json(label: &str, json: &str, requires_consent: bool) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let spec_hash = hex::encode(hasher.finalize());
        Self {
            label: label.to_string(),
            spec_hash,
            requires_explicit_consent: requires_consent,
        }
    }
}

/// Interface for components that can produce deterministic, cryptographically
/// comparable outputs for regression testing under a frozen invariant spec.
pub trait DualModeReference {
    /// Run the pure mathematical reference implementation.
    fn eval_reference(
        &self,
        invariants: &FrozenInvariantSpec,
        input_trace_hash: &str,
    ) -> EvolutionAuditRecord;

    /// Run the device‑trusted runtime, with TPM/TEE attestation report.
    fn eval_device_trusted(
        &self,
        invariants: &FrozenInvariantSpec,
        input_trace_hash: &str,
        attestation: Vec<u8>,
    ) -> EvolutionAuditRecord;
}

/// Helper to compute a composite hash from the three log hashes and the spec.
fn composite_hash(
    invariant_spec_id: &InvariantSpecId,
    state_log_hash: &str,
    invariant_log_hash: &str,
    decision_log_hash: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(invariant_spec_id.0.as_bytes());
    hasher.update(state_log_hash.as_bytes());
    hasher.update(invariant_log_hash.as_bytes());
    hasher.update(decision_log_hash.as_bytes());
    hex::encode(hasher.finalize())
}

impl EvolutionAuditRecord {
    /// Construct a new audit record, computing the composite hash from the
    /// underlying per‑log hashes and invariant spec id.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        seq: u64,
        code_version: String,
        invariants: &FrozenInvariantSpec,
        upgrade: Option<&UpgradeDescriptor>,
        mode: ExecutionMode,
        started_at: SystemTime,
        finished_at: SystemTime,
        input_trace_hash: String,
        state_log_hash: String,
        invariant_log_hash: String,
        decision_log_hash: String,
        aln_particle: ALNComplianceParticle,
        evidence: EvidenceBundle,
        hardware_attested: bool,
        runtime_attestation: Option<Vec<u8>>,
    ) -> Self {
        let invariant_spec_id = invariants.id();
        let composite = composite_hash(
            &invariant_spec_id,
            &state_log_hash,
            &invariant_log_hash,
            &decision_log_hash,
        );
        let upgrade_id = upgrade.map(|u| u.id.clone());
        Self {
            seq,
            code_version,
            invariant_spec_id,
            upgrade_id,
            mode,
            started_at,
            finished_at,
            input_trace_hash,
            state_log_hash,
            invariant_log_hash,
            decision_log_hash,
            composite_hash: composite,
            aln_particle,
            evidence,
            hardware_attested,
            runtime_attestation,
        }
    }

    /// Wall‑clock duration of this evaluation.
    pub fn elapsed(&self) -> Option<Duration> {
        self.finished_at.duration_since(self.started_at).ok()
    }
}

/// Strong equivalence check between an old and new version under the same
/// frozen invariant spec and input trace.
///
/// This function encodes your “non‑reversible compliance” rule:
/// an upgrade is considered safe only if the composite hashes match,
/// meaning the sequence of computationally gated actions and decisions
/// is identical for the same input and invariant spec.
pub fn check_equivalence(
    old: &EvolutionAuditRecord,
    new: &EvolutionAuditRecord,
) -> EquivalenceStatus {
    // Invariant spec and input trace must be identical.
    if old.invariant_spec_id != new.invariant_spec_id
        || old.input_trace_hash != new.input_trace_hash
    {
        return EquivalenceStatus::NonEquivalent;
    }

    // Full composite match: strongest guarantee.
    if old.composite_hash == new.composite_hash {
        return EquivalenceStatus::Equivalent;
    }

    // If invariants and decisions are identical, but state logs differ,
    // we treat this as a weaker form of equivalence that can be allowed
    // only under an explicit policy.
    if old.invariant_log_hash == new.invariant_log_hash
        && old.decision_log_hash == new.decision_log_hash
    {
        return EquivalenceStatus::InternalsDiffer;
    }

    EquivalenceStatus::NonEquivalent
}

/// Policy to decide whether a candidate upgrade may be accepted given its
/// equivalence status and the host's sovereign consent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeDecision {
    /// Fully accepted as a non‑breaking upgrade.
    Accept,
    /// Rejected as non‑equivalent; previous version must remain active.
    Reject,
    /// Requires explicit renewed consent from the host (breaking change).
    RequireRenewedConsent,
}

/// High‑level gate that combines equivalence results with the invariant
/// spec's consent requirements and the ALNComplianceParticle.
/// This is where the citizen's sovereignty is enforced in code.
pub fn decide_upgrade(
    invariants: &FrozenInvariantSpec,
    equiv: EquivalenceStatus,
    aln_particle: &ALNComplianceParticle,
) -> UpgradeDecision {
    match equiv {
        EquivalenceStatus::Equivalent => UpgradeDecision::Accept,
        EquivalenceStatus::InternalsDiffer => {
            // Only allow if invariants do NOT require explicit consent and
            // the ALN particle asserts that internal refactors are acceptable.
            if !invariants.requires_explicit_consent
                && aln_particle.allows_internal_refactor()
            {
                UpgradeDecision::Accept
            } else {
                UpgradeDecision::RequireRenewedConsent
            }
        }
        EquivalenceStatus::NonEquivalent => {
            // Non‑equivalent behavior is always a breaking change.
            UpgradeDecision::RequireRenewedConsent
        }
    }
}

/// Extension trait for ALNComplianceParticle to express consent checks
/// used in upgrade decisions. The concrete struct already carries
/// host DID, consent ledger refs, clause IDs, etc.
pub trait AlnUpgradeConsent {
    /// True if this particle contains explicit host consent for upgrading
    /// under a given invariant spec id (e.g., via a neurorights clause).
    fn allows_invariant_change(&self, invariant_spec: &InvariantSpecId) -> bool;

    /// True if the host has granted permission for internal refactors that
    /// preserve observable decisions and invariants.
    fn allows_internal_refactor(&self) -> bool;
}

impl AlnUpgradeConsent for ALNComplianceParticle {
    fn allows_invariant_change(&self, invariant_spec: &InvariantSpecId) -> bool {
        // Implementation detail:
        // Map clause IDs (e.g., "upgrade.breaking.allowed.invariant:<hash>")
        // to this spec. This keeps policy in ALN while enforcement is in Rust.
        self.clauses
            .iter()
            .any(|c| c.applies_to_invariant(&invariant_spec.0))
    }

    fn allows_internal_refactor(&self) -> bool {
        // Example: look for a generic "upgrade.internal_refactor.ok" ALN clause.
        self.clauses
            .iter()
            .any(|c| c.id == "upgrade.internal_refactor.ok")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phoenix_aln_particles::tests::dummy_particle_allowing_refactor;

    fn dummy_spec() -> FrozenInvariantSpec {
        FrozenInvariantSpec::from_canonical_json(
            "test-core",
            r#"{"energy_balance":"E_in - E_out = dE","lyapunov":"V(x) >= 0"}"#,
            true,
        )
    }

    #[test]
    fn equivalent_hashes_accept() {
        let spec = dummy_spec();
        let aln = dummy_particle_allowing_refactor();
        let mut base = EvolutionAuditRecord {
            seq: 1,
            code_version: "v1".into(),
            invariant_spec_id: spec.id(),
            upgrade_id: None,
            mode: ExecutionMode::ReferenceModel,
            started_at: SystemTime::now(),
            finished_at: SystemTime::now(),
            input_trace_hash: "trace".into(),
            state_log_hash: "s".into(),
            invariant_log_hash: "i".into(),
            decision_log_hash: "d".into(),
            composite_hash: String::new(),
            aln_particle: aln.clone(),
            evidence: EvidenceBundle::default(),
            hardware_attested: false,
            runtime_attestation: None,
        };
        base.composite_hash = composite_hash(
            &base.invariant_spec_id,
            &base.state_log_hash,
            &base.invariant_log_hash,
            &base.decision_log_hash,
        );

        let new = base.clone();
        let status = check_equivalence(&base, &new);
        assert_eq!(status, EquivalenceStatus::Equivalent);
        let decision = decide_upgrade(&spec, status, &base.aln_particle);
        assert_eq!(decision, UpgradeDecision::Accept);
    }

    #[test]
    fn non_equivalent_requires_renewed_consent() {
        let spec = dummy_spec();
        let aln = phoenix_aln_particles::tests::dummy_particle_neutral();
        let mut old = EvolutionAuditRecord {
            seq: 1,
            code_version: "v1".into(),
            invariant_spec_id: spec.id(),
            upgrade_id: None,
            mode: ExecutionMode::ReferenceModel,
            started_at: SystemTime::now(),
            finished_at: SystemTime::now(),
            input_trace_hash: "trace".into(),
            state_log_hash: "s1".into(),
            invariant_log_hash: "i1".into(),
            decision_log_hash: "d1".into(),
            composite_hash: String::new(),
            aln_particle: aln.clone(),
            evidence: EvidenceBundle::default(),
            hardware_attested: false,
            runtime_attestation: None,
        };
        old.composite_hash = composite_hash(
            &old.invariant_spec_id,
            &old.state_log_hash,
            &old.invariant_log_hash,
            &old.decision_log_hash,
        );

        let mut new = old.clone();
        new.state_log_hash = "s2".into();
        new.decision_log_hash = "d2".into();
        new.composite_hash = composite_hash(
            &new.invariant_spec_id,
            &new.state_log_hash,
            &new.invariant_log_hash,
            &new.decision_log_hash,
        );

        let status = check_equivalence(&old, &new);
        assert_eq!(status, EquivalenceStatus::NonEquivalent);
        let decision = decide_upgrade(&spec, status, &old.aln_particle);
        assert_eq!(decision, UpgradeDecision::RequireRenewedConsent);
    }
}
