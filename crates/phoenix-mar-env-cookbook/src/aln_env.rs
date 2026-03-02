//! ALN stubs for env.corridor.pfas.* and env.corridor.nitrate.* etc.

use serde::{Deserialize, Serialize};

/// Logical corridor identifier used for ALN and routing.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EnvCorridorId(pub String);

impl EnvCorridorId {
    pub fn pfas_well(id: &str) -> Self {
        Self(format!("env.corridor.pfas.well.{id}.v1"))
    }

    pub fn nitrate_river(id: &str) -> Self {
        Self(format!("env.corridor.nitrate.river.{id}.v1"))
    }

    pub fn embodied_asset(id: &str) -> Self {
        Self(format!("env.corridor.embodied.energy.{id}.v1"))
    }

    pub fn failure_subsystem(id: &str) -> Self {
        Self(format!("env.corridor.failure.{id}.v1"))
    }
}

/// Minimal ALN corridor config used at runtime (values are placeholders to be
/// derived from EPA / WEAU guidance and Phoenix MAR policy).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvAlnCorridor {
    pub id: EnvCorridorId,
    pub roh_ceiling: f32,
    pub k_min: f32,
    pub k_max: f32,
    pub e_max_kj: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvAlnConfig {
    pub corridors: Vec<EnvAlnCorridor>,
}

impl EnvAlnConfig {
    pub fn find(&self, id: &EnvCorridorId) -> Option<&EnvAlnCorridor> {
        self.corridors.iter().find(|c| &c.id == id)
    }
}
