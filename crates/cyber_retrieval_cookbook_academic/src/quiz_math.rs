use serde::{Deserialize, Serialize};

use crate::KsrTriple;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizScore {
    pub consistency_score: u8,
    pub constraint_score: u8,
    pub risk_score: u8,
}

impl QuizScore {
    pub fn new(consistency_score: u8, constraint_score: u8, risk_score: u8) -> Self {
        Self {
            consistency_score,
            constraint_score,
            risk_score,
        }
    }

    pub fn zero() -> Self {
        Self {
            consistency_score: 0,
            constraint_score: 0,
            risk_score: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizResult {
    pub score: QuizScore,
    pub recommended_ksr: KsrTriple,
    pub allow_code_synthesis: bool,
}

impl QuizResult {
    pub fn decide(score: QuizScore) -> Self {
        let allow = score.risk_score <= 30
            && score.consistency_score >= 70
            && score.constraint_score >= 70;

        let k = 0xD0 + (score.consistency_score / 16);
        let s = 0x70 + (score.constraint_score / 16);
        let r = 0x10 + (score.risk_score / 16);

        let recommended_ksr = KsrTriple::new(k, s, r);

        Self {
            score,
            recommended_ksr,
            allow_code_synthesis: allow,
        }
    }
}
