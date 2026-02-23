#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::session_guard::{session_guard_allows_retrieval, RetrievalMode, SessionGuardContext};
use cyber_metrics_core::{CyberRank, KnowledgeFactor};
use super::session_guard::{HostBudget, NeurorightsPolytope, RiskOfHarm};

/// Minimal CargoEnvDescriptor view for corridor safety.[file:22]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoEnvDescriptor {
    pub package_name: String,
    pub version: String,
    pub bioscale_corridor: String,
    pub offline_first: bool,
    pub browser_dependency: bool,
    pub host_did: String,
}

/// Hex-stamped evidence sequence attached to environment decisions.[file:24]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSequence {
    pub sequence_hex: Vec<String>, // at least 10 short hex strings
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionEnvGateInput {
    pub cargo_env: CargoEnvDescriptor,
    pub session_ctx: SessionGuardContext,
    pub evidence: EvidenceSequence,
}

/// Output: retrieval permission plus an audit-ready hex digest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionEnvGateDecision {
    pub allowed: bool,
    pub reason: String,
    pub audit_hex: String,
}

/// Hard constraints:
/// - offline_first == true
/// - browser_dependency == false (no reliance on Edge/Chrome/Comet security)[file:22]
/// - session_guard_allows_retrieval == true
/// - at least 10 evidence hex tags present
pub fn ingestion_env_gate(input: &IngestionEnvGateInput) -> IngestionEnvGateDecision {
    let (session_ok, session_log) = session_guard_allows_retrieval(&input.session_ctx);

    let offline_ok = input.cargo_env.offline_first && !input.cargo_env.browser_dependency;
    let evidence_ok = input.evidence.sequence_hex.len() >= 10;

    let allowed = session_ok && offline_ok && evidence_ok;

    // Derive a simple audit hex from booleans and counts (no hashes).
    let flags = format!(
        "{:x}{:x}{:x}",
        offline_ok as u8,
        evidence_ok as u8,
        session_ok as u8
    );
    let count_hex = format!("{:x}", input.evidence.sequence_hex.len());
    let audit_hex = format!("0x{}{}{}", flags, count_hex, "b7");

    let mut reason = String::new();
    if !offline_ok {
        reason.push_str("env_not_offline_first_or_browser_tainted;");
    }
    if !session_ok {
        reason.push_str("session_guard_rejected;");
    }
    if !evidence_ok {
        reason.push_str("insufficient_evidence_hex;");
    }
    if allowed && reason.is_empty() {
        reason.push_str("retrieval_only_ingestion_permitted");
    }

    IngestionEnvGateDecision {
        allowed,
        reason,
        audit_hex,
    }
}
