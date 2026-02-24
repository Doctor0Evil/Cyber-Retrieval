use std::time::SystemTime;
use bioscale_upgrade_store::{
    HostBudget, EnergyType, EnergyCost, ThermodynamicEnvelope, MlPassSchedule,
    ReversalConditions, EvidenceBundle, UpgradeDescriptor,
};
use cyberswarm_neurostack::{
    BciHostSnapshot, BciSafetyThresholds, NeuralControllerState,
};
use phoenix_aln_particles::{ALNComplianceParticle, ComplianceVerdict};
use serde::{Serialize, Deserialize};

/// 1. Frozen EEG feature map state space x(t)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignalState {
    /// Canonical neural activation vector n_i(t) (e.g. bandpower / decoder channels).
    pub neural_activations: Vec<f32>,
    /// Effective coupling / connectivity metrics s_ij(t) (flattened).
    pub couplings: Vec<f32>,
    /// Energy channels e_ℓ(t) mapped to EnergyType (Blood, Oxygen, RF, Battery, etc.).
    pub energies: Vec<f64>,
    /// Policy / Lagrange parameters p_k(t) (e.g. corridor gains, jsafe, RoH, ROD).
    pub policy_params: Vec<f64>,
}

/// Deterministic, frozen EEG feature map.
/// All implementations (deviceless or device-trusted) must be bit-identical for the same input.
pub trait FrozenEegMap {
    /// Deterministic mapping from raw biosignal window to SignalState.
    /// Normalization and reference (MAD, CAR/Laplacian, windowing) are fixed by spec.
    fn f_eeg(&self, raw_window: &[f32]) -> SignalState;
}

/// 2. Neuromorphic state x(t) + controller internals.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuromorphicState {
    /// Canonical EEG-derived state x(t).
    pub signal: SignalState,
    /// Internal neural controller state (weights, traces, mode flags).
    pub controller: NeuralControllerState,
    /// Evidence-backed resource ledger for all EnergyType channels.
    pub host_budget: HostBudget,
    /// Time index for this state.
    pub t: SystemTime,
}

/// 2a. Lyapunov- and envelope-based invariants over NeuromorphicState.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantSnapshot {
    /// Lyapunov-like scalar V(t) = Σ c_i n_i^2 + Σ d_ℓ e_ℓ^2.
    pub lyapunov_v: f64,
    /// ΔV = V(t+1) - V(t).
    pub delta_v: f64,
    /// Sum of energy channels.
    pub total_energy: f64,
    /// Whether all energy/resource invariants hold at this step.
    pub energy_ok: bool,
    /// Whether Lyapunov monotonicity (non-increasing in safety modes) holds.
    pub lyapunov_ok: bool,
    /// Whether plasticity bounds ‖s_ij‖ ≤ S_max etc. are satisfied.
    pub plasticity_ok: bool,
}

/// 3. Governance verdict and sovereign endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceVerdict {
    /// Composite predicate Cp(S(t), a(t)) – did this candidate action pass policy?
    pub compliance_predicate_pass: bool,
    /// Binary compliance bit χ(t).
    pub compliance_bit: bool,
    /// ALN particle carrying neurorights, consent, and regulatory bindings.
    pub aln_particle: ALNComplianceParticle,
    /// Full neuromorphic and invariant state at decision time for audit.
    pub invariants: InvariantSnapshot,
}

/// A Sovereign Endpoint is the only object allowed to cross the CyberneticEcosystem boundary.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SovereignEndpoint {
    /// Host-local monotonic identifier for this endpoint.
    pub endpoint_id: String,
    /// Candidate action a(t) – motor command, stim profile, API call, etc.
    pub action_payload: Vec<u8>,
    /// Governance verdict at the moment of emission.
    pub governance: GovernanceVerdict,
    /// Hash of pre- and post-host state (BciHostSnapshot, HostBudget, BrainSpecs, etc.).
    pub state_hash: [u8; 32],
    /// Time of emission in host clock.
    pub emitted_at: SystemTime,
}
