#![forbid(unsafe_code)]

pub fn rollbackalwayspreservesevidence(before: &EvidenceBundle, after: &EvidenceBundle) {
    assert_eq!(before.sequences, after.sequences, "rollback lost evidence tags");
}

pub fn neverexceedenergyjoules(host_budget: &HostBudget, max_joules: f64) {
    assert!(host_budget.energy_joules <= max_joules, "energy envelope breached");
}
