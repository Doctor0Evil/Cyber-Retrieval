#![forbid(unsafe_code)]

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceTag {
    pub hex: String,
    pub domain: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub sequences: Vec<EvidenceTag>, // CI enforces len == 10
}

pub const DEFAULT_BCI_EVIDENCE: [EvidenceTag; 10] = [
    EvidenceTag { hex: "a1f3c9b2".into(), domain: "resting_metabolic_rate_atp_turnover".into() },
    EvidenceTag { hex: "4be79d01".into(), domain: "oxphos_efficiency_joule_coupling".into() },
    EvidenceTag { hex: "9cd4a7e8".into(), domain: "protein_synthesis_cost_aa_nitrogen_balance".into() },
    EvidenceTag { hex: "2f8c6b44".into(), domain: "thermoregulatory_limits_delta_t_0.2_0.5C".into() },
    EvidenceTag { hex: "7e1da2ff".into(), domain: "peripheral_circulation_adaptation".into() },
    EvidenceTag { hex: "5b93e0c3".into(), domain: "neurovascular_coupling".into() },
    EvidenceTag { hex: "d0174aac".into(), domain: "safe_eeg_bci_duty_cycle_0.4_0.6".into() },
    EvidenceTag { hex: "6ac2f9d9".into(), domain: "neuromorphic_ml_energy_10e-10J_op".into() },
    EvidenceTag { hex: "c4e61b20".into(), domain: "protein_turnover_neural_skeletal_half_life".into() },
    EvidenceTag { hex: "8f09d5ee".into(), domain: "inflammation_pain_thresholds".into() },
];
