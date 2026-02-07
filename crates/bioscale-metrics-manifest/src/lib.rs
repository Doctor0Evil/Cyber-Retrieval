#![forbid(unsafe_code)]

use serde::{Serialize, Deserialize};
use bioscale_upgrade_store::UpgradeDescriptor;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricFamilyDef {
    pub name: String,
    pub help: String,
    pub kind: String,          // counter | gauge | histogram
    pub labels: Vec<String>,   // ["upgrade_id", "reason", "host_id", "policy_id"]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsSchemaSnapshot {
    pub version: String,
    pub families: Vec<MetricFamilyDef>,
}

impl MetricsSchemaSnapshot {
    pub fn from_upgrades(upgrades: &[UpgradeDescriptor]) -> Self {
        // Metric set is stable; cardinality is tied to upgrade_ids.
        let mut families = Vec::new();

        families.push(MetricFamilyDef {
            name: "bciguard_denied_total".into(),
            help: "BCI guard denials by upgrade and reason".into(),
            kind: "counter".into(),
            labels: vec!["upgrade_id".into(), "reason".into(), "host_id".into()],
        });

        families.push(MetricFamilyDef {
            name: "bcienvelope_corridor_joules".into(),
            help: "Session energy usage vs manifest envelope".into(),
            kind: "gauge".into(),
            labels: vec!["upgrade_id".into(), "host_id".into()],
        });

        // … more metrics families for duty, delta_T, etc …

        MetricsSchemaSnapshot {
            version: "bci-metrics-v1".into(),
            families,
        }
    }
}
