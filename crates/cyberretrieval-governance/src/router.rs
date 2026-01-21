#![forbid(unsafe_code)]

use crate::prompt::PromptEnvelope;
use neurorights_core::{NeurorightsBound, NeurorightsEnvelope};

/// Convenience alias for this crate.
#[derive(Clone, Debug)]
pub struct NeurorightsBoundEnvelope {
    pub bound: NeurorightsBound<PromptEnvelope, NeurorightsEnvelope>,
}
