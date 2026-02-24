//! EEG.Math invariant kernel for Rust.Learn Cybernetics.
//!
//! This crate implements a three-layer invariant stack:
//!   raw EEG  -> F_EEG -> NeuromorphicState
//!   NeuromorphicState + HostBudget/BciHostSnapshot -> InvariantChecks
//!   NeuromorphicState + governance -> G_BCI Endpoint
//!
//! It is device-agnostic (pure Rust) but device-trustable,
//! because the same math can run in deviceless tests and
//! in TPM/TEE-wrapped binaries with audit logging.

use std::time::SystemTime;

use bioscale_upgrade_store::{
    EvidenceBundle,
    EvidenceTag,
    HostBudget,
    UpgradeDescriptor,
    UpgradeDecision,
};
use cyberswarm_neurostack::bci_host_snapshot::BciHostSnapshot;

/// Canonical EEG sample representation (already digitized from hardware).
/// This is the only "raw" surface that F_EEG sees.
#[derive(Clone, Debug)]
pub struct EegSample {
    /// Timestamp when sample was acquired.
    pub timestamp: SystemTime,
    /// Microvolt-scaled channels, already calibrated.
    pub microvolts: Vec<f32>,
}

/// Fixed-length window of EEG samples used for feature extraction.
#[derive(Clone, Debug)]
pub struct EegWindow {
    pub samples: Vec<EegSample>,
    /// Sampling rate in Hz.
    pub fs_hz: f32,
}

/// Neuromorphic / organic CPU state tuple:
/// n_i(t), s_ij(t), e_l(t), p_k(t)
#[derive(Clone, Debug)]
pub struct NeuromorphicState {
    /// Neural activations n_i(t) (e.g., band-limited powers / decoder activations).
    pub activations: Vec<f32>,
    /// Effective synaptic / coupling weights s_ij(t), flattened.
    pub couplings: Vec<f32>,
    /// Energy channels e_l(t) (Blood, Oxygen, RF, etc.), in joules-equivalent.
    pub energy_channels: Vec<f64>,
    /// Policy / governance parameters p_k(t) (Lagrange multipliers, thresholds).
    pub policy_params: Vec<f64>,
}

/// Configuration for energy dynamics invariants per energy channel.
#[derive(Clone, Debug)]
pub struct EnergyInvariantConfig {
    /// E_max global upper bound for sum_l e_l(t).
    pub e_max_total_joules: f64,
    /// Minimum slope for ΔE_total / Δt over equilibrium windows.
    pub min_total_energy_slope: f64,
    /// Per-channel gamma_l, psi_l, omega_l coefficients.
    pub gamma_in: Vec<f64>,
    pub psi_out: Vec<f64>,
    pub omega_loss: Vec<f64>,
}

/// Configuration for Lyapunov-like neural/energy functional V(t).
#[derive(Clone, Debug)]
pub struct LyapunovConfig {
    /// Coefficients c_i for neural activations.
    pub c_neural: Vec<f64>,
    /// Coefficients d_l for energy channels.
    pub d_energy: Vec<f64>,
    /// Whether to enforce V(t+1) - V(t) <= 0 in safety mode.
    pub enforce_non_increasing: bool,
}

/// Configuration for synaptic plasticity invariants.
#[derive(Clone, Debug)]
pub struct PlasticityConfig {
    /// Learning rate eta.
    pub eta: f64,
    /// Maximum allowed norm of s_ij.
    pub s_norm_max: f64,
}

/// Governance / policy invariant configuration.
#[derive(Clone, Debug)]
pub struct GovernanceConfig {
    /// If true, χ(t) must be 1 for any action to be allowed.
    pub require_compliance_bit: bool,
}

/// Aggregated configuration for EEG.Math invariants.
#[derive(Clone, Debug)]
pub struct EegMathInvariantConfig {
    pub energy: EnergyInvariantConfig,
    pub lyapunov: LyapunovConfig,
    pub plasticity: PlasticityConfig,
    pub governance: GovernanceConfig,
    /// Evidence bundle tying numeric choices to biophysics.
    pub evidence: EvidenceBundle,
}

/// Result of evaluating invariants at a single time step.
#[derive(Clone, Debug)]
pub struct InvariantEvaluation {
    pub v_t: f64,
    pub v_t_next: f64,
    pub v_non_increasing_ok: bool,
    pub energy_sum_ok: bool,
    pub energy_slope_ok: bool,
    pub synaptic_norm_ok: bool,
    /// χ(t) = 1 if all governance predicates are satisfied.
    pub compliance_bit: bool,
}

/// Endpoint intent produced by G_BCI before gating.
#[derive(Clone, Debug)]
pub struct BciIntent {
    /// Continuous intent vector u(t) (cursor velocity, probability logits, etc.).
    pub intent_vector: Vec<f32>,
    /// Proposed action label (e.g., "MotorAssist", "NoOp", "StimPatternId(...)").
    pub action_label: String,
}

/// Final gated endpoint (action + proof).
#[derive(Clone, Debug)]
pub struct BciEndpoint {
    pub allowed: bool,
    pub action_label: Option<String>,
    pub intent_vector: Option<Vec<f32>>,
    /// Proof payload tying decision back to invariants and evidence.
    pub proof: BciProof,
}

/// Minimal proof object binding invariants to bioscale/ALN evidence.
#[derive(Clone, Debug)]
pub struct BciProof {
    pub v_t: f64,
    pub v_t_next: f64,
    pub energy_sum_ok: bool,
    pub energy_slope_ok: bool,
    pub synaptic_norm_ok: bool,
    pub compliance_bit: bool,
    pub evidence_tags: Vec<EvidenceTag>,
}

/// Canonical feature map F_EEG: EEG window -> NeuromorphicState.
///
/// This is intentionally deterministic and device-agnostic.
/// Any hardware backend must produce the same EegWindow
/// to obtain identical NeuromorphicState.
pub fn f_eeg_to_state(win: &EegWindow) -> NeuromorphicState {
    // 1. Simple scale normalization: subtract median, divide by MAD per channel.
    let channels = if win.samples.is_empty() {
        0
    } else {
        win.samples[0].microvolts.len()
    };
    let mut activations = Vec::with_capacity(channels);

    for ch in 0..channels {
        let mut values = Vec::with_capacity(win.samples.len());
        for s in &win.samples {
            values.push(s.microvolts[ch]);
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = values.len() / 2;
        let median = values.get(mid).copied().unwrap_or(0.0);

        let mut mad_vals = Vec::with_capacity(values.len());
        for v in &values {
            mad_vals.push((v - median).abs());
        }
        mad_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mad = mad_vals.get(mid).copied().unwrap_or(1e-6).max(1e-6);

        // Aggregate normalized variance as a proxy "activation".
        let mut var_acc = 0.0;
        for s in &win.samples {
            let z = (s.microvolts[ch] - median) / mad;
            var_acc += z * z;
        }
        let act = (var_acc / (win.samples.len() as f32)).sqrt();
        activations.push(act);
    }

    // 2. Placeholder couplings: identity-like weights based on channels.
    // In a full implementation, this would be derived from connectivity metrics.
    let mut couplings = Vec::new();
    for i in 0..channels {
        for j in 0..channels {
            if i == j {
                couplings.push(1.0);
            } else {
                couplings.push(0.0);
            }
        }
    }

    // 3. Energy channels: map simple band power proxies into shared numeric contract.
    // Here we just compute total activation energy and split into two channels:
    // [Blood+Oxygen, RF/Other], to be re-mapped by the caller if needed.
    let total_act_energy: f64 = activations
        .iter()
        .map(|a| (*a as f64) * (*a as f64))
        .sum();
    let energy_channels = vec![0.8 * total_act_energy, 0.2 * total_act_energy];

    // 4. Policy params initially empty; governance layer fills them.
    let policy_params = Vec::new();

    NeuromorphicState {
        activations,
        couplings,
        energy_channels,
        policy_params,
    }
}

/// Compute Lyapunov-like functional
/// V(t) = sum_i c_i n_i^2 + sum_l d_l e_l^2
fn compute_lyapunov(state: &NeuromorphicState, cfg: &LyapunovConfig) -> f64 {
    let mut v = 0.0;

    for (i, n) in state.activations.iter().enumerate() {
        let c = cfg
            .c_neural
            .get(i)
            .copied()
            .unwrap_or(*cfg.c_neural.last().unwrap_or(&1.0));
        v += c * (*n as f64) * (*n as f64);
    }

    for (l, e) in state.energy_channels.iter().enumerate() {
        let d = cfg
            .d_energy
            .get(l)
            .copied()
            .unwrap_or(*cfg.d_energy.last().unwrap_or(&1.0));
        v += d * (*e) * (*e);
    }

    v
}

/// Compute squared norm of synaptic weights ||s||^2.
fn synaptic_norm_sq(couplings: &[f32]) -> f64 {
    couplings
        .iter()
        .map(|w| (*w as f64) * (*w as f64))
        .sum::<f64>()
}

/// Evaluate energy invariants:
///   e_l(t+1) = e_l(t) + gamma_l I_l - psi_l O_l - omega_l L_l
///   sum_l e_l(t) <= E_max
///   ΔE_total/Δt > 0 over equilibrium windows (approx. here by single step).
fn evaluate_energy_invariants(
    state_t: &NeuromorphicState,
    state_t1: &NeuromorphicState,
    cfg: &EnergyInvariantConfig,
) -> (bool, bool) {
    let sum_t: f64 = state_t.energy_channels.iter().copied().sum();
    let sum_t1: f64 = state_t1.energy_channels.iter().copied().sum();
    let energy_sum_ok = sum_t1 <= cfg.e_max_total_joules + 1e-9;

    let delta_e = sum_t1 - sum_t;
    let energy_slope_ok = delta_e >= cfg.min_total_energy_slope;

    (energy_sum_ok, energy_slope_ok)
}

/// Evaluate all invariants at a step (t -> t+1).
pub fn evaluate_invariants_step(
    state_t: &NeuromorphicState,
    state_t1: &NeuromorphicState,
    cfg: &EegMathInvariantConfig,
) -> InvariantEvaluation {
    let v_t = compute_lyapunov(state_t, &cfg.lyapunov);
    let v_t1 = compute_lyapunov(state_t1, &cfg.lyapunov);

    let v_non_increasing_ok = if cfg.lyapunov.enforce_non_increasing {
        v_t1 <= v_t + 1e-9
    } else {
        true
    };

    let (energy_sum_ok, energy_slope_ok) =
        evaluate_energy_invariants(state_t, state_t1, &cfg.energy);

    let syn_norm_sq = synaptic_norm_sq(&state_t1.couplings);
    let synaptic_norm_ok = syn_norm_sq.sqrt() <= cfg.plasticity.s_norm_max + 1e-9;

    // Governance compliance bit χ(t): here we require that all hard invariants hold.
    let compliance_bit = if cfg.governance.require_compliance_bit {
        v_non_increasing_ok && energy_sum_ok && energy_slope_ok && synaptic_norm_ok
    } else {
        true
    };

    InvariantEvaluation {
        v_t,
        v_t_next: v_t1,
        v_non_increasing_ok,
        energy_sum_ok,
        energy_slope_ok,
        synaptic_norm_ok,
        compliance_bit,
    }
}

/// Device-independent compliance predicate χ(t) = 1{C_p(S(t), a(t)) = 1}.
fn compute_compliance_bit(eval: &InvariantEvaluation, cfg: &GovernanceConfig) -> bool {
    if !cfg.require_compliance_bit {
        return true;
    }
    eval.v_non_increasing_ok
        && eval.energy_sum_ok
        && eval.energy_slope_ok
        && eval.synaptic_norm_ok
}

/// Neuromorphic decoding: map NeuromorphicState to a continuous intent vector u(t).
///
/// This function is intentionally simple and deterministic; more complex decoders
/// can be plugged in as long as they preserve the same signature.
pub fn decode_intent(state: &NeuromorphicState) -> BciIntent {
    // Example: first two activations define a 2D intent vector.
    let u0 = state.activations.get(0).copied().unwrap_or(0.0);
    let u1 = state.activations.get(1).copied().unwrap_or(0.0);

    BciIntent {
        intent_vector: vec![u0, u1],
        action_label: "GenericIntent".to_string(),
    }
}

/// Cybernetic gating under energy, Lyapunov, and governance constraints.
///
/// Endpoint(t) =
///   a(t)                 if χ(t) = 1 and energy/Lyapunov constraints hold
///   No-op (safe fallback) otherwise.
pub fn g_bci_endpoint(
    state_t: &NeuromorphicState,
    state_t1: &NeuromorphicState,
    cfg: &EegMathInvariantConfig,
    host_budget: &HostBudget,
    host_snapshot: &BciHostSnapshot,
    proposed_action: &str,
) -> BciEndpoint {
    let eval = evaluate_invariants_step(state_t, state_t1, cfg);
    let chi = compute_compliance_bit(&eval, &cfg.governance);

    // Host-level gating: require that host budgets are non-negative and
    // that snapshot is within safe envelope (leveraging existing logic).
    let host_energy_ok = host_budget.remaining_energy_joules > 0.0;
    let host_protein_ok = host_budget.remaining_protein_grams >= 0.0;

    // Basic telemetry-based safety: reuse BciHostSnapshot semantics.
    let temp_ok = host_snapshot.core_temp_c <= 37.8
        && host_snapshot.local_temp_c <= (host_snapshot.core_temp_c + 0.5);
    let pain_ok = host_snapshot.pain_vas <= 3.0;
    let infl_ok = host_snapshot.inflammation_score <= 2.0;

    let all_ok = chi && host_energy_ok && host_protein_ok && temp_ok && pain_ok && infl_ok;

    let intent = decode_intent(state_t1);

    let proof = BciProof {
        v_t: eval.v_t,
        v_t_next: eval.v_t_next,
        energy_sum_ok: eval.energy_sum_ok,
        energy_slope_ok: eval.energy_slope_ok,
        synaptic_norm_ok: eval.synaptic_norm_ok,
        compliance_bit: eval.compliance_bit,
        evidence_tags: cfg.evidence.sequences.clone(),
    };

    if all_ok {
        BciEndpoint {
            allowed: true,
            action_label: Some(proposed_action.to_string()),
            intent_vector: Some(intent.intent_vector),
            proof,
        }
    } else {
        BciEndpoint {
            allowed: false,
            action_label: None,
            intent_vector: None,
            proof,
        }
    }
}

/// Bridge to bioscale upgrade evaluation: convert a BCI endpoint into an
/// UpgradeDecision-like guard. This lets you reuse existing router logic
/// without bypassing bioscale/ALN envelopes.
pub fn evaluate_bci_endpoint_as_upgrade(
    endpoint: &BciEndpoint,
    desc: &UpgradeDescriptor,
    host: &HostBudget,
) -> UpgradeDecision {
    if !endpoint.allowed {
        return UpgradeDecision::Denied {
            reason: "EEG.Math invariants or host envelopes not satisfied".to_string(),
        };
    }

    // Delegate to bioscale store semantics: caller will use this together
    // with an actual BioscaleUpgradeStore implementation.
    let required_joules: f64 = desc
        .energy_costs
        .iter()
        .map(|e| e.joules)
        .sum();

    if required_joules > host.remaining_energy_joules {
        return UpgradeDecision::Denied {
            reason: "Insufficient remaining energy budget for BCI endpoint".to_string(),
        };
    }

    UpgradeDecision::Approved {
        scheduled_at: SystemTime::now(),
        expected_completion: SystemTime::now(),
    }
}
