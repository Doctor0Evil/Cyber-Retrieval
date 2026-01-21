#![forbid(unsafe_code)]

use crate::roles::{GovernanceRole, StakeSnapshot};

/// Canonical ALN ids for governance bindings.
pub const ALN_ASSET_CHAT_STAKE_V1: &str = "asset.chat.stake.v1";
pub const ALN_GOV_CHAT_WEBSITE_V1: &str = "governance.chat.website.v1";
pub const ALN_GOV_TOTEM_SUPERPOSITION_V1: &str = "governance.totem.superposition.v1";

/// Raw, ALN-derived governance thresholds for one policy scope.
/// These fields are meant to be generated from ALN, not hand-edited.
#[derive(Clone, Debug)]
pub struct AlnGovernanceShard {
    pub id: String,
    pub version: String,
    pub anchor: String, // DID / registry-chain pointer

    // asset.chat.stake.v1: stake thresholds in CHAT, CY, ZEN, LIFEFORCE units.
    pub min_chat_for_superchair: u128,
    pub min_chat_for_council: u128,
    pub min_chat_for_proposer: u128,

    pub min_cy_for_superchair: u128,
    pub min_zen_for_council: u128,
    pub min_lifeforce_for_superchair: u128,

    // governance.chat.website.v1: contribution / authorship indices.
    pub min_contribution_index: u32,
    pub min_audit_trail_depth: u32,

    // governance.totem.superposition.v1: extra constraints for superchair seats.
    pub require_registry_chain_presence: bool,
    pub require_hex_stamped_profile: bool,
}

impl AlnGovernanceShard {
    /// Fallback profile for local testing; in production this should be
    /// fully generated from the ALN registry via build.rs.
    pub fn testing_defaults() -> Self {
        Self {
            id: ALN_ASSET_CHAT_STAKE_V1.to_string(),
            version: "0.1-testing".to_string(),
            anchor: "did:aln:testing-governance-shard".to_string(),
            min_chat_for_superchair: 10_000,
            min_chat_for_council: 1_000,
            min_chat_for_proposer: 100,
            min_cy_for_superchair: 0,
            min_zen_for_council: 0,
            min_lifeforce_for_superchair: 0,
            min_contribution_index: 1,
            min_audit_trail_depth: 1,
            require_registry_chain_presence: true,
            require_hex_stamped_profile: true,
        }
    }
}

/// Role thresholds now act as a pure adapter over an ALN shard.
#[derive(Clone, Debug)]
pub struct RoleThresholds {
    pub shard: AlnGovernanceShard,
}

impl RoleThresholds {
    pub fn from_aln_shard(shard: AlnGovernanceShard) -> Self {
        Self { shard }
    }

    pub fn eligible_roles(&self, stake: &StakeSnapshot) -> Vec<GovernanceRole> {
        let mut roles = Vec::new();
        let s = &self.shard;

        // Superchair: high CHAT stake + contribution index, plus optional CY / Lifeforce.
        if stake.chat >= s.min_chat_for_superchair
            && stake.contribution_index >= s.min_contribution_index
            && stake.cy >= s.min_cy_for_superchair
            && stake.lifeforce >= s.min_lifeforce_for_superchair
        {
            roles.push(GovernanceRole::Superchair);
        }

        // Council: mid-tier CHAT + either ZEN stake or contribution index.
        if stake.chat >= s.min_chat_for_council
            && (stake.contribution_index >= s.min_contribution_index
                || stake.zen >= s.min_zen_for_council)
        {
            roles.push(GovernanceRole::Council);
        }

        // Proposer: lowest threshold, essentially “can open an Eibon proposal”.
        if stake.chat >= s.min_chat_for_proposer
            && stake.contribution_index >= s.min_contribution_index
        {
            roles.push(GovernanceRole::Proposer);
        }

        roles
    }
}
