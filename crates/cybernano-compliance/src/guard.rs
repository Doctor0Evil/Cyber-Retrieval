use crate::config::ComplianceConfig;
use cybernano_model::nanoswarm::NanoswarmNeuroThermoCorridorState;

pub trait GuardKernel {
    fn admissible(&self, cfg: &ComplianceConfig, state: &NanoswarmNeuroThermoCorridorState) -> bool;
    fn lyapunov_descent(
        &self,
        cfg: &ComplianceConfig,
        state: &NanoswarmNeuroThermoCorridorState,
    ) -> NanoswarmNeuroThermoCorridorState;
    fn chat_knowledge_factor(&self) -> f32;
}

#[derive(Clone, Debug)]
pub struct NanoswarmCorridorGuard;

impl GuardKernel for NanoswarmCorridorGuard {
    fn admissible(&self, cfg: &ComplianceConfig, s: &NanoswarmNeuroThermoCorridorState) -> bool {
        if s.core_temperature_c > cfg.corridor_temp_max_c { return false; }
        if s.local_delta_t_c < -2.0 || s.local_delta_t_c > 2.0 { return false; }
        if s.il6_pg_ml > cfg.il6_max_pg_ml { return false; }
        if s.hrv.lf_hf_ratio > cfg.hrv_lf_hf_max { return false; }
        if s.thermal_duty_cycle < 0.0 || s.thermal_duty_cycle > 1.0 { return false; }
        true
    }

    fn lyapunov_descent(
        &self,
        cfg: &ComplianceConfig,
        s: &NanoswarmNeuroThermoCorridorState,
    ) -> NanoswarmNeuroThermoCorridorState {
        let mut next = s.clone();
        if next.core_temperature_c > cfg.corridor_temp_max_c {
            let excess = next.core_temperature_c - cfg.corridor_temp_max_c;
            let k = 0.5_f32;
            next.thermal_duty_cycle = (next.thermal_duty_cycle - k * excess).max(0.0);
        }
        if next.il6_pg_ml > cfg.il6_max_pg_ml {
            let k = 0.1_f32;
            next.thermal_duty_cycle = (next.thermal_duty_cycle - k).max(0.0);
        }
        next
    }

    fn chat_knowledge_factor(&self) -> f32 {
        0.93
    }
}
