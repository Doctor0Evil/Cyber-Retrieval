#![forbid(unsafe_code)]

use serde::{Serialize, Deserialize};
use bioscale_upgrade_store::{UpgradeDescriptor, EvidenceBundle, ReversalConditions};
use aln_invariants::AlnInvariantsSurface;
use bioscale_metrics_manifest::MetricsSchemaSnapshot;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestUpgradeRow {
    pub upgrade_id: String,
    pub energy_joules: f64,
    pub protein_aa: u64,
    pub max_delta_c: f32,
    pub max_core_c: f32,
    pub max_duty_fraction: f32,
    pub reversal: ReversalConditions,
    pub evidence: EvidenceBundle,     // enforced 10-tag chain
    pub kani_harnesses: Vec<String>,  // names required in CI
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyResearchManifest {
    pub host_did: String,
    pub bostrom_address: String,
    pub git_commit: String,
    pub crate_versions: Vec<(String, String)>,
    pub bci_invariants: BciAlnInvariants,
    pub upgrades: Vec<ManifestUpgradeRow>,
    pub metrics_schema: MetricsSchemaSnapshot,
}

impl DailyResearchManifest {
    pub fn from_components(
        invariants: &AlnInvariantsSurface,
        upgrades: &[UpgradeDescriptor],
        thresholds: &BciSafetyThresholds,
        metrics: &MetricsSchemaSnapshot,
    ) -> Result<Self, ManifestError> {
        let rows = upgrades
            .iter()
            .map(|u| ManifestUpgradeRow {
                upgrade_id: u.upgrade_id.to_string(),
                energy_joules: u.energy_cost_joules,
                protein_aa: u.protein_cost_aa,
                max_delta_c: invariants.bci.max_delta_c,
                max_core_c: invariants.bci.max_core_c,
                max_duty_fraction: invariants.bci.max_duty_fraction,
                reversal: u.reversal.clone(),
                evidence: u.evidence.clone(),
                kani_harnesses: u.kani_harnesses.clone(),
            })
            .collect();

        Ok(DailyResearchManifest {
            host_did: std::env::var("HOST_DID").unwrap_or_default(),
            bostrom_address: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".into(),
            git_commit: Self::current_git_commit()?,
            crate_versions: Self::current_crate_versions()?,
            bci_invariants: invariants.bci.clone(),
            upgrades: rows,
            metrics_schema: metrics.clone(),
        })
    }

    pub fn validate_ci_contract(
        &self,
        harness_index: &KaniHarnessIndex,
        metrics_schema: &MetricsSchemaSnapshot,
    ) -> Result<(), ManifestError> {
        // 1. Every upgrade has exactly 10 evidence tags.
        for u in &self.upgrades {
            if u.evidence.sequences.len() != 10 {
                return Err(ManifestError::BadEvidenceLen(u.upgrade_id.clone()));
            }
        }

        // 2. Each upgrade has at least one Kani harness registered.
        for u in &self.upgrades {
            if !harness_index.covers_upgrade(&u.upgrade_id, &u.kani_harnesses) {
                return Err(ManifestError::MissingHarness(u.upgrade_id.clone()));
            }
        }

        // 3. Metrics schema version and families are consistent.
        metrics_schema.validate_against_manifest(self)?;

        Ok(())
    }
}
