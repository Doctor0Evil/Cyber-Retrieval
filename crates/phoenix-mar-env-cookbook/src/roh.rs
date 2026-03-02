//! RoH envelope for Phoenix MAR environmental routing.

#[derive(Clone, Copy, Debug)]
pub struct RiskEnvelope {
    pub knowledge_factor: f32,
    pub risk_of_harm: f32,
    pub cybostate_factor: f32,
    pub hexstamp: &'static str,
}

pub const ENV_RISK_ENVELOPE: RiskEnvelope = RiskEnvelope {
    knowledge_factor: 0.91,
    risk_of_harm: 0.12,     // << hard below 0.3
    cybostate_factor: 0.88,
    hexstamp: "0x9F21A3C7_PhoenixMarEnvCookbook_v1",
};
