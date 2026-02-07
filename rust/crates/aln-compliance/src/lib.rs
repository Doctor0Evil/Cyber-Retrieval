#![forbid(unsafe_code)]

#[derive(Clone, Debug)]
pub enum NeurorightsClause {
    RollbackAnytimeV1,
    NoNonConsensualModulationV1,
    NoRawEEGExportV1,
}

#[derive(Clone, Debug)]
pub enum ComplianceRole {
    PatientConsent,
    EthicsBoard,
    RegulatorQuorum,
}

#[derive(Clone, Debug)]
pub struct ALNComplianceParticle {
    pub host_did: String,
    pub upgrade_hash: [u8; 32],
    pub evidence: EvidenceBundle,
    pub budget_fit: bool,

    // Canonical neurorights flags – no free text.
    pub rollback_anytime: bool,
    pub no_nonconsensual_modulation: bool,
    pub no_raw_eeg_export: bool,

    // Role set and clause IDs for manifest binding.
    pub roles: Vec<ComplianceRole>,
    pub clause_ids: Vec<NeurorightsClause>,
}

impl ALNComplianceParticle {
    pub fn is_compliant_for_bci(&self) -> bool {
        self.budget_fit
            && self.rollback_anytime
            && self.no_nonconsensual_modulation
            && self.no_raw_eeg_export
    }
}
