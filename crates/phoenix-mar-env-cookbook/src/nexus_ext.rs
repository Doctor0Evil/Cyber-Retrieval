//! NexusSample extension for PFAS, nitrate, embodied energy, failure telemetry.

use serde::{Deserialize, Serialize};
use xr_lab_grid::nexussample::NexusSample;

/// Domain discriminator for environmental signals carried inside a NexusSample.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EnvKind {
    Pfas,
    Nitrate,
    EmbodiedEnergy,
    FailureTelemetry,
}

/// PFAS-specific view (concentrations etc.), attached via evidence-only struct.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PfasSignal {
    pub corridor_id: String,        // env.corridor.pfas.<aquifer|well>.v1
    pub sample_id: String,
    pub concentration_ng_l: f64,
    pub detection_limit_ng_l: f64,
    pub epa_method: String,
}

/// Nitrate-specific view.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NitrateSignal {
    pub corridor_id: String,        // env.corridor.nitrate.<river|well>.v1
    pub sample_id: String,
    pub concentration_mg_l: f64,
    pub standard_limit_mg_l: f64,
}

/// Embodied energy telemetry for infrastructure assets.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbodiedEnergySignal {
    pub asset_id: String,
    pub corridor_id: String,        // env.corridor.embodied.energy.<asset>.v1
    pub energy_kj: f64,
    pub co2e_kg: f64,
}

/// Failure telemetry from monitoring systems.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FailureTelemetrySignal {
    pub subsystem: String,
    pub corridor_id: String,        // env.corridor.failure.<subsystem>.v1
    pub failure_code: String,
    pub severity: u8,
    pub recovered: bool,
}

/// Unified, cookbook-level envelope around a NexusSample plus decoded env signal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvDomainSample {
    pub kind: EnvKind,
    pub nexus: NexusSample,
    pub pfas: Option<PfasSignal>,
    pub nitrate: Option<NitrateSignal>,
    pub energy: Option<EmbodiedEnergySignal>,
    pub failure: Option<FailureTelemetrySignal>,
}

pub trait NexusEnvExt {
    fn as_env_domain_sample(self, decoded: EnvSignalDecoded) -> EnvDomainSample;
}

/// Helper for passing decoded env payloads into the NexusEnvExt.
#[derive(Clone, Debug)]
pub enum EnvSignalDecoded {
    Pfas(PfasSignal),
    Nitrate(NitrateSignal),
    Embodied(EmbodiedEnergySignal),
    Failure(FailureTelemetrySignal),
}

impl NexusEnvExt for NexusSample {
    fn as_env_domain_sample(self, decoded: EnvSignalDecoded) -> EnvDomainSample {
        match decoded {
            EnvSignalDecoded::Pfas(p) => EnvDomainSample {
                kind: EnvKind::Pfas,
                nexus: self,
                pfas: Some(p),
                nitrate: None,
                energy: None,
                failure: None,
            },
            EnvSignalDecoded::Nitrate(n) => EnvDomainSample {
                kind: EnvKind::Nitrate,
                nexus: self,
                pfas: None,
                nitrate: Some(n),
                energy: None,
                failure: None,
            },
            EnvSignalDecoded::Embodied(e) => EnvDomainSample {
                kind: EnvKind::EmbodiedEnergy,
                nexus: self,
                pfas: None,
                nitrate: None,
                energy: Some(e),
                failure: None,
            },
            EnvSignalDecoded::Failure(f) => EnvDomainSample {
                kind: EnvKind::FailureTelemetry,
                nexus: self,
                pfas: None,
                nitrate: None,
                energy: None,
                failure: Some(f),
            },
        }
    }
}
