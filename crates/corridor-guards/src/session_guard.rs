#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use cyber_metrics_core::{CyberRank, KnowledgeFactor}; // from previous crate

/// Host-local energy/protein envelope.[file:21]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostBudget {
    pub remaining_energy_j: f64,
    pub daily_energy_j: f64,
    pub remaining_protein_g: f64,
    pub daily_protein_g: f64,
}

/// Neurorights polytope defined in the host's ALN shard.[file:14][file:21]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurorightsPolytope {
    pub cognitive_liberty_min: f64,
    pub mental_privacy_min: f64,
    pub authorship_min: f64,
}

/// Risk-of-Harm snapshot in [0,1].[file:21][file:20]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RiskOfHarm {
    pub roh_value: f64,
}

/// Retrieval mode: strictly non-mutative by construction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetrievalMode {
    RetrievalOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionGuardContext {
    pub host_budget: HostBudget,
    pub neurorights_polytope: NeurorightsPolytope,
    pub risk_of_harm: RiskOfHarm,
    pub cyberrank: CyberRank,
    pub knowledge_factor: KnowledgeFactor,
    pub retrieval_mode: RetrievalMode,
    /// Hex-stamped corridor identifier for audit.[file:21][file:31]
    pub corridor_hexstamp: String,
}

/// Hex log entry for each retrieval decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalHexLog {
    pub corridor_hexstamp: String,
    pub decision_hex: String,
    pub roh_value: f64,
    pub cyberrank_safety: f64,
}

/// Hard invariants:
/// - RoH <= 0.3
/// - neurorights dimensions of CyberRank >= polytope minima
/// - retrieval_mode == RetrievalOnly
/// - HostBudget not exhausted (energy/protein headroom)[file:21][file:31]
pub fn session_guard_allows_retrieval(ctx: &SessionGuardContext) -> (bool, RetrievalHexLog) {
    let roh_ok = ctx.risk_of_harm.roh_value <= 0.3;
    let nr_ok = ctx.cyberrank.neurorights >= ctx.neurorights_polytope.cognitive_liberty_min
        && ctx.cyberrank.neurorights >= ctx.neurorights_polytope.mental_privacy_min
        && ctx.cyberrank.neurorights >= ctx.neurorights_polytope.authorship_min;
    let retrieval_only = ctx.retrieval_mode == RetrievalMode::RetrievalOnly;

    let energy_frac = if ctx.host_budget.daily_energy_j > 0.0 {
        ctx.host_budget.remaining_energy_j / ctx.host_budget.daily_energy_j
    } else {
        0.0
    };
    let protein_frac = if ctx.host_budget.daily_protein_g > 0.0 {
        ctx.host_budget.remaining_protein_g / ctx.host_budget.daily_protein_g
    } else {
        0.0
    };
    let budget_ok = energy_frac >= 0.05 && protein_frac >= 0.05;

    let allowed = roh_ok && nr_ok && retrieval_only && budget_ok;

    // Derive a deterministic hex decision tag from the booleans (no hashing).[file:21]
    let bit_pattern = format!(
        "{:x}{:x}{:x}{:x}",
        roh_ok as u8,
        nr_ok as u8,
        retrieval_only as u8,
        budget_ok as u8
    );
    let decision_hex = format!("0x{}{}", bit_pattern, "a3");

    let log = RetrievalHexLog {
        corridor_hexstamp: ctx.corridor_hexstamp.clone(),
        decision_hex,
        roh_value: ctx.risk_of_harm.roh_value,
        cyberrank_safety: ctx.cyberrank.safety,
    };

    (allowed, log)
}
