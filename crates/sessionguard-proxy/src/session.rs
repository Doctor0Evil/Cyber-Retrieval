#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Chat,
    Stakeholder,
    Governance,
    Observer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NeurorightsFlags {
    pub cognitive_liberty: bool,
    pub mental_privacy: bool,
    pub mental_integrity: bool,
    pub augmentation_continuity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecureChannelProfile {
    pub dns_fail_closed: bool,
    pub doh_pinned: bool,
    pub tls_pinned: bool,
    pub browserless: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionToken {
    pub host_did: String,
    pub bostrom_addr_primary: String,
    pub roles: Vec<Role>,
    pub roh_leq_03: bool,
    pub expiry_utc: String,
    pub device_fingerprint: String,
    pub secure_channel: SecureChannelProfile,
    pub neurorights: NeurorightsFlags,
    pub hex_stamp: String,
}

#[derive(Debug)]
pub enum SessionGuardError {
    InvalidEnv(&'static str),
    Expired,
    RohViolation,
    NeurorightsViolation(&'static str),
}

#[derive(Debug, Clone)]
pub struct SessionGuard {
    token: SessionToken,
}

impl SessionGuard {
    /// Mandatory constructor: if this fails, the request must not be forwarded.
    pub fn new(
        token: SessionToken,
        observed_device_fingerprint: &str,
        observed_secure_channel: &SecureChannelProfile,
        bcienabled: bool,
        now_utc: &str,
    ) -> Result<Self, SessionGuardError> {
        // 1. Bind token to physical device + channel.
        if token.device_fingerprint != observed_device_fingerprint {
            return Err(SessionGuardError::InvalidEnv("device_fingerprint_mismatch"));
        }

        if token.secure_channel != *observed_secure_channel {
            return Err(SessionGuardError::InvalidEnv("secure_channel_mismatch"));
        }

        if !token.secure_channel.dns_fail_closed || !token.secure_channel.doh_pinned {
            return Err(SessionGuardError::InvalidEnv("insecure_dns_profile"));
        }

        if !token.secure_channel.tls_pinned {
            return Err(SessionGuardError::InvalidEnv("insecure_tls_profile"));
        }

        // 2. Enforce RoH ceiling.
        if !token.roh_leq_03 {
            return Err(SessionGuardError::RohViolation);
        }

        // 3. Enforce neurorights flags and BCI posture.
        if !token.neurorights.cognitive_liberty {
            return Err(SessionGuardError::NeurorightsViolation("cognitive_liberty_false"));
        }
        if !token.neurorights.mental_privacy {
            return Err(SessionGuardError::NeurorightsViolation("mental_privacy_false"));
        }
        if !token.neurorights.mental_integrity {
            return Err(SessionGuardError::NeurorightsViolation("mental_integrity_false"));
        }
        if !token.neurorights.augmentation_continuity {
            return Err(SessionGuardError::NeurorightsViolation("augmentation_continuity_false"));
        }
        if !bcienabled {
            return Err(SessionGuardError::InvalidEnv("bci_disabled"));
        }

        // 4. Expiry check (simple string compare placeholder; replace with real time parsing).
        if token.expiry_utc <= now_utc {
            return Err(SessionGuardError::Expired);
        }

        Ok(SessionGuard { token })
    }

    pub fn token(&self) -> &SessionToken {
        &self.token
    }

    pub fn has_role(&self, role: &Role) -> bool {
        self.token.roles.contains(role)
    }
}
