use serde::{Deserialize, Serialize};

/// Normalized safety margin for one dimension.
/// 1.0 = at boundary, >1.0 = inside safe zone, <1.0 = breached.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SafetyMargin {
    pub name: &'static str,
    pub value: f64,
}

impl SafetyMargin {
    pub fn new(name: &'static str, value: f64) -> Self {
        Self { name, value }
    }

    pub fn intact(&self) -> bool {
        self.value >= 1.0
    }
}

/// Logical biophysical separator (BBB, gut barrier, neuromorph sandbox gate).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiophysicalSeparator {
    pub id: &'static str,
    pub margins: Vec<SafetyMargin>,
}

impl BiophysicalSeparator {
    pub fn new(id: &'static str, margins: Vec<SafetyMargin>) -> Self {
        Self { id, margins }
    }

    /// Composite margin: minimum over all dimensions.
    pub fn composite_margin(&self) -> f64 {
        self.margins
            .iter()
            .map(|m| m.value)
            .fold(f64::INFINITY, f64::min)
    }

    pub fn intact(&self) -> bool {
        self.composite_margin() >= 1.0
    }
}

/// Software-respected "BBB" with separate physical and neurorights envelopes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BloodBrainBarrierGuard {
    pub physical: BiophysicalSeparator,
    pub neurorights: BiophysicalSeparator,
}

impl BloodBrainBarrierGuard {
    /// Low-impact reads/queries.
    pub fn allow_neuromorph_read(&self) -> bool {
        self.physical.intact() && self.neurorights.intact()
    }

    /// High-impact evolution only when both envelopes are comfortably safe.
    pub fn allow_high_impact_evolution(&self) -> bool {
        self.physical.composite_margin() >= 1.1
            && self.neurorights.composite_margin() >= 1.1
    }
}
