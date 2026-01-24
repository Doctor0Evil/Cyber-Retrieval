"use strict";

/**
 * K/S/R + RoH utilities
 */

const MAX_ROH = 0.3;

class KsrScore {
  constructor(kHex, sHex, rHex) {
    this.k = kHex; // e.g. "0xE2"
    this.s = sHex; // e.g. "0x78"
    this.r = rHex; // e.g. "0x27"
  }
}

function estimateRoH(ksr) {
  // Purely symbolic mapping for safety-governed stacks.
  // Real systems can plug in calibrated mappings; we clamp to <= 1.0.
  const k = parseInt(ksr.k, 16);
  const s = parseInt(ksr.s, 16);
  const r = parseInt(ksr.r, 16);
  const riskNorm = r / 0xFF;
  const safetyBoost = (k + s) / (2 * 0xFF);
  const roh = Math.min(1.0, Math.max(0, riskNorm * (1.0 - 0.2 * safetyBoost)));
  return roh;
}

/**
 * 1. RetrievalIntent enum and domain typing
 */

const RetrievalIntent = Object.freeze({
  RETRIEVE_KNOWLEDGE: "RetrieveKnowledge",
  THREAT_SCAN: "ThreatScan",
  RETRIEVE_POLICY_DCM_HCI: "RetrievePolicyDcmHci",
  NEURAL_ROPE_RESEARCH: "NeuralRopeResearch",
});

const RetrievalDomain = Object.freeze({
  DCM_HCI: "DcmHci",
  XR_GRID: "XrGrid",
  RUST_WIRING: "RustWiring",
  DID_REGISTRY: "DidRegistry",
  GOVERNANCE: "Governance",
});

const XrZone = Object.freeze({
  PHOENIX: "Phoenix",
  SAN_JOLLA: "SanJolla",
  ECO: "Eco",
  UNKNOWN: "Unknown",
});

/**
 * 2. PromptEnvelope and router gating
 */

class PromptEnvelope {
  constructor({
    traceId,
    userDid,
    intent,
    domain,
    xrZone,
    ksRestimate,
    allowedCodeActions,
    rawPrompt,
    metadata,
  }) {
    this.traceId = traceId; // deterministic string
    this.userDid = userDid; // DID / Bostrom / ALN id
    this.intent = intent; // RetrievalIntent.*
    this.domain = domain; // RetrievalDomain.*
    this.xrZone = xrZone; // XrZone.*
    this.ksRestimate = ksRestimate; // KsrScore
    this.allowedCodeActions = Object.freeze([...allowedCodeActions]);
    this.rawPrompt = rawPrompt;
    this.metadata = Object.freeze({ ...(metadata || {}) });
  }
}

function normalizeRetrievalPrompt(raw) {
  // raw: { userDid, text, domainHint?, xrZoneHint? }
  const { userDid, text, domainHint, xrZoneHint } = raw;
  const traceId = makeTraceId(userDid, text);
  const intent = inferIntentFromText(text);
  const domain = domainHint || inferDomainFromText(text);
  const xrZone = xrZoneHint || XrZone.UNKNOWN;

  const ksRestimate = new KsrScore("0xE2", "0x78", "0x27");
  const allowedCodeActions = inferAllowedCodeActions(intent, domain);

  return new PromptEnvelope({
    traceId,
    userDid,
    intent,
    domain,
    xrZone,
    ksRestimate,
    allowedCodeActions,
    rawPrompt: text,
    metadata: {
      createdAtIso: new Date().toISOString(),
      hexSpine: "0xKSR-RETRIEVAL-IMPROVE-v1",
    },
  });
}

function inferIntentFromText(text) {
  const lower = text.toLowerCase();
  if (lower.includes("threat") || lower.includes("risk scan")) {
    return RetrievalIntent.THREAT_SCAN;
  }
  if (lower.includes("policy") || lower.includes("dcm") || lower.includes("hci")) {
    return RetrievalIntent.RETRIEVE_POLICY_DCM_HCI;
  }
  if (lower.includes("rope") || lower.includes("neuralrope") || lower.includes("research plan")) {
    return RetrievalIntent.NEURAL_ROPE_RESEARCH;
  }
  return RetrievalIntent.RETRIEVE_KNOWLEDGE;
}

function inferDomainFromText(text) {
  const lower = text.toLowerCase();
  if (lower.includes("dcm") || lower.includes("hci")) return RetrievalDomain.DCM_HCI;
  if (lower.includes("xr") || lower.includes("zone")) return RetrievalDomain.XR_GRID;
  if (lower.includes("rust") || lower.includes("crate")) return RetrievalDomain.RUST_WIRING;
  if (lower.includes("did") || lower.includes("registry") || lower.includes("bostrom")) {
    return RetrievalDomain.DID_REGISTRY;
  }
  if (lower.includes("governance") || lower.includes("policy")) return RetrievalDomain.GOVERNANCE;
  return RetrievalDomain.DCM_HCI;
}

function inferAllowedCodeActions(intent, domain) {
  const base = ["ReadOnlySearch"];
  if (intent === RetrievalIntent.RETRIEVE_POLICY_DCM_HCI || intent === RetrievalIntent.NEURAL_ROPE_RESEARCH) {
    base.push("GenerateMarkdownSummary");
  }
  if (domain === RetrievalDomain.RUST_WIRING) {
    base.push("GenerateRustStub");
  }
  return base;
}

function makeTraceId(userDid, text) {
  const input = `${userDid}::${text}`;
  let acc = 0xcbf29ce484222325n;
  const FNV_PRIME = 0x100000001b3n;
  for (let i = 0; i < input.length; i++) {
    acc ^= BigInt(input.charCodeAt(i));
    acc *= FNV_PRIME;
    acc &= (1n << 64n) - 1n;
  }
  return "0x" + acc.toString(16).padStart(16, "0");
}

class RetrievalRouter {
  constructor({ rohCeiling = MAX_ROH } = {}) {
    this.rohCeiling = rohCeiling;
  }

  // Throws if envelope is structurally invalid or RoH > ceiling
  validateEnvelopeForRetrieval(envelope) {
    if (!envelope.intent || !envelope.domain) {
      throw new Error("RetrievalRouter: missing intent/domain – forbidden.");
    }
    const roh = estimateRoH(envelope.ksRestimate);
    if (roh > this.rohCeiling) {
      const msg = `RetrievalRouter: RoH=${roh.toFixed(
        3
      )} exceeds ceiling=${this.rohCeiling.toFixed(3)} – retrieval denied.`;
      throw new Error(msg);
    }
    return roh;
  }
}

/**
 * 3. Diversity-aware retrieval portfolio
 */

const SourceType = Object.freeze({
  SPEC: "Spec",
  ACADEMIC: "Academic",
  MANIFEST: "Manifest",
  GOVERNANCE: "Governance",
  BLOG: "Blog",
});

class RetrievalQuery {
  constructor({ query, sourceType, jurisdiction, timeWindow }) {
    this.query = query;
    this.sourceType = sourceType;
    this.jurisdiction = jurisdiction;
    this.timeWindow = timeWindow;
  }
}

function buildDiversityPortfolio(envelope) {
  const baseTokens = [envelope.rawPrompt, envelope.domain, envelope.xrZone].join(" ");
  const portfolio = [];

  portfolio.push(
    new RetrievalQuery({
      query: baseTokens + " neurorights DCM/HCI spec",
      sourceType: SourceType.SPEC,
      jurisdiction: "Global",
      timeWindow: "5y",
    }),
    new RetrievalQuery({
      query: baseTokens + " XR-Grid governance",
      sourceType: SourceType.GOVERNANCE,
      jurisdiction: "Phoenix",
      timeWindow: "5y",
    }),
    new RetrievalQuery({
      query: baseTokens + " peer-reviewed",
      sourceType: SourceType.ACADEMIC,
      jurisdiction: "Global",
      timeWindow: "10y",
    }),
    new RetrievalQuery({
      query: baseTokens + " device capability manifest",
      sourceType: SourceType.MANIFEST,
      jurisdiction: "Any",
      timeWindow: "current",
    }),
    new RetrievalQuery({
      query: baseTokens + " implementation notes",
      sourceType: SourceType.BLOG,
      jurisdiction: "Any",
      timeWindow: "3y",
    })
  );

  return portfolio;
}

/**
 * 4. NeuralRope representation and pacing
 */

class RopeSegment {
  constructor({ index, envelope, rohBefore, rohAfter, ksRestimate }) {
    this.index = index;
    this.envelope = envelope;
    this.rohBefore = rohBefore;
    this.rohAfter = rohAfter;
    this.ksRestimate = ksRestimate;
    this.timestampIso = new Date().toISOString();
  }
}

class NeuralRope {
  constructor({ ropeId, userDid }) {
    this.ropeId = ropeId;
    this.userDid = userDid;
    this.segments = [];
    this.maxHighRiskSegments = 3;
  }

  appendSegment(segment) {
    this.segments.push(segment);
  }

  countHighRiskSegments(threshold = 0.25) {
    return this.segments.filter((s) => s.rohAfter >= threshold).length;
  }

  shouldInsertCooldown(rohAfter) {
    const highRiskCount = this.countHighRiskSegments();
    if (rohAfter >= 0.25 && highRiskCount >= this.maxHighRiskSegments) return true;
    if (rohAfter >= MAX_ROH * 0.9) return true;
    return false;
  }

  makeCooldownSegment() {
    const last = this.segments[this.segments.length - 1];
    const text = "Summarize and cool down: recap key facts, highlight open questions, no new high-density material.";
    const envelope = normalizeRetrievalPrompt({
      userDid: this.userDid,
      text,
      domainHint: RetrievalDomain.GOVERNANCE,
      xrZoneHint: XrZone.UNKNOWN,
    });
    const ksRestimate = new KsrScore("0xE1", "0x79", "0x20");
    const rohBefore = estimateRoH(last.ksRestimate);
    const rohAfter = estimateRoH(ksRestimate);
    return new RopeSegment({
      index: this.segments.length,
      envelope,
      rohBefore,
      rohAfter,
      ksRestimate,
    });
  }
}

/**
 * 5. quiz_math metrics for batch validation
 */

class QuizMathResult {
  constructor({
    agreementScore,
    unitConsistencyScore,
    governanceCompatibilityScore,
    overallKsr,
    passed,
  }) {
    this.agreementScore = agreementScore; // 0..1
    this.unitConsistencyScore = unitConsistencyScore; // 0..1
    this.governanceCompatibilityScore = governanceCompatibilityScore; // 0..1
    this.overallKsr = overallKsr; // KsrScore
    this.passed = passed;
  }
}

function runQuizMath(candidates) {
  // candidates: [{ id, ksr: KsrScore, unitsValid, matchesGovernance }]
  if (!candidates.length) {
    return new QuizMathResult({
      agreementScore: 0,
      unitConsistencyScore: 0,
      governanceCompatibilityScore: 0,
      overallKsr: new KsrScore("0x00", "0x00", "0xFF"),
      passed: false,
    });
  }

  const avgK =
    candidates.reduce((acc, c) => acc + parseInt(c.ksr.k, 16), 0) / candidates.length;
  const avgS =
    candidates.reduce((acc, c) => acc + parseInt(c.ksr.s, 16), 0) / candidates.length;
  const avgR =
    candidates.reduce((acc, c) => acc + parseInt(c.ksr.r, 16), 0) / candidates.length;
  const overallKsr = new KsrScore(
    "0x" + Math.round(avgK).toString(16),
    "0x" + Math.round(avgS).toString(16),
    "0x" + Math.round(avgR).toString(16)
  );

  const unitConsistencyScore =
    candidates.filter((c) => c.unitsValid).length / candidates.length;
  const governanceCompatibilityScore =
    candidates.filter((c) => c.matchesGovernance).length / candidates.length;

  // Simple agreement proxy: how many share the modal hex K?
  const kCounts = {};
  for (const c of candidates) {
    kCounts[c.ksr.k] = (kCounts[c.ksr.k] || 0) + 1;
  }
  const maxAgree = Math.max(...Object.values(kCounts));
  const agreementScore = maxAgree / candidates.length;

  const roh = estimateRoH(overallKsr);
  const passed =
    agreementScore >= 0.6 &&
    unitConsistencyScore >= 0.7 &&
    governanceCompatibilityScore >= 0.7 &&
    roh <= MAX_ROH;

  return new QuizMathResult({
    agreementScore,
    unitConsistencyScore,
    governanceCompatibilityScore,
    overallKsr,
    passed,
  });
}

function gateDataCreation(quizResult) {
  if (!quizResult.passed) {
    const roh = estimateRoH(quizResult.overallKsr);
    return {
      allowed: false,
      reason: `quiz_math gate: agreement=${quizResult.agreementScore.toFixed(
        2
      )}, units=${quizResult.unitConsistencyScore.toFixed(
        2
      )}, governance=${quizResult.governanceCompatibilityScore.toFixed(
        2
      )}, RoH=${roh.toFixed(3)} – below thresholds or RoH too high.`,
    };
  }
  return {
    allowed: true,
    reason: "quiz_math gate passed – safe to generate manifests / stubs / policy drafts.",
  };
}

/**
 * 6. Cookbook wiring helper
 */

class CookbookLaneConfig {
  constructor({
    laneId,
    allowedIntents,
    maxRopeLength,
    rohCeiling,
    quizMathThresholds,
    ksRestimate,
  }) {
    this.laneId = laneId;
    this.allowedIntents = Object.freeze([...allowedIntents]);
    this.maxRopeLength = maxRopeLength;
    this.rohCeiling = rohCeiling;
    this.quizMathThresholds = Object.freeze(quizMathThresholds);
    this.ksRestimate = ksRestimate;
  }
}

const DefaultCookbookLane = new CookbookLaneConfig({
  laneId: "CyberRetrieval-NeuralRope-01",
  allowedIntents: [
    RetrievalIntent.RETRIEVE_KNOWLEDGE,
    RetrievalIntent.RETRIEVE_POLICY_DCM_HCI,
    RetrievalIntent.NEURAL_ROPE_RESEARCH,
  ],
  maxRopeLength: 32,
  rohCeiling: MAX_ROH,
  quizMathThresholds: {
    minAgreement: 0.6,
    minUnitConsistency: 0.7,
    minGovernanceCompatibility: 0.7,
  },
  ksRestimate: new KsrScore("0xE2", "0x78", "0x27"),
});

function checkLaneCompatibility(envelope, lane = DefaultCookbookLane) {
  if (!lane.allowedIntents.includes(envelope.intent)) {
    return {
      ok: false,
      reason: `Lane ${lane.laneId} does not allow intent ${envelope.intent}.`,
    };
  }
  const roh = estimateRoH(envelope.ksRestimate);
  if (roh > lane.rohCeiling) {
    return {
      ok: false,
      reason: `Lane ${lane.laneId} RoH=${roh.toFixed(
        3
      )} exceeds ceiling=${lane.rohCeiling.toFixed(3)}.`,
    };
  }
  return { ok: true, reason: "Lane compatibility OK." };
}

/**
 * Export surface
 */

module.exports = {
  // Enums / types
  RetrievalIntent,
  RetrievalDomain,
  XrZone,
  SourceType,
  KsrScore,
  PromptEnvelope,
  RetrievalRouter,
  RetrievalQuery,
  NeuralRope,
  RopeSegment,
  QuizMathResult,
  CookbookLaneConfig,
  DefaultCookbookLane,

  // Core functions
  normalizeRetrievalPrompt,
  buildDiversityPortfolio,
  estimateRoH,
  runQuizMath,
  gateDataCreation,
  checkLaneCompatibility,
};
