use serde::{Deserialize, Serialize};

use crate::{
    DEFAULT_CYBOSTATE_FACTOR,
    DEFAULT_KNOWLEDGE_FACTOR,
    DEFAULT_RISK_OF_HARM,
    RISK_OF_HARM_CEILING,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEnvelope {
    pub knowledge_factor: f64,
    pub risk_of_harm: f64,
    pub cybostate_factor: f64,
    pub hexstamp: String,
}

impl RiskEnvelope {
    /// Constructs a default website-governance RiskEnvelope under your architecture.
    pub fn default(hexstamp: impl Into<String>) -> Self {
        Self {
            knowledge_factor: DEFAULT_KNOWLEDGE_FACTOR,
            risk_of_harm: DEFAULT_RISK_OF_HARM,
            cybostate_factor: DEFAULT_CYBOSTATE_FACTOR,
            hexstamp: hexstamp.into(),
        }
    }

    /// Validates that the risk-of-harm is under the ceiling defined in ALN.
    pub fn validate(&self) -> Result<(), RiskError> {
        if self.risk_of_harm > RISK_OF_HARM_CEILING {
            return Err(RiskError::RiskOfHarmExceeded {
                roh: self.risk_of_harm,
                ceiling: RISK_OF_HARM_CEILING,
            });
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RiskError {
    #[error("risk-of-harm {roh} exceeds ceiling {ceiling}")]
    RiskOfHarmExceeded { roh: f64, ceiling: f64 },
}
