use bioscale_fairness_core::ResponsibilityScalar;
use bioscale_upgrade_store::EvidenceBundle;

#[derive(Clone, Debug)]
pub struct CybernanoShardRow {
    pub shard_id: String,
    pub rohvalue: f64,
    pub lifeforcescalar: f64,
    pub ecook: bool,
    pub responsibility_r: ResponsibilityScalar,
    pub evidence: EvidenceBundle,
    pub neurorights_flags: u32,
}

impl CybernanoShardRow {
    pub fn mark_ecologically_ok(&mut self, verdict: &crate::KarmaVerdict) {
        self.ecook = verdict.is_karma_admissible;
        self.responsibility_r = verdict.post_r;
    }
}

// filename: crates/neuromorph-audit-particles/src/lib.rs

use bioscale_fairness_core::ResponsibilityScalar;

#[derive(Clone, Debug)]
pub struct NeuromorphEvolutionAuditParticle {
    pub particle_id: String,
    pub host_did: String,
    pub model_id: String,
    pub pre_action_r: ResponsibilityScalar,
    pub post_action_r: ResponsibilityScalar,
    pub delta_r: f64,
    pub is_karma_admissible: bool,
    pub roh_before: f64,
    pub roh_after: f64,
    pub sovereignty_flags: u32,
    pub eco_flags: u32,
}
