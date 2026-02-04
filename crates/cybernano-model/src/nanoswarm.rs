use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EegData {
    pub beta_over_alpha: f32,
    pub engagement_index: f32,
    pub bands: [f32; 8], // domain‑defined ordering
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HrvData {
    pub rmssd_ms: f32,
    pub lf_hf_ratio: f32,
    pub sdnn_ms: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoswarmNeuroThermoCorridorState {
    pub host_did: String,              // DID / ALN / Bostrom linkage
    pub corridor_id: String,           // 5D7D corridor
    pub core_temperature_c: f32,
    pub local_delta_t_c: f32,
    pub actuator_temperature_c: f32,
    pub thermal_duty_cycle: f32,       // 0.0–1.0
    pub il6_pg_ml: f32,
    pub eeg: EegData,
    pub hrv: HrvData,
}

// lifeforce compound type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeforceScalar {
    pub cy: f32,
    pub zen: f32,
    pub chi: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeforceNanoswarmEnvelopeSample {
    pub host_did: String,
    pub corridor_id: String,
    pub lifeforce: LifeforceScalar,
    pub lifeforce_drain_window_s: u32,
    pub blood_token_debits: i64,
    pub bio_karma: f32,
}
