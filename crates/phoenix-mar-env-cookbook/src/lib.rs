#![forbid(unsafe_code)]

pub mod nexus_ext;
pub mod env_guard;
pub mod aln_env;
pub mod roh;

pub use nexus_ext::{
    EnvDomainSample, EnvKind, NexusEnvExt, PfasSignal, NitrateSignal,
    EmbodiedEnergySignal, FailureTelemetrySignal,
};
pub use env_guard::{EnvGuardKernel, DefaultEnvGuardKernel, EnvGuardDecision};
pub use aln_env::{EnvCorridorId, EnvAlnCorridor, EnvAlnConfig};
pub use roh::{RiskEnvelope, ENV_RISK_ENVELOPE};
