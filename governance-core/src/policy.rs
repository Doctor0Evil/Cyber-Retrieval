#![forbid(unsafe_code)]

use crate::roles::{GovernanceRole, StakeSnapshot};

/// Static, ALN-mirrored thresholds (wire these to ALN later).
#[derive(Clone, Debug)]
pub struct RoleThresholds {
    pub superchair_min_chat: u128,
    pub council_min_chat: u128,
    pub proposer_min_chat: u128,
    pub min_contribution_index: u32,
}

/// Neurorights and risk bounds for governance logic.
#[derive(Clone, Debug)]
pub struct GovernanceSafetyProfile {
    pub risk_of_harm_ceiling: f32, // must stay ≤ 0.3
    pub no_inner_state_scoring: bool,
    pub no_neurocoercion: bool,
    pub revocable_at_will: bool,
    pub ecosocial_reporting_required: bool,
}

impl GovernanceSafetyProfile {
    pub const fn default_profile() -> Self {
        Self {
            risk_of_harm_ceiling: 0.08,
            no_inner_state_scoring: true,
            no_neurocoercion: true,
            revocable_at_will: true,
            ecosocial_reporting_required: true,
        }
    }
}

impl RoleThresholds {
    pub const fn default_thresholds() -> Self {
        Self {
            superchair_min_chat: 10_000,
            council_min_chat: 1_000,
            proposer_min_chat: 100,
            min_contribution_index: 1,
        }
    }

    pub fn eligible_roles(&self, stake: &StakeSnapshot) -> Vec<GovernanceRole> {
        let mut roles = Vec::new();

        if stake.chat >= self.superchair_min_chat
            && stake.contribution_index >= self.min_contribution_index
        {
            roles.push(GovernanceRole::Superchair);
        }
        if stake.chat >= self.council_min_chat
            && stake.contribution_index >= self.min_contribution_index
        {
            roles.push(GovernanceRole::Council);
        }
        if stake.chat >= self.proposer_min_chat
            && stake.contribution_index >= self.min_contribution_index
        {
            roles.push(GovernanceRole::Proposer);
        }

        roles
    }
}
