#[derive(Clone, Debug)]
pub struct ComplianceConfig {
    pub roh_max: f32,                // e.g. 0.30 hard ceiling
    pub risk_global_max: f32,
    pub density_tissue_max: f32,
    pub snr_neural_min_db: f32,
    pub lambda2_net_max: f32,
    pub coverage_cortex_min: f32,
    pub energy_implant_max_j: f32,
    pub pe_link_max: f32,
    pub corridor_temp_max_c: f32,
    pub il6_max_pg_ml: f32,
    pub hrv_lf_hf_max: f32,
}
