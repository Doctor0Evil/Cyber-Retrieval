pub enum ComplianceDecision {
    Safe,
    Brake { reason: String },
    RollbackRequired { reason: String },
}

pub struct NanoswarmComplianceFieldV1 {
    pub radiation_dose_gy_per_hr: f32,
    pub il6_level_pg_ml: f32,
    pub bone_density_tscore: f32,
    pub muscle_mass_index: f32,
    pub circulatory_pressure_mm_hg: f32,
    pub eeg_load_norm: f32,
    pub hrv_norm: f32,
    pub self_reported_stress_0_1: f32,
    pub dopamine_budget_0_1: f32,
    pub serotonin_budget_0_1: f32,
}
