#![forbid(unsafe_code)]

use std::time::SystemTime;

use governance_core::roles::GovernanceRole;

/// High‑level kind of governance action (for analytics + continuity).
#[derive(Clone, Debug)]
pub enum GovernanceActionKind {
    WebsitePagePublish,
    WebsitePageReview,
    ChatStakeSpend,
    RoleSuccessionCheck,
    Custom(String),
}

/// Subset of the full PromptEnvelope relevant to governance continuity.
/// The full definition lives in your neurorights firewall crate; this is a
/// local alias that stays in sync via shared modules.
#[derive(Clone, Debug)]
pub struct PromptEnvelope {
    pub did: String,
    pub aln_scope: String,
    pub bostrom_address: String,
    pub eibon_label: String,
    pub hex_stamp: String,
    pub captured_at: SystemTime,
    // Optional term metadata per role, derived from governance.totem.superposition.v1.
    pub term_end_superchair: Option<SystemTime>,
    pub term_end_council: Option<SystemTime>,
}

impl PromptEnvelope {
    pub fn role_term_end(&self, role: &GovernanceRole) -> Option<SystemTime> {
        match role {
            GovernanceRole::Superchair => self.term_end_superchair,
            GovernanceRole::Council => self.term_end_council,
            GovernanceRole::Proposer => None,
        }
    }

    pub fn eibon_label(&self) -> &str {
        &self.eibon_label
    }
}
