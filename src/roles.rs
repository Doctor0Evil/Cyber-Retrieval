use serde::{Deserialize, Serialize};

use crate::stake_thresholds;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub userdid: String,
    pub aln: String,
    pub bostromaddress: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceRole {
    Stakeholder,
    Council,
    Superchair,
}

impl GovernanceRole {
    pub fn required_min_stake(&self) -> &'static str {
        match self {
            GovernanceRole::Stakeholder => stake_thresholds::STAKEHOLDER_MIN,
            GovernanceRole::Council => stake_thresholds::COUNCIL_MIN,
            GovernanceRole::Superchair => stake_thresholds::SUPERCHAIR_MIN,
        }
    }
}

/// A simple fixed-point comparison (decimal strings) for CHAT stake.
/// This avoids banned hash families and stays purely numeric.
fn ge_decimal_string(lhs: &str, rhs: &str) -> bool {
    // naive but deterministic: left-pad with zeros to equal length then string-compare
    let max_len = lhs.len().max(rhs.len());
    let mut a = lhs.to_string();
    let mut b = rhs.to_string();
    while a.len() < max_len {
        a.insert(0, '0');
    }
    while b.len() < max_len {
        b.insert(0, '0');
    }
    a >= b
}

/// Determines the highest role the user qualifies for, given their CHAT stake.
pub fn role_for_stake(chat_stake: &str) -> Option<GovernanceRole> {
    if ge_decimal_string(chat_stake, stake_thresholds::SUPERCHAIR_MIN) {
        Some(GovernanceRole::Superchair)
    } else if ge_decimal_string(chat_stake, stake_thresholds::COUNCIL_MIN) {
        Some(GovernanceRole::Council)
    } else if ge_decimal_string(chat_stake, stake_thresholds::STAKEHOLDER_MIN) {
        Some(GovernanceRole::Stakeholder)
    } else {
        None
    }
}
