//! EnvGuardKernel trait under NeurorightsBound routing and RoH ≤ 0.3.

use neurorights_core::{NeurorightsBound, NeurorightsEnvelope};
use neurorights_firewall::PromptEnvelope;
use serde::{Deserialize, Serialize};

use crate::aln_env::{EnvAlnConfig, EnvCorridorId};
use crate::nexus_ext::{EnvDomainSample, EnvKind};
use crate::roh::ENV_RISK_ENVELOPE;

/// Result of applying the guard to one EnvDomainSample.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvGuardDecision {
    pub allowed: bool,
    pub roh_estimate: f32,
    pub reason: Option<String>,
}

/// Core trait: any environment playbook guard implements this.
pub trait EnvGuardKernel {
    fn admissible(
        &self,
        aln: &EnvAlnConfig,
        sample: &EnvDomainSample,
    ) -> EnvGuardDecision;
}

/// Minimal, cookbook-default implementation.
#[derive(Clone, Debug, Default)]
pub struct DefaultEnvGuardKernel;

impl DefaultEnvGuardKernel {
    fn corridor_id_for(sample: &EnvDomainSample) -> Option<EnvCorridorId> {
        match sample.kind {
            EnvKind::Pfas => sample
                .pfas
                .as_ref()
                .map(|p| EnvCorridorId(p.corridor_id.clone())),
            EnvKind::Nitrate => sample
                .nitrate
                .as_ref()
                .map(|n| EnvCorridorId(n.corridor_id.clone())),
            EnvKind::EmbodiedEnergy => sample
                .energy
                .as_ref()
                .map(|e| EnvCorridorId(e.corridor_id.clone())),
            EnvKind::FailureTelemetry => sample
                .failure
                .as_ref()
                .map(|f| EnvCorridorId(f.corridor_id.clone())),
        }
    }

    fn roh_estimate(sample: &EnvDomainSample) -> f32 {
        // Simple placeholder: treat all env telemetry as low-risk, retrieval-only.
        // Bound strictly under the global RoH ceiling 0.3 for Cyber-Retrieval.
        let base = 0.12;
        match sample.kind {
            EnvKind::Pfas | EnvKind::Nitrate => base,
            EnvKind::EmbodiedEnergy | EnvKind::FailureTelemetry => base * 0.9,
        }
    }
}

impl EnvGuardKernel for DefaultEnvGuardKernel {
    fn admissible(
        &self,
        aln: &EnvAlnConfig,
        sample: &EnvDomainSample,
    ) -> EnvGuardDecision {
        let roh = Self::roh_estimate(sample);

        if roh > 0.3 {
            return EnvGuardDecision {
                allowed: false,
                roh_estimate: roh,
                reason: Some("RoH ceiling 0.3 exceeded for env sample".into()),
            };
        }

        let corridor_id = match Self::corridor_id_for(sample) {
            Some(id) => id,
            None => {
                return EnvGuardDecision {
                    allowed: false,
                    roh_estimate: roh,
                    reason: Some("Missing env corridor id".into()),
                }
            }
        };

        let Some(corr) = aln.find(&corridor_id) else {
            return EnvGuardDecision {
                allowed: false,
                roh_estimate: roh,
                reason: Some("Unknown env corridor in ALN config".into()),
            };
        };

        // Knowledge-factor style check: K within corridor bounds.
        let k = ENV_RISK_ENVELOPE.knowledge_factor;
        if k < corr.k_min || k > corr.k_max {
            return EnvGuardDecision {
                allowed: false,
                roh_estimate: roh,
                reason: Some("Knowledge-Factor outside corridor bounds".into()),
            };
        }

        EnvGuardDecision {
            allowed: true,
            roh_estimate: roh,
            reason: None,
        }
    }
}

/// NeurorightsBound entry point for env guard playbooks.
/// This is the cookbook’s “NeurorightsBound routing + RoH ≤ 0.3” surface.
pub fn handle_env_guard(
    bound: NeurorightsBound<PromptEnvelope, NeurorightsEnvelope>,
    aln_cfg: &EnvAlnConfig,
    sample: EnvDomainSample,
    guard: &impl EnvGuardKernel,
) -> EnvGuardDecision {
    let _env = bound.neurorights_envelope(); // compile-time neurorights contract
    let decision = guard.admissible(aln_cfg, &sample);

    // All decisions are implicitly hex-stamped via ENV_RISK_ENVELOPE.
    // External logging code can attach ENV_RISK_ENVELOPE.hexstamp and the host DID.

    decision
}
