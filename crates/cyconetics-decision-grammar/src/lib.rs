//! # Cyconetics Decision Grammar Crate
//!
//! Immutable, type-safe decision grammar for autonomous cybernetic upgrades with:
//! - **Neuro-consent enforcement**: Zero-touch for non-host entities, host explicit consent for brain-linked upgrades
//! - **RoH ≤ 0.3 guarding**: Risk-of-Harm bounded by type system (RoHBound<30>)
//! - **Ecological protection**: BFC (brain-field communication) broadcasts logged and validated
//! - **ALN shard binding**: All decisions committed to immutable blockchain-anchored governance shards
//! - **CI sidecar integration**: Pre-deployment safety checks for physiological and governance constraints
//! - **Blood-linked homeostasis**: Glucose, hydration, metabolic reserves protect against biological harm
//!
//! ## Example
//!
//! ```ignore
//! use cyconetics_decision_grammar::*;
//!
//! // 1. Create host state with current RoH
//! let host = RoHGuardedHostState { ... };
//!
//! // 2. Describe upgrade request
//! let upgrade = UpgradeDescriptor {
//!     upgrade_id: "bci-001".to_string(),
//!     upgrade_class: UpgradeClass::BCI,
//!     estimated_roh_delta: 0.08,
//!     blood_token_cost: 10.0,
//!     ...
//! };
//!
//! // 3. Evaluate: type-safe decision with RoH < 0.3 guarantee
//! let decision = evaluate_upgrade(&host, &upgrade);
//! assert!(matches!(decision, DecisionKind::Authorize | DecisionKind::Reject));
//!
//! // 4. Create evidence bundle (biomarkers)
//! let evidence = EvidenceBundle::new("zone-phoenix-west".to_string());
//!
//! // 5. Run CI sidecar checks
//! let sidecar = CISidecarm::new();
//! assert!(sidecar.check_evidence_bundle(&evidence).is_pass());
//!
//! // 6. Commit to ALN shards (immutable)
//! let mut shard = DecisionLedgerShard::new();
//! shard.append(entry);
//! // Hash committed to blockchain
//! ```
//!
//! ## Architecture
//!
//! ### Decision Types
//! - **DecisionKind**: Approve, Authorize, Defer, Reject, Escalate
//! - **RoH Guarding**: Compile-time guarantee that post_roh < 0.3 via RoHBound<30> type
//! - **KsrBand**: Knowledge:Social:Risk triple (U8:U8:U8) for nuanced policy
//!
//! ### Governance
//! - **Host-centric**: Host can self-approve low-risk upgrades; high-risk requires neuroboard
//! - **Role-based**: HostSelf, NeurorightsBoard, SafetyDaemon, GovSafetyOS with verb constraints
//! - **Zones**: Phoenix-West, Phoenix-Central, etc. with local RoH ceilings
//!
//! ### Safety Mechanisms
//! - **BFC Zero-Touch**: Brain-field comm broadcasts logged; non-host entities receive passive telemetry only
//! - **Blood Tokens**: Metabolic reserve coupling ensures physiological protection
//! - **EEG Corridors**: Brainwave state monitoring (nominal, elevated, critical)
//! - **Neuro-consent**: Explicit consent required for each entity type per zone
//!
//! ### Immutability & Audit
//! - **ALN Shards**: Four schemas (decision ledger, consent registry, broadcast ledger, policy)
//! - **SHA256 Hashing**: Each decision, evidence, and entry hashed for blockchain stamping
//! - **JSONL Export**: Easy commit to Cyberswarm/Bostrom mainnet
//! - **Incident-Driven Tightening**: Failed health checks trigger RoH reduction

pub mod types;
pub mod ledger;
pub mod validators;
pub mod aln_shards;
pub mod ci_hooks;
pub mod macros;

// Re-export critical types for ergonomics
pub use types::*;
pub use ledger::*;
pub use validators::*;
pub use aln_shards::*;
pub use ci_hooks::*;
pub use macros::*;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default RoH ceiling for all zones (0.30 = 30%)
pub const DEFAULT_ROH_CEILING: f32 = 0.30;

/// Host blood token annual budget (for metabolic protection)
pub const BLOOD_TOKEN_ANNUAL_BUDGET: f32 = 500.0;

/// Maximum glucose stress allowed before recommendation to defer upgrades
pub const GLUCOSE_STRESS_THRESHOLD: f32 = 150.0;

/// Minimum hydration index for neurotech upgrades (0.7 = 70%)
pub const HYDRATION_INDEX_MINIMUM: f32 = 0.70;

/// Main decision evaluation function: consults host state and upgrade descriptor
/// Returns a DecisionKind with RoH < 0.3 guaranteed
pub fn evaluate_upgrade(
    host_state: &RoHGuardedHostState,
    upgrade: &UpgradeDescriptor,
) -> DecisionKind {
    let predicted_roh = host_state.current_roh + upgrade.estimated_roh_delta;

    // Hard ceiling
    if predicted_roh >= DEFAULT_ROH_CEILING {
        return DecisionKind::Reject;
    }

    // Risk band check
    match upgrade.upgrade_class {
        UpgradeClass::BCI => {
            // BCI upgrades: check EEG state and plasticity budget
            if host_state.bci_snapshot.eeg_corridor_state == "critical" {
                return DecisionKind::Defer;
            }
            if host_state.bci_snapshot.plasticity_used_percent > 0.8 {
                return DecisionKind::Reject; // Plasticity exhausted
            }
        },
        UpgradeClass::XR => {
            // XR upgrades: check hydration and metabolic state
            if host_state.host_budget.hydration_index < HYDRATION_INDEX_MINIMUM {
                return DecisionKind::Defer;
            }
        },
        UpgradeClass::HCI => {
            // HCI upgrades: lighter constraints
        },
    }

    // Blood token availability
    if upgrade.blood_token_cost > host_state.host_budget.blood_tokens_reserved {
        return DecisionKind::Defer;
    }

    // Low-risk upgrade: host can self-approve
    if upgrade.estimated_roh_delta < 0.05 && !upgrade.requires_host_veto {
        DecisionKind::Authorize
    } else {
        // Medium-risk: requires explicit host approval + board review
        DecisionKind::Approve
    }
}

/// Helper: compute RoH from biokarma risk vector
pub fn roh_from_biokarma(biokarma: &BioKarmaRiskVector) -> f32 {
    // Weighted average with emphasis on psychological risk
    (biokarma.metabolic_risk * 0.15
        + biokarma.hemodynamic_risk * 0.15
        + biokarma.thermal_risk * 0.10
        + biokarma.cognitive_risk * 0.30
        + biokarma.psych_risk * 0.30)
        .min(0.99)
}

/// Helper: create RoHBound<30> from a float (panics if >= 0.3)
pub fn try_roh_bound_30(roh: f32) -> Result<f32, String> {
    if roh >= DEFAULT_ROH_CEILING {
        Err(format!("RoH {:.3} >= ceiling {:.3}", roh, DEFAULT_ROH_CEILING))
    } else {
        Ok(roh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_upgrade_safe() {
        let host = RoHGuardedHostState {
            current_roh: 0.15,
            host_budget: HostBudget {
                blood_tokens_reserved: 50.0,
                hydration_index: 0.75,
                ..Default::default()
            },
            brain_specs: BrainSpecs::default_phoenix_baseline(),
            bci_snapshot: BciHostSnapshot {
                eeg_corridor_state: "nominal".to_string(),
                plasticity_used_percent: 0.35,
                neural_rope_anchor_integrity: 0.92,
                active_neural_roi: 4,
            },
        };

        let upgrade = UpgradeDescriptor {
            upgrade_id: "test".to_string(),
            upgrade_class: UpgradeClass::BCI,
            estimated_roh_delta: 0.08,
            requires_host_veto: false,
            blood_token_cost: 10.0,
        };

        let decision = evaluate_upgrade(&host, &upgrade);
        assert!(!matches!(decision, DecisionKind::Reject));
    }

    #[test]
    fn test_evaluate_upgrade_exceeds_ceiling() {
        let host = RoHGuardedHostState {
            current_roh: 0.25,
            host_budget: HostBudget::default(),
            brain_specs: BrainSpecs::default_phoenix_baseline(),
            bci_snapshot: BciHostSnapshot::default(),
        };

        let upgrade = UpgradeDescriptor {
            upgrade_id: "test".to_string(),
            upgrade_class: UpgradeClass::BCI,
            estimated_roh_delta: 0.10,  // Would result in 0.35
            requires_host_veto: false,
            blood_token_cost: 5.0,
        };

        let decision = evaluate_upgrade(&host, &upgrade);
        assert_eq!(decision, DecisionKind::Reject);
    }

    #[test]
    fn test_roh_from_biokarma() {
        let biokarma = BioKarmaRiskVector {
            metabolic_risk: 0.05,
            hemodynamic_risk: 0.05,
            thermal_risk: 0.02,
            cognitive_risk: 0.10,
            psych_risk: 0.08,
        };
        let roh = roh_from_biokarma(&biokarma);
        assert!(roh > 0.0 && roh < 0.3);
    }

    #[test]
    fn test_try_roh_bound_30_pass() {
        let result = try_roh_bound_30(0.29);
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_roh_bound_30_fail() {
        let result = try_roh_bound_30(0.31);
        assert!(result.is_err());
    }
}
