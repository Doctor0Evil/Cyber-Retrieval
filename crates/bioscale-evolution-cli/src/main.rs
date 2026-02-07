#![forbid(unsafe_code)]

use std::process::ExitCode;
use bioscale_upgrade_store::{UpgradeDescriptor, HostBudget, BciSafetyThresholds};
use bioscale_metrics_manifest::MetricsSchemaSnapshot;
use bioscale_manifest_emit::{DailyResearchManifest};
use cyberswarm_neurostack::registry::all_descriptors;
use aln_invariants::AlnInvariantsSurface;
use kani_harness_index::KaniHarnessIndex;

fn main() -> ExitCode {
    // 1. Regenerate ALN invariants from host evidence + policy.
    let invariants = AlnInvariantsSurface::from_env()
        .expect("ALN invariants regeneration failed");

    // 2. Load all upgrade descriptors from guard crates.
    let upgrades: Vec<UpgradeDescriptor> = all_descriptors();

    // 3. Derive runtime thresholds from invariants + descriptors.
    let thresholds = BciSafetyThresholds::from_invariants(&invariants);

    // 4. Regenerate metrics schema from upgrades.
    let metrics_schema = MetricsSchemaSnapshot::from_upgrades(&upgrades);

    // 5. Build machine-readable manifest.
    let manifest = DailyResearchManifest::from_components(
        &invariants,
        &upgrades,
        &thresholds,
        &metrics_schema,
    ).expect("manifest build failed");

    // 6. CI validation: invariants, guards, metrics, harnesses must align.
    let harness_index = KaniHarnessIndex::discover().expect("harness discovery failed");
    if let Err(e) = manifest.validate_ci_contract(&harness_index, &metrics_schema) {
        eprintln!("CI contract failed: {e:?}");
        return ExitCode::from(1);
    }

    // 7. Write manifest to disk (e.g. research-<date>-manifest.json).
    if let Err(e) = manifest.write_to_fs() {
        eprintln!("failed to write manifest: {e:?}");
        return ExitCode::from(1);
    }

    ExitCode::from(0)
}
