use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetrievalIntent {
    RetrieveKnowledgeAcademic,
    ThreatScanAcademic,
    RetrievePolicyDcmHci,
    NeuralRopeResearchAcademic,
}
