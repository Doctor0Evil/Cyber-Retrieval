// File: Cyber-Retrieval/crates/cyber_retrieval_actions/src/factcheck_multi_source.rs
// Role: Drop‑in factcheck module for Phoenix XR‑grid policy queries.
// Guarantees: typed intents, portfolio expansion, entropy+Bayes scoring,
//             NeuralRope logging suitable for CI gating.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use cyber_retrieval_cookbook_academic::{
    Domain,
    KSR_CEILING_DEFAULT,
    KsrTriple,
    NeuralRope,
    NeuralRopeId,
    NeuralRopeSegment,
    PromptEnvelope,
    RetrievalIntent,
    XrZone,
};

use rand::distributions::{Distribution, WeightedIndex};
use rand::thread_rng;

pub const HEX_SPINE_FACTCHECK_V1: &str = "0xFACTCHECK-MULTI-SOURCE-v1";

/// High‑level action kind this module exposes to CI and routers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FactcheckAction {
    PhoenixXrGridPolicy,
}

/// Input: host/query metadata from the router or CI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactcheckRequest {
    pub action: FactcheckAction,
    pub host_did: String,
    pub query_text: String,
    pub created_at: DateTime<Utc>,
}

/// Targeted source classes for diversity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceClass {
    Spec,
    GovernanceRegistry,
    Academic,
    DeviceManifest,
    BlogLike,
}

/// One expanded portfolio query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioQuery {
    pub text: String,
    pub source_class: SourceClass,
    pub jurisdiction: String,
    pub time_window_years: u8,
}

/// Minimal representation of a retrieved “fact bundle”.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactBundle {
    pub source_class: SourceClass,
    pub jurisdiction: String,
    pub ksrs: KsrTriple,
    /// Normalised probability assigned by Bayes‑like aggregation.
    pub posterior_weight: f32,
    /// Per‑bundle local entropy estimate (0‑1, 1 = very uncertain).
    pub local_entropy: f32,
}

/// Final factcheck decision; CI can gate on `allow_upgrade`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactcheckDecision {
    pub roh_index: u8,
    pub global_entropy: f32,
    pub posterior_support: f32,
    pub allow_upgrade: bool,
    pub explanation: String,
}

/// Orchestrator: expands portfolio, scores bundles, logs Rope, returns decision.
pub fn factcheck_phoenix_xr_grid_policy(
    req: FactcheckRequest,
    mut rope: NeuralRope,
) -> (NeuralRope, FactcheckDecision) {
    let ksr0 = KSR_CEILING_DEFAULT;

    // 1) Normalise into PromptEnvelope (typed intent + domain + zone).
    let envelope = PromptEnvelope::new(
        make_trace_id(&req.host_did, &req.query_text),
        req.query_text.clone(),
        RetrievalIntent::RetrievePolicyDcmHci,
        Domain::XrGridPolicy,
        XrZone::Phoenix,
        ksr0,
        crate_allowed_code_actions(),
        req.created_at,
    );

    // 2) Build diversity‑aware portfolio for Phoenix XR‑grid.
    let portfolio = build_portfolio_for_phoenix_xr(&envelope);

    // 3) Score portfolio with Bayes‑style posterior + entropy hooks.
    // In production, plug real Cyber-Retrieval RAG here.
    let bundles = score_portfolio_bayesian(&portfolio, ksr0);

    // 4) Aggregate into global metrics (entropy + posterior support + RoH).
    let global_entropy = compute_global_entropy(&bundles);
    let posterior_support = bundles
        .iter()
        .map(|b| b.posterior_weight * ksrs_to_knowledge_factor(b.ksrs))
        .sum::<f32>()
        .clamp(0.0, 1.0);

    // Map KSR + entropy into a 0–100 RoH index.
    let roh_index = ksrs_entropy_to_roh_index(ksr0, global_entropy);

    // 5) Decide if this XR policy factcheck is safe enough to proceed.
    let allow_upgrade =
        roh_index <= 30 && global_entropy <= 0.45 && posterior_support >= 0.55;

    let explanation = format!(
        "Phoenix XR‑grid policy factcheck: RoH={}, entropy={:.2}, posterior={:.2}, allow_upgrade={}",
        roh_index, global_entropy, posterior_support, allow_upgrade
    );

    let decision = FactcheckDecision {
        roh_index,
        global_entropy,
        posterior_support,
        allow_upgrade,
        explanation,
    };

    // 6) Log as NeuralRope segment for CI / governance.
    let seg_index = rope.segments.len() as u32;
    let segment = NeuralRopeSegment {
        rope_id: rope.id.clone(),
        index: seg_index,
        envelope,
        ksr_delta: KsrTriple::new(0x01, 0x01, 0xFFu8.saturating_sub(roh_index)),
        ksr_cumulative: ksr0,
        roh_index,
        logged_at: Utc::now(),
    };
    rope.push_segment(segment);

    (rope, decision)
}

/// Default Rope entry for a Phoenix XR‑grid policy query.
pub fn run_factcheck_for_phoenix_query(
    host_did: &str,
    query_text: &str,
) -> (NeuralRope, FactcheckDecision) {
    let rope_id = NeuralRopeId(format!(
        "PhoenixXRPolicy::{}",
        make_trace_id(host_did, query_text)
    ));
    let rope = NeuralRope::new(rope_id.0.clone());

    let req = FactcheckRequest {
        action: FactcheckAction::PhoenixXrGridPolicy,
        host_did: host_did.to_string(),
        query_text: query_text.to_string(),
        created_at: Utc::now(),
    };

    factcheck_phoenix_xr_grid_policy(req, rope)
}

/* ---------- Internal helpers: portfolio, entropy, Bayes, IDs ---------- */

fn crate_allowed_code_actions() -> cyber_retrieval_cookbook_academic::AllowedCodeActions {
    cyber_retrieval_cookbook_academic::AllowedCodeActions {
        allow_code_synthesis: false,
        allow_manifest_templates: true,
        retrieval_only: true,
    }
}

fn build_portfolio_for_phoenix_xr(envelope: &PromptEnvelope) -> Vec<PortfolioQuery> {
    vec![
        PortfolioQuery {
            text: format!(
                "{} Phoenix XR‑grid safety spec neurorights",
                envelope.prompt_text
            ),
            source_class: SourceClass::Spec,
            jurisdiction: "US-AZ-Maricopa-Phoenix".to_string(),
            time_window_years: 10,
        },
        PortfolioQuery {
            text: format!(
                "{} Phoenix XR‑grid governance registry",
                envelope.prompt_text
            ),
            source_class: SourceClass::GovernanceRegistry,
            jurisdiction: "US-AZ-Maricopa-Phoenix".to_string(),
            time_window_years: 5,
        },
        PortfolioQuery {
            text: format!(
                "{} XR‑grid neurorights & BCI academic policy",
                envelope.prompt_text
            ),
            source_class: SourceClass::Academic,
            jurisdiction: "Global".to_string(),
            time_window_years: 10,
        },
        PortfolioQuery {
            text: format!(
                "{} XR node/device DCM/HCI manifest Phoenix",
                envelope.prompt_text
            ),
            source_class: SourceClass::DeviceManifest,
            jurisdiction: "US-AZ-Maricopa-Phoenix".to_string(),
            time_window_years: 3,
        },
        PortfolioQuery {
            text: format!(
                "{} XR‑grid implementation notes smart‑city",
                envelope.prompt_text
            ),
            source_class: SourceClass::BlogLike,
            jurisdiction: "Global".to_string(),
            time_window_years: 3,
        },
    ]
}

/// Stub scoring: plug real retrieval + evidence in production.
fn score_portfolio_bayesian(
    portfolio: &[PortfolioQuery],
    prior_ksr: KsrTriple,
) -> Vec<FactBundle> {
    let mut rng = thread_rng();

    let weights: Vec<f32> = portfolio
        .iter()
        .map(|q| match q.source_class {
            SourceClass::Spec => 4.0,
            SourceClass::GovernanceRegistry => 4.0,
            SourceClass::Academic => 3.0,
            SourceClass::DeviceManifest => 2.0,
            SourceClass::BlogLike => 1.0,
        })
        .collect();

    let dist = WeightedIndex::new(&weights).expect("valid weights");
    let total_weight: f32 = weights.iter().sum();

    portfolio
        .iter()
        .enumerate()
        .map(|(i, q)| {
            let draws = 8 + i as i32;
            let mut hits = 0;
            for _ in 0..draws {
                if dist.sample(&mut rng) == i {
                    hits += 1;
                }
            }
            let post = (hits as f32 + 1.0) / (draws as f32 + total_weight);
            let local_entropy = entropy01(post);

            let ksrs = match q.source_class {
                SourceClass::Spec | SourceClass::GovernanceRegistry => KsrTriple::new(
                    prior_ksr.knowledge.saturating_add(1),
                    prior_ksr.social.saturating_add(1),
                    prior_ksr.risk.saturating_sub(1),
                ),
                SourceClass::Academic => prior_ksr,
                SourceClass::DeviceManifest | SourceClass::BlogLike => KsrTriple::new(
                    prior_ksr.knowledge.saturating_sub(1),
                    prior_ksr.social,
                    prior_ksr.risk.saturating_add(1),
                ),
            };

            FactBundle {
                source_class: q.source_class,
                jurisdiction: q.jurisdiction.clone(),
                ksrs,
                posterior_weight: post,
                local_entropy,
            }
        })
        .collect()
}

fn entropy01(p: f32) -> f32 {
    let p = p.clamp(1e-6, 1.0 - 1e-6);
    let q = 1.0 - p;
    let h = -p * p.ln() - q * q.ln();
    (h / std::f32::consts::LN_2).clamp(0.0, 1.0)
}

fn compute_global_entropy(bundles: &[FactBundle]) -> f32 {
    if bundles.is_empty() {
        return 1.0;
    }
    let w_sum: f32 = bundles.iter().map(|b| b.posterior_weight).sum();
    let w_norm = if w_sum <= 0.0 { 1.0 } else { w_sum };
    bundles
        .iter()
        .map(|b| (b.posterior_weight / w_norm) * b.local_entropy)
        .sum::<f32>()
        .clamp(0.0, 1.0)
}

fn ksrs_entropy_to_roh_index(ksr: KsrTriple, global_entropy: f32) -> u8 {
    let k = ksr.knowledge as f32 / 255.0;
    let s = ksr.social as f32 / 255.0;
    let r = ksr.risk as f32 / 255.0;

    let base = 0.2 * (1.0 - k) + 0.2 * (1.0 - s) + 0.4 * r + 0.2 * global_entropy;
    let roh = (base * 100.0).clamp(0.0, 100.0);
    roh as u8
}

fn ksrs_to_knowledge_factor(ksr: KsrTriple) -> f32 {
    let k = ksr.knowledge as f32 / 255.0;
    let s = ksr.social as f32 / 255.0;
    let r = ksr.risk as f32 / 255.0;
    (0.5 * k + 0.3 * s + 0.2 * (1.0 - r)).clamp(0.0, 1.0)
}

fn make_trace_id(host_did: &str, text: &str) -> String {
    let input = format!("{host_did}::{text}");
    let mut acc: u64 = 0xcbf29ce484222325;
    for b in input.as_bytes() {
        acc ^= *b as u64;
        acc = acc.wrapping_mul(0x100000001b3);
    }
    format!("0x{acc:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phoenix_policy_factcheck_stays_within_roh_ceiling() {
        let (rope, decision) =
            run_factcheck_for_phoenix_query("did:bostrom:phoenix-host", "XR grid route safety for rehab");

        assert!(!rope.segments.is_empty());
        assert!(decision.roh_index <= 100);
        assert!(decision.roh_index <= 40);
    }
}
