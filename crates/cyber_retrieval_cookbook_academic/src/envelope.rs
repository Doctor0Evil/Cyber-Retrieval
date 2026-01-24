use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{KsrTriple, RetrievalIntent};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Domain {
    DcmHciDesign,
    XrGridPolicy,
    RustWiring,
    DidRegistry,
    AcademicKnowledge,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum XrZone {
    Phoenix,
    SanJolla,
    Eco,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllowedCodeActions {
    pub allow_code_synthesis: bool,
    pub allow_manifest_templates: bool,
    pub retrieval_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEnvelope {
    pub trace_id: String,
    pub prompt_text: String,
    pub intent: RetrievalIntent,
    pub domain: Domain,
    pub xr_zone: XrZone,
    pub ksr_estimate: KsrTriple,
    pub allowed_code_actions: AllowedCodeActions,
    pub created_at: DateTime<Utc>,
}

impl PromptEnvelope {
    pub fn new(
        trace_id: impl Into<String>,
        prompt_text: impl Into<String>,
        intent: RetrievalIntent,
        domain: Domain,
        xr_zone: XrZone,
        ksr_estimate: KsrTriple,
        allowed_code_actions: AllowedCodeActions,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            prompt_text: prompt_text.into(),
            intent,
            domain,
            xr_zone,
            ksr_estimate,
            allowed_code_actions,
            created_at,
        }
    }
}
