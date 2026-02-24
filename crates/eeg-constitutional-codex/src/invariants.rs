use super::{SignalState, NeuromorphicState, InvariantSnapshot};
use bioscale_upgrade_store::{HostBudget, ThermodynamicEnvelope, MlPassSchedule};
use cyberswarm_neurostack::{BciHostSnapshot, BciSafetyThresholds};

/// Trait for controllers that can be checked against the constitutional invariants.
pub trait ConstitutionalInvariants {
    /// Compute Lyapunov-like functional V(t) from canonical state.
    fn compute_lyapunov(&self, signal: &SignalState, energies: &[f64]) -> f64;

    /// Resource invariant update:
    /// e_ℓ(t+1) = e_ℓ(t) + γ_ℓ I_ℓ(t) - ψ_ℓ O_ℓ(t) - ω_ℓ L_ℓ(t),
    /// applying host-level constraints like Σ e_ℓ(t) ≤ E_max.
    fn update_and_check_energy(
        &self,
        prev_budget: &HostBudget,
        next_budget: &HostBudget,
    ) -> (f64, bool);

    /// Plasticity invariants: norm bounds on s_ij and restricted learning windows.
    fn check_plasticity(&self, prev: &NeuromorphicState, next: &NeuromorphicState) -> bool;

    /// Single-step invariant evaluation; returns a snapshot suitable for audit and attestation.
    fn evaluate_invariants(
        &self,
        prev: &NeuromorphicState,
        next: &NeuromorphicState,
        thermo: &ThermodynamicEnvelope,
        ml: &MlPassSchedule,
        bci_snap: &BciHostSnapshot,
        prev_v: f64,
    ) -> InvariantSnapshot {
        let (total_energy, energy_ok) =
            self.update_and_check_energy(&prev.host_budget, &next.host_budget);

        let v_next = self.compute_lyapunov(&next.signal, &next.host_budget.energy_channels());
        let delta_v = v_next - prev_v;

        let thresholds = BciSafetyThresholds::from_descriptors(
            thermo.clone(),
            ml.clone(),
            // reuse ReversalConditions from descriptor at policy layer
            ReversalConditions::default(),
        );
        let telemetry_ok = thresholds.snapshot_safe(bci_snap.clone());

        let plasticity_ok = self.check_plasticity(prev, next);

        InvariantSnapshot {
            lyapunov_v: v_next,
            delta_v,
            total_energy,
            energy_ok: energy_ok && telemetry_ok,
            lyapunov_ok: delta_v <= 0.0,
            plasticity_ok,
        }
    }
}

/// Helper extension for HostBudget to expose ordered energy channels.
pub trait EnergyChannels {
    fn energy_channels(&self) -> Vec<f64>;
}

impl EnergyChannels for HostBudget {
    fn energy_channels(&self) -> Vec<f64> {
        self.energy_state
            .iter()
            .map(|(_, e)| e.current_joules)
            .collect()
    }
}
