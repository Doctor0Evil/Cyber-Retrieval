#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Scalar knowledge factor F_K in [0,1].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeFactor {
    pub v_validation: f64, // [0,1]
    pub r_reuse: f64,      // [0,1]
    pub e_eco: f64,        // [0,1]
    pub n_novelty: f64,    // [0,1]
    pub f_value: f64,      // [0,1] derived
}

/// Neurorights-aware CyberRank vector.
/// All components live in [0,1] and are interpreted as
/// probabilities-of-observation under neurorights-safe routing.[file:31]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CyberRank {
    pub safety: f64,        // neurorights / bioscale safety axis
    pub neurorights: f64,   // cognitive liberty, mental privacy, authorship
    pub eco: f64,           // EcoSys alignment
    pub jurisdiction: f64,  // Globe + policy.lattice compliance
}

/// Hard clamp to [0,1] for any scalar.
fn clamp01(x: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

/// Compute F_K = w_v V + w_r R + w_e E + w_n N with host-safe bounds.[file:31]
pub fn compute_knowledge_factor(
    v_validation: f64,
    r_reuse: f64,
    e_eco: f64,
    n_novelty: f64,
    w_v: f64,
    w_r: f64,
    w_e: f64,
    w_n: f64,
) -> KnowledgeFactor {
    let v = clamp01(v_validation);
    let r = clamp01(r_reuse);
    let e = clamp01(e_eco);
    let n = clamp01(n_novelty);

    // Normalize weights to sum 1 but keep zero-safe.
    let w_sum = w_v + w_r + w_e + w_n;
    let (wv, wr, we, wn) = if w_sum <= 0.0 {
        (0.25, 0.25, 0.25, 0.25)
    } else {
        (w_v / w_sum, w_r / w_sum, w_e / w_sum, w_n / w_sum)
    };

    let f_raw = wv * v + wr * r + we * e + wn * n;
    let f_value = clamp01(f_raw);

    KnowledgeFactor {
        v_validation: v,
        r_reuse: r,
        e_eco: e,
        n_novelty: n,
        f_value,
    }
}

/// One-step CyberRank update under donut-loop style couplings.[file:31]
/// This is retrieval-only: it never mutates external state; callers decide
/// whether to commit the returned value.
pub fn update_cyberrank(
    current: CyberRank,
    delta_safety: f64,
    delta_neurorights: f64,
    delta_eco: f64,
    delta_jurisdiction: f64,
) -> CyberRank {
    // Contract: each axis can only move by at most 0.1 per step to keep
    // Lyapunov-style boundedness and avoid abrupt governance flips.[file:31]
    fn limited_step(curr: f64, delta: f64) -> f64 {
        let d_clamped = clamp01(0.5 + delta) - 0.5; // soft bound [-0.5,0.5]→[-0.5,0.5]
        let d_limited = if d_clamped > 0.1 {
            0.1
        } else if d_clamped < -0.1 {
            -0.1
        } else {
            d_clamped
        };
        clamp01(curr + d_limited)
    }

    CyberRank {
        safety: limited_step(current.safety, delta_safety),
        neurorights: limited_step(current.neurorights, delta_neurorights),
        eco: limited_step(current.eco, delta_eco),
        jurisdiction: limited_step(current.jurisdiction, delta_jurisdiction),
    }
}

/// Simple Lyapunov-like residual V = ||r_target - r||_2^2 used in formal proofs.[file:31]
pub fn cyberrank_residual(target: CyberRank, current: CyberRank) -> f64 {
    let ds = target.safety - current.safety;
    let dn = target.neurorights - current.neurorights;
    let de = target.eco - current.eco;
    let dj = target.jurisdiction - current.jurisdiction;
    ds * ds + dn * dn + de * de + dj * dj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knowledge_factor_in_unit_interval() {
        let kf = compute_knowledge_factor(1.2, -0.1, 0.5, 0.3, 1.0, 1.0, 1.0, 1.0);
        assert!(kf.f_value >= 0.0 && kf.f_value <= 1.0);
    }

    #[test]
    fn cyberrank_update_is_bounded() {
        let r0 = CyberRank {
            safety: 0.5,
            neurorights: 0.5,
            eco: 0.5,
            jurisdiction: 0.5,
        };
        let r1 = update_cyberrank(r0, 10.0, -10.0, 10.0, -10.0);
        // Each component moves by at most 0.1 and stays in [0,1].
        assert!(r1.safety >= 0.0 && r1.safety <= 1.0);
        assert!((r1.safety - 0.5).abs() <= 0.1 + 1e-9);
    }
}
