use serde::{Serialize, Deserialize};
use neurorights_firewall::{NeurorightsProfile, HasNeurorightsProfile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub user_did: String,
    pub aln: String,
    pub bostrom_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub trace_source: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Governance {
    pub eibon_label: String,
    pub policy_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    RetrieveKnowledge,
    PlanUpgrade,
    ScoreAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEnvelope {
    pub trace_id: String,
    pub intent: Intent,
    pub args: serde_json::Value,
    pub security_level: SecurityLevel,
    pub identity: Identity,
    pub provenance: Provenance,
    pub governance: Governance,
    pub neurorights_profile: NeurorightsProfile,
}

impl HasNeurorightsProfile for PromptEnvelope {
    fn set_neurorights_profile(&mut self, profile: NeurorightsProfile) {
        self.neurorights_profile = profile;
    }
}

impl PromptEnvelope {
    pub fn with_citizen_neurorights(
        trace_id: impl Into<String>,
        intent: Intent,
        args: serde_json::Value,
        security_level: SecurityLevel,
        identity: Identity,
        provenance: Provenance,
        governance: Governance,
        anchor: impl Into<String>,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            intent,
            args,
            security_level,
            identity,
            provenance,
            governance,
            neurorights_profile: NeurorightsProfile::citizen_v1(anchor),
        }
    }
}
