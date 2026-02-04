use cybernano_model::nanoswarm::NanoswarmNeuroThermoCorridorState;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AlnParticleKind {
    Sample,
    Telemetry,
    Event,
    EvolutionEvent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DonutloopLedgerEntry<T> {
    pub particle: AlnParticleKind,
    pub aln_particle_id: String,   // e.g. "nanoswarm.neurothermo.corridor.v1"
    pub host_did: String,
    pub bostrom_address: String,
    pub payload: T,
    pub roh_index: f32,
    pub knowledge_factor: f32,
    pub cybostate_factor: f32,
    pub hexstamp: String,
    pub txn_hash: Option<String>,
}

impl DonutloopLedgerEntry<NanoswarmNeuroThermoCorridorState> {
    pub fn new_sample(
        state: NanoswarmNeuroThermoCorridorState,
        roh_index: f32,
        knowledge_factor: f32,
        cybostate_factor: f32,
        hexstamp: String,
    ) -> Self {
        Self {
            particle: AlnParticleKind::Sample,
            aln_particle_id: "nanoswarm.neurothermo.corridor.v1".to_string(),
            host_did: state.host_did.clone(),
            bostrom_address: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
            payload: state,
            roh_index,
            knowledge_factor,
            cybostate_factor,
            hexstamp,
            txn_hash: None,
        }
    }
}
