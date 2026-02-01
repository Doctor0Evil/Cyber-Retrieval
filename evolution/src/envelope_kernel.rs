use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum EvoDomain {
    NeuromorphReflex,
    NeuromorphSense,
    NeuromorphAttention,
    Defensive,
    General,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Envelopes {
    /// Lifeforce / physiology margin, 0.0–1.0 (1.0 = at boundary, >1 internally clamped to 1).
    pub lifeforce: f64,
    /// Neurorights invariants margin (mental privacy, cognitive liberty, integrity).
    pub neurorights: f64,
    /// Eco / carbon / device-hours margin.
    pub eco: f64,
    /// Biophysical aura / karma margin (kindness, non‑coercion, rescue, eco‑care).
    pub karma: f64,
}

impl Envelopes {
    pub fn clamped(&self) -> Self {
        let clamp = |x: f64| x.max(0.0).min(1.0);
        Self {
            lifeforce: clamp(self.lifeforce),
            neurorights: clamp(self.neurorights),
            eco: clamp(self.eco),
            karma: clamp(self.karma),
        }
    }
}

pub fn decay_from_envelopes(env: &Envelopes, domain: EvoDomain) -> f64 {
    let e = env.clamped();

    // If neurorights are breached, evolution is hard‑stopped regardless of other envelopes.
    if e.neurorights <= 0.0 {
        return 0.0;
    }

    let base = e.lifeforce.min(e.eco).min(e.neurorights);

    // Karma bonus for clearly protective domains only.
    let karma_bonus = match domain {
        EvoDomain::NeuromorphReflex
        | EvoDomain::NeuromorphSense
        | EvoDomain::NeuromorphAttention
        | EvoDomain::Defensive => 0.1 * e.karma,
        EvoDomain::General => 0.0,
    };

    let decay = (base + karma_bonus).max(0.0).min(1.0);
    decay
}
