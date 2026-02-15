use std::path::PathBuf;
use std::time::SystemTime;

use bioscale_fairness_core::{ResponsibilityScalar, OuterFreedomModel, KarmaConstraint};
use cybernet_metrics::{BiophysicalFlowsSnapshot, load_flows_for_manifest};
use cybernet_manifest::{DailyEvolutionManifest, load_manifest};

struct LabOuterFreedomModel;

impl OuterFreedomModel for LabOuterFreedomModel {
    fn outer_envelope(&self, r: ResponsibilityScalar) -> bioscale_fairness_core::OuterFreedomEnvelope {
        // Example: convex, monotone mapping tuned to lab policy.
        let v = r.value();
        let clamped = v.max(-1.0).min(5.0);
        let radius = if clamped <= 0.0 { 0.0 } else { (clamped / 5.0) };
        bioscale_fairness_core::OuterFreedomEnvelope {
            max_outer_radius: radius,
            bandwidth_cap: radius,
            device_control_cap: radius,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let manifest_path = PathBuf::from(args.next().expect("missing --manifest path"));

    let manifest: DailyEvolutionManifest = load_manifest(&manifest_path)?;
    let flows = load_flows_for_manifest(&manifest)?;

    let model = LabOuterFreedomModel;
    let now = SystemTime::now();
    let mut r = manifest.pre_window_r.unwrap_or(ResponsibilityScalar::neutral());
    let mut violations = Vec::new();

    for entry in &manifest.entries {
        let upgrade = entry.upgrade_ref(); // returns a FairnessConstrainedUpgrade impl.
        let snap: BiophysicalFlowsSnapshot = flows.snapshot_for(entry)?;

        let verdict = upgrade.karma_admissible(now, r, &snap, &model);

        if !verdict.is_karma_admissible {
            violations.push((
                entry.id.clone(),
                verdict.pre_r.value(),
                verdict.post_r.value(),
                verdict.delta_r,
            ));
        }

        r = verdict.post_r;
    }

    if !violations.is_empty() {
        eprintln!("Errority: Fairness violations detected (Δr < 0 with outer expansion):");
        for (id, pre, post, d) in violations {
            eprintln!("  - upgrade {}: pre_r = {:.3}, post_r = {:.3}, Δr = {:.3}", id, pre, post, d);
        }
        std::process::exit(1);
    }

    Ok(())
}
