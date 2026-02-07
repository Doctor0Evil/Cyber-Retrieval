#![cfg(kani)]
#![forbid(unsafe_code)]

use bioscale_upgrade_store::{HostBudget, BciHostSnapshot, UpgradeDescriptor};
use aln_invariants::AlnInvariantsSurface;
use cyberswarm_neurostack::bci_guard::evaluate_bci_upgrade;

#[kani::proof]
fn check_evolution_window_safety() {
    let inv: AlnInvariantsSurface = kani::any();
    kani::assume(inv.bci.max_joules_per_session > 0.0);
    kani::assume(inv.bci.max_duty_fraction <= 0.6);

    let budget: HostBudget = kani::any();
    let snap: BciHostSnapshot = kani::any();
    let desc: UpgradeDescriptor = kani::any();
    let thresholds = BciSafetyThresholds::from_invariants(&inv);
    let metrics = GuardMetrics::null_sink();

    let decision = evaluate_bci_upgrade(
        &inv, &thresholds, &budget, &snap, &desc,
        &EvolutionWindowState::zero(), &metrics,
    );

    // Invariant: decision can never approve when combined energy > session cap.
    let total_energy = budget.used_joules + desc.energy_cost_joules;
    if total_energy > inv.bci.max_joules_per_session {
        assert!(matches!(decision, UpgradeDecision::DeniedEnergy));
    }
}
