#![forbid(unsafe_code)]

use bioscale_upgrade_store::{
    UpgradeDescriptor, HostBudget, BciHostSnapshot, BciSafetyThresholds, UpgradeDecision,
};
use bioscale_metrics_guard::GuardMetrics;
use aln_invariants::AlnInvariantsSurface;

pub fn evaluate_bci_upgrade(
    invariants: &AlnInvariantsSurface,
    thresholds: &BciSafetyThresholds,
    host_budget: &HostBudget,
    snapshot: &BciHostSnapshot,
    desc: &UpgradeDescriptor,
    state: &EvolutionWindowState,
    metrics: &GuardMetrics,
) -> UpgradeDecision {
    let mut decision = UpgradeDecision::Approved;

    if !thresholds.energy_within_bounds(host_budget, desc, invariants) {
        metrics.bciguard_denied("energy_budget_exceeded", &desc.upgrade_id);
        decision = UpgradeDecision::DeniedEnergy;
    }

    if !thresholds.thermal_within_bounds(snapshot, invariants) {
        metrics.bciguard_denied("thermal_envelope_exceeded", &desc.upgrade_id);
        decision = UpgradeDecision::DeniedThermal;
    }

    if !thresholds.duty_within_bounds(snapshot, invariants) {
        metrics.bciguard_denied("duty_envelope_exceeded", &desc.upgrade_id);
        decision = UpgradeDecision::DeniedDuty;
    }

    // … additional checks for protein, inflammation, neurorights flags …

    metrics.bciguard_observe_corridor(snapshot, desc, &decision);
    decision
}
