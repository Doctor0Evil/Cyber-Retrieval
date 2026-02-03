use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct HighRiskResearchEntry {
    pub entry_id: String,
    pub proposal_id: String,
    pub subjectid: String,
    pub changetype: String,
    pub roh_before: f32,
    pub roh_after: f32,
    pub roh_delta: f32,
    pub token_id: String,
    pub physioguard_state: String,
    pub timestamp_utc: String,
    pub hexstamp: String,
}

pub trait DonutloopWriter {
    fn append_highrisk_entry(
        &mut self,
        entry: HighRiskResearchEntry,
    ) -> Result<(), String>;
}
