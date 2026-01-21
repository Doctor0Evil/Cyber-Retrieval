#![forbid(unsafe_code)]

use governance_core::roles::GovernanceRole;
use governance_core::eligibility::GovernanceIndices;
use neurorights_core::{NeurorightsBound, NeurorightsEnvelope};
use neurorights_firewall::audit::{Authorship, EvidenceStamp};

#[derive(Clone, Debug)]
pub struct GovernanceDecisionLog {
    pub authorship: Authorship,
    pub evidence: EvidenceStamp,
    pub holder_did: String,
    pub aln_scope: String,
    pub bostrom_address: String,
    pub eligible_roles: Vec<GovernanceRole>,
    pub indices: GovernanceIndices,
}

impl GovernanceDecisionLog {
    pub fn from_query_and_decision(
        bound: &NeurorightsBound<crate::PromptEnvelope, NeurorightsEnvelope>,
        query: &crate::router::GovernanceQuery,
        decision: &crate::router::GovernanceDecision,
    ) -> Self {
        let authorship = Authorship {
            userdid: query.holder_did.clone(),
            aln: query.aln_scope.clone(),
            bostromaddress: query.bostrom_address.clone(),
            eibonlabel: "governance.eligibility.snapshot.v1".to_string(),
            neurorightsversion: bound.neurorights_envelope().policyversion.to_string(),
        };

        let evidence = EvidenceStamp::default_hex();

        Self {
            authorship,
            evidence,
            holder_did: query.holder_did.clone(),
            aln_scope: query.aln_scope.clone(),
            bostrom_address: query.bostrom_address.clone(),
            eligible_roles: decision.eligible_roles.clone(),
            indices: decision.indices.clone(),
        }
    }
}
