use serde::{Deserialize, Serialize};

use neurorights_core::{NeurorightsBound, NeurorightsEnvelope};
use neurorights_firewall::PromptEnvelope;

use crate::roles::{role_for_stake, GovernanceRole, Identity};
use crate::risk::{RiskEnvelope, RiskError};
use crate::governance_constraints;

/// A normalized, neurorights-bound envelope for website governance actions.
pub type WebsiteGovEnvelope =
    NeurorightsBound<PromptEnvelope<WebsiteGovArgs>, NeurorightsEnvelope>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteGovArgs {
    pub identity: Identity,
    pub chat_stake_decimal: String,
    pub action: WebsiteAction,
    pub page_id: String,
    pub section_id: Option<String>,
    pub eibonlabel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebsiteAction {
    ProposePage,
    ReviewPage,
    PublishPage,
    ReviewSection,
    PublishSection,
    UpdateSitePolicy,
    RollbackSitePolicy,
    VetoChange,
}

/// Entry point for neurorights-bound website governance.
/// This is designed to be called only from routers that already required
/// NeurorightsBound<PromptEnvelope<WebsiteGovArgs>, NeurorightsEnvelope>.
pub fn handle_website_governance(
    env: WebsiteGovEnvelope,
    risk_envelope: RiskEnvelope,
) -> Result<GovernanceDecision, GovernanceError> {
    if !governance_constraints::NEURORIGHTS_REQUIRED {
        return Err(GovernanceError::NeurorightsConstraintMissing);
    }

    risk_envelope.validate()?;

    if risk_envelope.risk_of_harm > governance_constraints::RISK_OF_HARM_CEILING {
        return Err(GovernanceError::RiskTooHigh {
            roh: risk_envelope.risk_of_harm,
            ceiling: governance_constraints::RISK_OF_HARM_CEILING,
        });
    }

    let args = env.inner().args.clone(); // PromptEnvelope<WebsiteGovArgs>.args
    let role = role_for_stake(&args.chat_stake_decimal)
        .ok_or(GovernanceError::InsufficientStake)?;

    enforce_permissions(role, &args.action)?;

    Ok(GovernanceDecision {
        allowed: true,
        role,
        page_id: args.page_id,
        section_id: args.section_id,
        eibonlabel: args.eibonlabel,
        risk_envelope,
    })
}

fn enforce_permissions(role: GovernanceRole, action: &WebsiteAction) -> Result<(), GovernanceError> {
    use GovernanceRole::*;
    use WebsiteAction::*;

    let allowed = match (role, action) {
        (Stakeholder, ProposePage) => true,
        (Stakeholder, _) => false,

        (Council, ProposePage | ReviewPage | ReviewSection) => true,
        (Council, _) => false,

        (Superchair, ProposePage
            | ReviewPage
            | PublishPage
            | ReviewSection
            | PublishSection
            | UpdateSitePolicy
            | RollbackSitePolicy
            | VetoChange) => true,
    };

    if allowed {
        Ok(())
    } else {
        Err(GovernanceError::PermissionDenied { role, action: format!("{:?}", action) })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceDecision {
    pub allowed: bool,
    pub role: GovernanceRole,
    pub page_id: String,
    pub section_id: Option<String>,
    pub eibonlabel: String,
    pub risk_envelope: RiskEnvelope,
}

#[derive(Debug, thiserror::Error)]
pub enum GovernanceError {
    #[error("neurorights constraints required but not enabled")]
    NeurorightsConstraintMissing,

    #[error("risk-of-harm {roh} exceeds governance ceiling {ceiling}")]
    RiskTooHigh { roh: f64, ceiling: f64 },

    #[error("insufficient CHAT stake for any governance role")]
    InsufficientStake,

    #[error("role {role:?} has no permission for action {action}")]
    PermissionDenied { role: GovernanceRole, action: String },

    #[error(transparent)]
    Risk(#[from] RiskError),
}
