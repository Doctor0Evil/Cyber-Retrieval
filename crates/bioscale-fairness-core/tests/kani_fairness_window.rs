#![cfg(kani)]

use kani::any;
use bioscale_fairness_core::{
    ResponsibilityScalar, OuterFreedomModel, OuterFreedomEnvelope, KarmaConstraint,
};
use bioscale_upgrade_store::{HostBudget, UpgradeDecision};
use cybernet_metrics::BiophysicalFlowsSnapshot;

/// Simple monotone b_outer(r) model: radius grows linearly with r≥0, clamped at 0 for r<0.
struct LinearOuterFreedom;

impl OuterFreedomModel for LinearOuterFreedom {
    fn outer_envelope(&self, r: ResponsibilityScalar) -> OuterFreedomEnvelope {
        let v = r.value();
        let radius = if v <= 0.0 { 0.0 } else { v.min(1.0) };
        OuterFreedomEnvelope {
            max_outer_radius: radius,
            bandwidth_cap: radius,
            device_control_cap: radius,
        }
    }
}

/// Stub upgrade implementing KarmaConstraint so Kani can explore sequences.
#[derive(Clone, Debug)]
struct StubUpgrade;

impl KarmaConstraint for StubUpgrade {
    fn responsibility_delta(
        &self,
        _now: std::time::SystemTime,
        flows: &BiophysicalFlowsSnapshot,
    ) -> bioscale_fairness_core::ResponsibilityEvidence {
        // For harness, delta_r is just provided by flows.r_delta, bounded by Kani.
        bioscale_fairness_core::ResponsibilityEvidence {
            flows: flows.clone(),
            delta_r: flows.r_delta,
            evidence_bundle: flows.evidence.clone(),
        }
    }
}

#[kani::proof]
fn check_evolution_window_fairness() {
    let mut r = ResponsibilityScalar::neutral();
    let model = LinearOuterFreedom;
    let now = std::time::UNIX_EPOCH;

    // Bounded window of N steps.
    const N: usize = 3;

    for _ in 0..N {
        // Kani chooses flows; r_delta unconstrained.
        let flows: BiophysicalFlowsSnapshot = any();
        let up = StubUpgrade;

        let pre_env = model.outer_envelope(r);
        let verdict = up.karma_admissible(now, r, &flows, &model);
        let post_env = model.outer_envelope(verdict.post_r);

        // Fairness invariant: if Δr < 0, outer envelope must NOT expand.
        if verdict.delta_r < 0.0 {
            assert!(post_env.max_outer_radius <= pre_env.max_outer_radius + 1e-9);
            assert!(post_env.bandwidth_cap <= pre_env.bandwidth_cap + 1e-9);
            assert!(post_env.device_control_cap <= pre_env.device_control_cap + 1e-9);
        }

        r = verdict.post_r;
    }
}
