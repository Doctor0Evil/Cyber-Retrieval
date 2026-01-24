use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KsrTriple {
    pub knowledge: u8,
    pub social: u8,
    pub risk: u8,
}

impl KsrTriple {
    pub const fn new(knowledge: u8, social: u8, risk: u8) -> Self {
        Self {
            knowledge,
            social,
            risk,
        }
    }
}

pub const KSR_CEILING_DEFAULT: KsrTriple = KsrTriple {
    knowledge: 0xE2,
    social: 0x78,
    risk: 0x27,
};
