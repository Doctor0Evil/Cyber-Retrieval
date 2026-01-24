use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{KsrTriple, PromptEnvelope};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NeuralRopeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralRopeSegment {
    pub rope_id: NeuralRopeId,
    pub index: u32,
    pub envelope: PromptEnvelope,
    pub ksr_delta: KsrTriple,
    pub ksr_cumulative: KsrTriple,
    pub roh_index: u8,
    pub logged_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralRope {
    pub id: NeuralRopeId,
    pub segments: Vec<NeuralRopeSegment>,
}

impl NeuralRope {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: NeuralRopeId(id.into()),
            segments: Vec::new(),
        }
    }

    pub fn push_segment(&mut self, segment: NeuralRopeSegment) {
        self.segments.push(segment);
    }

    pub fn current_ksr(&self) -> Option<KsrTriple> {
        self.segments.last().map(|s| s.ksr_cumulative)
    }

    pub fn current_roh(&self) -> Option<u8> {
        self.segments.last().map(|s| s.roh_index)
    }
}
