#![forbid(unsafe_code)]

use std::time::SystemTime;

use bioscale_upgrade_store::{
    UpgradeDescriptor, HostBudget, UpgradeDecision, EvidenceBundle,
};
use cybernet_metrics::BiophysicalFlowsSnapshot; // CEIM/NanoKarma style external flows
use cybernet_aln::FairnessClauseRef;            // ALN clause handle

/// 1D biospatial fairness coordinate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResponsibilityScalar(pub f64);

impl ResponsibilityScalar {
    /// Neutral baseline r = 0.
    pub fn neutral() -> Self {
        ResponsibilityScalar(0.0)
    }

    /// Monotone-safe addition of a delta.
    pub fn add_delta(self, delta: f64) -> Self {
        ResponsibilityScalar(self.0 + delta)
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

/// Outer action / freedom descriptor coupled to r.
#[derive(Clone, Debug)]
pub struct OuterFreedomEnvelope {
    /// Maximum outer action “radius” permitted at this r (normalized 0–1).
    pub max_outer_radius: f64,
    /// Optional domain-specific caps (bandwidth, control channels, etc.).
    pub bandwidth_cap: f64,
    pub device_control_cap: f64,
}

/// Biophysical evidence for Δr.
#[derive(Clone, Debug)]
pub struct ResponsibilityEvidence {
    /// Measured external flows (energy, mass, emissions, sequestration).
    pub flows: BiophysicalFlowsSnapshot,
    /// Rust-side responsibility delta in physical units mapped to r.
    pub delta_r: f64,
    /// Ten-hex scientific anchors.
    pub evidence_bundle: EvidenceBundle,
}

/// Predicate result: is an action fair on the r-axis.
#[derive(Clone, Debug)]
pub struct KarmaVerdict {
    pub is_karma_admissible: bool,
    pub pre_r: ResponsibilityScalar,
    pub post_r: ResponsibilityScalar,
    pub delta_r: f64,
}

/// Trait mapping r to admissible outer freedom.
/// This is the public b_outer(r) function.
pub trait OuterFreedomModel {
    /// Compute outer envelope allowed at current r.
    fn outer_envelope(&self, r: ResponsibilityScalar) -> OuterFreedomEnvelope;

    /// Hard monotonicity check: for r2 ≥ r1, envelope must not shrink.
    fn monotone_check(&self, r1: ResponsibilityScalar, r2: ResponsibilityScalar) -> bool {
        if r2.value() < r1.value() {
            return true;
        }
        let e1 = self.outer_envelope(r1);
        let e2 = self.outer_envelope(r2);
        e2.max_outer_radius + 1e-9 >= e1.max_outer_radius
            && e2.bandwidth_cap + 1e-9 >= e1.bandwidth_cap
            && e2.device_control_cap + 1e-9 >= e1.device_control_cap
    }
}

/// Trait every upgrade or artifact must implement to be considered fair.
/// All gating logic for outer power passes through this predicate.
pub trait KarmaConstraint {
    /// Compute Δr from external flows; MUST ignore neural telemetry.
    fn responsibility_delta(
        &self,
        now: SystemTime,
        flows: &BiophysicalFlowsSnapshot,
    ) -> ResponsibilityEvidence;

    /// Core predicate: outer freedom may not grow if Δr < 0.
    fn karma_admissible(
        &self,
        now: SystemTime,
        pre_r: ResponsibilityScalar,
        flows: &BiophysicalFlowsSnapshot,
        model: &dyn OuterFreedomModel,
    ) -> KarmaVerdict {
        let re = self.responsibility_delta(now, flows);
        let post_r = pre_r.add_delta(re.delta_r);

        // Compute outer envelopes before and after.
        let pre_env = model.outer_envelope(pre_r);
        let post_env = model.outer_envelope(post_r);

        // Greed signature: post-env larger while Δr < 0.
        let greed_pattern = re.delta_r < 0.0
            && (post_env.max_outer_radius > pre_env.max_outer_radius + 1e-9
                || post_env.bandwidth_cap > pre_env.bandwidth_cap + 1e-9
                || post_env.device_control_cap > pre_env.device_control_cap + 1e-9);

        KarmaVerdict {
            is_karma_admissible: !greed_pattern && re.delta_r >= 0.0,
            pre_r,
            post_r,
            delta_r: re.delta_r,
        }
    }
}

/// Trait that couples UpgradeDescriptor to fairness.
/// This is the compile-time gate for bioscale upgrades.
pub trait FairnessConstrainedUpgrade: KarmaConstraint {
    fn descriptor(&self) -> &UpgradeDescriptor;

    /// HostBudget + CEIM/NanoKarma + r-axis gate.
    fn evaluate_with_fairness(
        &self,
        host: &HostBudget,
        now: SystemTime,
        current_r: ResponsibilityScalar,
        flows: &BiophysicalFlowsSnapshot,
        model: &dyn OuterFreedomModel,
    ) -> UpgradeDecision {
        let verdict = self.karma_admissible(now, current_r, flows, model);

        if !verdict.is_karma_admissible {
            return UpgradeDecision::Denied {
                reason: "FairnessConstraint: Δr < 0 for requested outer expansion".into(),
            };
        }

        // Delegate to existing bioscale evaluation; outer envelope is already
        // encoded in HostBudget / descriptor bounds.
        bioscale_upgrade_store::evaluate_upgrade(self.descriptor(), host, now)
    }
}

/// Binding from Rust predicate to ALN clause so legal/ethical rules
/// can reference the same fairness invariant.
#[derive(Clone, Debug)]
pub struct FairnessBinding {
    pub aln_clause: FairnessClauseRef,
    pub pre_r: ResponsibilityScalar,
    pub post_r: ResponsibilityScalar,
    pub delta_r: f64,
}

impl FairnessBinding {
    pub fn from_verdict(aln_clause: FairnessClauseRef, v: &KarmaVerdict) -> Self {
        FairnessBinding {
            aln_clause,
            pre_r: v.pre_r,
            post_r: v.post_r,
            delta_r: v.delta_r,
        }
    }
}
