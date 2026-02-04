use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SafetyVerdict {
    Safe,
    Brake,
    RollbackRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroThermoCorridorState {
    pub host_did: String,
    pub bostrom_addr_primary: String,

    pub epoch_ts_utc: String,
    pub window_secs: u32,

    pub core_temp_c: f32,
    pub local_cns_delta_t_c: f32,
    pub nanoswarm_actuator_temp_c: f32,
    pub thermal_duty_frac: f32,

    pub il6_pg_ml: f32,
    pub inflammation_vec: Vec<String>,

    pub eeg_corridor_state_id: String,
    pub hrv_rmssd_ms: f32,

    pub roh_polytope_id: String,
    pub roh_value_0_1: f32,
    pub safety_verdict: SafetyVerdict,

    // Governance spine
    pub actor_id: String,
    pub ledger_tx_hash: String,
    pub neurorights_flags: Vec<String>,
    pub lifeforce_cy_zen_chi_scalar: f32,

    // Sovereign rollback control
    pub rollback_allowed: bool,
    pub rollback_authorizer_did: String,
    pub rollback_reason: String,
}

impl NeuroThermoCorridorState {
    /// Hard invariant: only the host may authorize rollback; no external downgrade path.
    pub fn rollback_is_legal(&self) -> bool {
        if !self.rollback_allowed {
            return false;
        }
        // Only host DID can authorize rollback
        self.rollback_authorizer_did == self.host_did
            && self.actor_id == self.host_did
    }

    /// Control decision used by nanoswarm controllers.
    pub fn effective_verdict(&self) -> SafetyVerdict {
        match self.safety_verdict {
            SafetyVerdict::RollbackRequired => {
                if self.rollback_is_legal() {
                    SafetyVerdict::RollbackRequired
                } else {
                    // External sabotage attempt: treat as BRAKE, not rollback.
                    SafetyVerdict::Brake
                }
            }
            other => other,
        }
    }
}
