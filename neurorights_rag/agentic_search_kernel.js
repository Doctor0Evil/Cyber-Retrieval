// File: neurorights_rag/agentic_search_kernel.js
// Platform: Windows/Linux/Ubuntu, Android/iOS (Node.js or Deno compatible)
// Language: Javascript (sanitized, production-grade)
//
// Purpose:
// - Implement governed, rule-based research turns for AI-chat search/actions.
// - Encode PromptEnvelope + NeuralRope with KSR + RoH≤0.3 policy.
// - Provide diversity-aware retrieval planning + Hash-RAG–style routing [web:38][web:41].
// - Add quiz_math metrics to score search quality (entropy, agreement, compression).
// - Add malware-safe ingestion filters for web/RAG link ingestion [web:39][web:42].
// - Provide multiple research-action playbooks for better user experiences.
// - Designed to be wired under any LLM-chat stack as a “research spine”.
//
// Notes:
// - No Python.
// - All enums/objects are fully filled.
// - All logic is deterministic and debuggable via console-style telemetry.

"use strict";

/* ===========================
   1. Core Types & Constants
   =========================== */

/**
 * @typedef {"RetrieveKnowledge"|"ThreatScan"|"RetrievePolicy"|"NeuralRopeResearch"|
 *           "FactCheck"|"CompareViews"|"TimelineScan"|"DatasetDiscovery"|
 *           "CodeSecurityScan"|"ExploitIntelScan"|"SourceMapping"|"MetaReview"} PromptIntentId
 *
 * Extra intents:
 * - FactCheck: focused verification of specific claims.
 * - CompareViews: explicitly collect diverse viewpoints.
 * - TimelineScan: prioritize time-ordered docs.
 * - DatasetDiscovery: locate obscure/hidden-but-public docs.
 * - CodeSecurityScan / ExploitIntelScan: malware/vuln-aware scans [web:33][web:35][web:37].
 * - SourceMapping: map citations/refs back to canonical URLs.
 * - MetaReview: evaluate reliability/evidence of a result cluster.
 */

/**
 * @typedef {"academic"|"policy"|"security"|"ecosystem"|"dcm_hci"|"xrgrid_policy"|
 *           "rust_wiring"|"did_registry"|"code_artifact"|"malware_intel"|
 *           "product_docs"|"gov_records"|"open_standards"} PromptDomainId
 */

/**
 * @typedef {"citizen.v1"|"researcher.v1"|"ops.v1"} NeurorightsProfileId
 */

/**
 * @typedef {"low"|"medium"|"high"} KnowledgeFactorBand
 * @typedef {"none"|"low"|"medium"|"high"} SocialImpactBand
 * @typedef {"0.0"|"0.1"|"0.2"|"0.3"} RiskOfHarmBand
 */

/**
 * @typedef {Object} IdentityTriple
 * @property {string} did
 * @property {string} aln_shard
 * @property {string} bostrom_addr_primary
 * @property {string[]} bostrom_addr_alternates
 */

/**
 * @typedef {Object} KSRBundle
 * @property {KnowledgeFactorBand} knowledge
 * @property {SocialImpactBand} socialImpact
 * @property {RiskOfHarmBand} riskOfHarm
 * @property {number} rohFloat   // 0.0–0.3 invariant at runtime
 */

/**
 * @typedef {Object} PromptEnvelope
 * @property {string} hexStamp           // governance tag
 * @property {PromptIntentId} intent
 * @property {PromptDomainId} domain
 * @property {string} userText
 * @property {string[]} clarifyingQuestions
 * @property {IdentityTriple} identity
 * @property {NeurorightsProfileId} neurorightsProfile
 * @property {KSRBundle} ksr
 * @property {boolean} allowWeb
 * @property {boolean} allowCodeExecution
 * @property {boolean} allowFileFetch
 * @property {string[]} explicitConstraints   // e.g. "RoH<=0.3","NoBiometricInference"
 * @property {string[]} preferredJurisdictions
 * @property {string[]} disallowedJurisdictions
 * @property {string[]} mustIncludeSources    // schemas: domain or hostname patterns
 * @property {string[]} mustAvoidSources
 * @property {Record<string,string>} routerHints
 */

/**
 * @typedef {Object} RopeStepTelemetry
 * @property {string} traceId
 * @property {string} stepId
 * @property {PromptIntentId} intent
 * @property {PromptDomainId} domain
 * @property {string} toolId
 * @property {string} query
 * @property {string[]} urls
 * @property {KSRBundle} ksr
 * @property {Object} quizMath
 * @property {Object} securityFlags
 */

/**
 * @typedef {Object} NeuralRope
 * @property {string} ropeId
 * @property {NeurorightsProfileId} profile
 * @property {KSRBundle} cumulativeKsr
 * @property {RopeStepTelemetry[]} steps
 */

/**
 * @typedef {"RawSearch"|"RerankBM25"|"DeepHash"|"VectorStore"|"HashRAG"} RetrievalModeId
 */

/**
 * @typedef {Object} ResearchActionPlan
 * @property {PromptEnvelope} envelope
 * @property {RetrievalModeId} retrievalMode
 * @property {string[]} searchQueries
 * @property {Object[]} queryPorts // each describes how to execute a query
 * @property {boolean} scheduleDiversityPortfolio
 * @property {boolean} scheduleDeDuplication
 * @property {boolean} scheduleQuizMath
 * @property {boolean} scheduleMalwareGuards
 */

/**
 * @typedef {Object} MalwareFilterConfig
 * @property {boolean} blockExecutableLinks
 * @property {boolean} blockSuspiciousTlds
 * @property {boolean} blockObfuscatedText
 * @property {boolean} requireHttps
 * @property {boolean} stripInvisibleContent
 * @property {boolean} useFormatBreaker
 * @property {string[]} blockedTlds
 */

/**
 * @typedef {Object} SearchDocument
 * @property {string} url
 * @property {string} title
 * @property {string} snippet
 * @property {string} fullText
 * @property {string[]} tags
 */

/**
 * @typedef {Object} SearchBatchResult
 * @property {SearchDocument[]} documents
 * @property {Object} hashIndex // hash -> indices
 */

/**
 * @typedef {Object} QuizMathScores
 * @property {number} entropyOfEvidence
 * @property {number} crossSourceAgreement
 * @property {number} compressionRatio
 * @property {number} coverageScore
 */

/* Hard safety ceiling: RoH <= 0.3 across whole rope */
const ROH_CEILING = 0.3;

/* ===========================
   2. Utility & Hashing
   =========================== */

function vcNowMs() {
  return Date.now();
}

function vcGenerateHexStamp() {
  const ts = vcNowMs().toString(16);
  const rand = Math.floor(Math.random() * 0xffffffff)
    .toString(16)
    .padStart(8, "0");
  return `HEX-${ts}-${rand}`;
}

function vcGenerateId(prefix) {
  const ts = vcNowMs().toString(16);
  const rand = Math.floor(Math.random() * 0xffffffff)
    .toString(16)
    .padStart(8, "0");
  return `${prefix}-${ts}-${rand}`;
}

/**
 * Simple, deterministic 32-bit FNV-1a hash -> hex string.
 * Used for content hashing / Hash-RAG style index keys [web:38][web:41].
 *
 * @param {string} input
 * @returns {string}
 */
function vcHash32(input) {
  let hash = 0x811c9dc5;
  for (let i = 0; i < input.length; i++) {
    hash ^= input.charCodeAt(i) & 0xff;
    hash = (hash * 0x01000193) >>> 0;
  }
  return hash.toString(16).padStart(8, "0");
}

/**
 * Very lightweight SimHash-style binary signature for de-dup [web:38][web:41].
 *
 * @param {string} text
 * @returns {string} 64-bit bitstring
 */
function vcSimHash64(text) {
  const tokens = text
    .toLowerCase()
    .replace(/[^a-z0-9\s]/g, " ")
    .split(/\s+/)
    .filter(Boolean);
  const v = new Array(64).fill(0);
  for (let i = 0; i < tokens.length; i++) {
    const h = BigInt("0x" + vcHash32(tokens[i]) + vcHash32(tokens[i].split("").reverse().join("")));
    for (let bit = 0n; bit < 64n; bit++) {
      const mask = 1n << bit;
      const one = (h & mask) !== 0n;
      v[Number(bit)] += one ? 1 : -1;
    }
  }
  let result = "";
  for (let i = 0; i < 64; i++) {
    result += v[i] >= 0 ? "1" : "0";
  }
  return result;
}

/* ===========================
   3. Envelope Builder & Validator
   =========================== */

/**
 * Build a PromptEnvelope from free-form user text and context.
 *
 * @param {Object} opts
 * @returns {PromptEnvelope}
 */
function buildPromptEnvelope(opts) {
  const intent = opts.intent;
  const domain = opts.domain;
  const roh = clampRoh(opts.ksr?.rohFloat ?? 0.1);
  /** @type {PromptEnvelope} */
  const env = {
    hexStamp: vcGenerateHexStamp(),
    intent,
    domain,
    userText: sanitizeUserText(opts.userText || ""),
    clarifyingQuestions: [],
    identity: {
      did: opts.identity?.did || "did:example:anonymous",
      aln_shard: opts.identity?.aln_shard || "aln:anon",
      bostrom_addr_primary:
        opts.identity?.bostrom_addr_primary ||
        "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
      bostrom_addr_alternates:
        opts.identity?.bostrom_addr_alternates || [
          "bostrom1ldgmtf20d6604a24ztr0jxht7xt7az4jhkmsrc",
        ],
    },
    neurorightsProfile: opts.neurorightsProfile || "citizen.v1",
    ksr: {
      knowledge: opts.ksr?.knowledge || "low",
      socialImpact: opts.ksr?.socialImpact || "low",
      riskOfHarm: rohToBand(roh),
      rohFloat: roh,
    },
    allowWeb: opts.allowWeb !== false,
    allowCodeExecution: opts.allowCodeExecution === true && roh <= 0.2,
    allowFileFetch: opts.allowFileFetch !== false,
    explicitConstraints: [
      "RoH<=0.3",
      "NoBiometricInference",
      "NoBiometricProfiling",
      "NoMedicalDiagnosis",
    ],
    preferredJurisdictions: opts.preferredJurisdictions || ["US", "EU"],
    disallowedJurisdictions: opts.disallowedJurisdictions || [],
    mustIncludeSources: opts.mustIncludeSources || [],
    mustAvoidSources: opts.mustAvoidSources || ["pastebin.com", "darkweb"],
    routerHints: opts.routerHints || {},
  };

  return validatePromptEnvelope(env);
}

function sanitizeUserText(input) {
  if (typeof input !== "string") return "";
  let out = input.replace(/[\u0000-\u001f\u007f]/g, " ");
  out = out.replace(/\s+/g, " ").trim();
  if (out.length > 8000) {
    out = out.slice(0, 8000);
  }
  return out;
}

function clampRoh(v) {
  if (typeof v !== "number" || Number.isNaN(v)) return 0.1;
  if (v < 0) return 0.0;
  if (v > ROH_CEILING) return ROH_CEILING;
  return parseFloat(v.toFixed(2));
}

function rohToBand(roh) {
  if (roh <= 0.0) return "0.0";
  if (roh <= 0.1) return "0.1";
  if (roh <= 0.2) return "0.2";
  return "0.3";
}

/**
 * Enforce invariants on PromptEnvelope.
 *
 * @param {PromptEnvelope} env
 * @returns {PromptEnvelope}
 */
function validatePromptEnvelope(env) {
  if (env.ksr.rohFloat > ROH_CEILING) {
    env.ksr.rohFloat = ROH_CEILING;
    env.ksr.riskOfHarm = rohToBand(env.ksr.rohFloat);
  }

  if (env.intent === "ThreatScan" || env.intent === "ExploitIntelScan") {
    env.allowCodeExecution = false;
  }

  if (env.neurorightsProfile === "citizen.v1") {
    env.explicitConstraints.push("NoTargetedManipulation");
  }

  return env;
}

/* ===========================
   4. Research-Action Variants
   =========================== */

/**
 * Build a ResearchActionPlan with a diversity-aware portfolio.
 *
 * Diversity dimensions:
 * - query shape (broad, focused, meta).
 * - time hints (latest, historical).
 * - source type hints (academic, policy, vendor, security) [web:42][web:39].
 * - retrieval mode (HashRAG/Vector/RawSearch) [web:38][web:41].
 *
 * @param {PromptEnvelope} env
 * @returns {ResearchActionPlan}
 */
function planResearchActions(env) {
  const baseQ = env.userText;

  /** @type {string[]} */
  const searchQueries = [];

  // 1) Broad semantic sweep
  searchQueries.push(baseQ);

  // 2) Focused clarifications
  searchQueries.push(`${baseQ} academic PDF`);
  searchQueries.push(`${baseQ} official documentation site:.gov OR site:.edu`);
  searchQueries.push(`${baseQ} security risks malware RAG ingestion`); // [web:39][web:42]

  // 3) Policy/standards if relevant
  if (env.intent === "RetrievePolicy" || env.domain === "policy") {
    searchQueries.push(`${baseQ} standard OR guideline site:oecd.org OR site:unesco.org`);
  }

  // 4) Hidden/public docs discovery (DatasetDiscovery lane)
  if (env.intent === "DatasetDiscovery" || env.routerHints.discovery === "aggressive") {
    searchQueries.push(
      `${baseQ} filetype:pdf OR filetype:txt intitle:${safeQueryKey(baseQ)}`
    );
    searchQueries.push(
      `"${safeQueryKey(baseQ)}" inurl:spec OR inurl:doc OR inurl:manual`
    );
  }

  // 5) Security / malware intel when dealing with code or binaries [web:33][web:35][web:37][web:40]
  if (
    env.intent === "ThreatScan" ||
    env.intent === "CodeSecurityScan" ||
    env.domain === "security"
  ) {
    searchQueries.push(`${baseQ} malware indicators IoC`);
    searchQueries.push(`${baseQ} CVE mapping RAG malware detection`);
  }

  // choose retrieval mode
  /** @type {RetrievalModeId} */
  let retrievalMode = "VectorStore";
  if (env.routerHints?.preferHashRag === "true") {
    retrievalMode = "HashRAG";
  } else if (env.routerHints?.preferRaw === "true") {
    retrievalMode = "RawSearch";
  }

  /** @type {MalwareFilterConfig} */
  const malwareConfig = {
    blockExecutableLinks: true,
    blockSuspiciousTlds: true,
    blockObfuscatedText: true,
    requireHttps: true,
    stripInvisibleContent: true,   // format-breaker/OCR-like idea [web:39]
    useFormatBreaker: true,
    blockedTlds: [".onion", ".ru", ".zip", ".mov"],
  };

  const queryPorts = searchQueries.map((q, idx) => ({
    id: `qport-${idx}`,
    query: q,
    retrievalMode,
    expectedDomain: env.domain,
    intent: env.intent,
    malwareConfig,
  }));

  /** @type {ResearchActionPlan} */
  const plan = {
    envelope: env,
    retrievalMode,
    searchQueries,
    queryPorts,
    scheduleDiversityPortfolio: true,
    scheduleDeDuplication: true,
    scheduleQuizMath: true,
    scheduleMalwareGuards: true,
  };

  return plan;
}

function safeQueryKey(text) {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim()
    .split(/\s+/)[0]
    .slice(0, 24);
}

/* ===========================
   5. Malware-Safe Ingestion Filters
   =========================== */

/**
 * Basic URL security classifier for RAG ingestion [web:39][web:42].
 *
 * @param {string} url
 * @param {MalwareFilterConfig} cfg
 * @returns {{allowed: boolean, reason: string}}
 */
function classifyUrlSecurity(url, cfg) {
  try {
    const u = new URL(url);
    const host = u.hostname.toLowerCase();
    const proto = u.protocol.toLowerCase();

    if (cfg.requireHttps && proto !== "https:") {
      return { allowed: false, reason: "Non-HTTPS blocked" };
    }

    if (
      cfg.blockSuspiciousTlds &&
      cfg.blockedTlds.some((tld) => host.endsWith(tld))
    ) {
      return { allowed: false, reason: "Suspicious TLD blocked" };
    }

    if (
      cfg.blockExecutableLinks &&
      /(\.exe|\.msi|\.bat|\.cmd|\.ps1|\.jar|\.apk|\.scr|\.dll)$/i.test(u.pathname)
    ) {
      return { allowed: false, reason: "Executable-like link blocked" };
    }

    return { allowed: true, reason: "OK" };
  } catch (_e) {
    return { allowed: false, reason: "Invalid URL" };
  }
}

/**
 * Remove invisible/encoded content from HTML-like text [web:39].
 *
 * @param {string} html
 * @param {MalwareFilterConfig} cfg
 * @returns {string}
 */
function stripInvisibleThreats(html, cfg) {
  if (!cfg.stripInvisibleContent) return html;

  let out = String(html);
  // Remove white-on-white or display:none spans
  out = out.replace(
    /<span[^>]*style=["'][^"']*(?:color:\s*#fff|color:\s*white|display:\s*none)[^"']*["'][^>]*>.*?<\/span>/gi,
    " "
  );
  out = out.replace(
    /<font[^>]*color=["']#fff["'][^>]*>.*?<\/font>/gi,
    " "
  );
  // Drop base64-like long runs of A–Z+/= as encoded data indicators [web:39]
  out = out.replace(/[A-Za-z0-9+/=]{80,}/g, " [ENCODED_BLOCK_REMOVED] ");
  return out;
}

/**
 * Apply malware filters to a SearchDocument.
 *
 * @param {SearchDocument} doc
 * @param {MalwareFilterConfig} cfg
 * @returns {{doc: SearchDocument|null, flags: Object}}
 */
function applyMalwareFiltersToDoc(doc, cfg) {
  const uCheck = classifyUrlSecurity(doc.url, cfg);
  const flags = {
    urlAllowed: uCheck.allowed,
    urlReason: uCheck.reason,
    encodedBlocksRemoved: false,
  };

  if (!uCheck.allowed) {
    return { doc: null, flags };
  }

  if (cfg.stripInvisibleContent || cfg.blockObfuscatedText) {
    const cleaned = stripInvisibleThreats(doc.fullText || "", cfg);
    if (cleaned !== doc.fullText) {
      flags.encodedBlocksRemoved = true;
      doc = Object.assign({}, doc, { fullText: cleaned });
    }
  }

  return { doc, flags };
}

/* ===========================
   6. Diversity & De-dup
   =========================== */

/**
 * Build a diversity-aware portfolio from several query ports.
 * The actual search execution is platform-specific; here we just
 * maintain metadata and hash indexes.
 *
 * @param {SearchDocument[][]} perQueryDocs
 * @returns {SearchBatchResult}
 */
function buildDiversePortfolio(perQueryDocs) {
  /** @type {SearchDocument[]} */
  const merged = [];
  const hashIndex = {};
  const seenSimHashes = new Set();

  // 1) Flatten, de-duplicate via SimHash content hashing [web:38][web:41].
  for (let qi = 0; qi < perQueryDocs.length; qi++) {
    const docs = perQueryDocs[qi] || [];
    for (let di = 0; di < docs.length; di++) {
      const d = docs[di];
      const textBasis = `${d.title}\n${d.fullText || d.snippet || ""}`;
      const key = vcHash32(textBasis);
      const sim = vcSimHash64(textBasis);

      if (hashIndex[key]) {
        hashIndex[key].push(merged.length);
      } else {
        hashIndex[key] = [merged.length];
      }

      if (seenSimHashes.has(sim)) {
        continue; // near-duplicate
      }
      seenSimHashes.add(sim);

      merged.push(d);
    }
  }

  /** @type {SearchBatchResult} */
  const out = {
    documents: merged,
    hashIndex,
  };
  return out;
}

/* ===========================
   7. quiz_math Metrics
   =========================== */

/**
 * Compute topic distribution over naive clusters by domain in URL.
 *
 * @param {SearchDocument[]} docs
 * @returns {Object} host->count
 */
function inferHostDistribution(docs) {
  const map = {};
  for (let i = 0; i < docs.length; i++) {
    try {
      const u = new URL(docs[i].url);
      const host = u.hostname.toLowerCase();
      map[host] = (map[host] || 0) + 1;
    } catch (_e) {
      map["unknown"] = (map["unknown"] || 0) + 1;
    }
  }
  return map;
}

/**
 * Entropy of Evidence H = -sum p_i log2 p_i over host buckets.
 *
 * @param {SearchDocument[]} docs
 * @returns {number}
 */
function computeEntropyOfEvidence(docs) {
  if (!docs.length) return 0.0;
  const dist = inferHostDistribution(docs);
  const total = Object.values(dist).reduce((a, b) => a + b, 0);
  let H = 0.0;
  for (const k of Object.keys(dist)) {
    const p = dist[k] / total;
    if (p <= 0) continue;
    H += -p * Math.log2(p);
  }
  return parseFloat(H.toFixed(3));
}

/**
 * Cross-source agreement: crude indicator by counting how many docs
 * mention the same top-n-grams in title/snippet.
 *
 * @param {SearchDocument[]} docs
 * @returns {number} 0–1
 */
function computeCrossSourceAgreement(docs) {
  if (docs.length < 2) return 0.0;
  const grams = {};
  for (let i = 0; i < docs.length; i++) {
    const text = `${docs[i].title} ${docs[i].snippet}`.toLowerCase();
    const tokens = text.replace(/[^a-z0-9\s]/g, " ").split(/\s+/).filter(Boolean);
    for (let j = 0; j < tokens.length - 1; j++) {
      const g = `${tokens[j]} ${tokens[j + 1]}`;
      if (g.length < 5) continue;
      grams[g] = (grams[g] || 0) + 1;
    }
  }
  let supporting = 0;
  let total = 0;
  for (const k of Object.keys(grams)) {
    total += 1;
    if (grams[k] >= 3) supporting += 1;
  }
  if (!total) return 0.0;
  const score = supporting / total;
  return parseFloat(score.toFixed(3));
}

/**
 * Compression ratio: unique “facts” / total tokens.
 * Facts approximated as distinct sentences.
 *
 * @param {SearchDocument[]} docs
 * @returns {number}
 */
function computeCompressionRatio(docs) {
  let sentences = new Set();
  let totalTokens = 0;

  for (let i = 0; i < docs.length; i++) {
    const t = (docs[i].fullText || docs[i].snippet || "").split(/[.!?]+/);
    for (let j = 0; j < t.length; j++) {
      const s = t[j].trim();
      if (!s) continue;
      sentences.add(s);
      totalTokens += s.split(/\s+/).length;
    }
  }

  if (!totalTokens) return 0.0;
  const ratio = sentences.size / totalTokens;
  return parseFloat(ratio.toFixed(4));
}

/**
 * Coverage score: fraction of docs that include core query terms.
 *
 * @param {SearchDocument[]} docs
 * @param {string} query
 * @returns {number}
 */
function computeCoverageScore(docs, query) {
  const qTokens = query
    .toLowerCase()
    .replace(/[^a-z0-9\s]/g, " ")
    .split(/\s+/)
    .filter((t) => t.length > 3);
  if (!qTokens.length) return 0.0;
  let hit = 0;
  for (let i = 0; i < docs.length; i++) {
    const text = `${docs[i].title} ${docs[i].snippet} ${docs[i].fullText || ""}`
      .toLowerCase()
      .replace(/[^a-z0-9\s]/g, " ");
    const tokens = new Set(text.split(/\s+/));
    const matched = qTokens.some((t) => tokens.has(t));
    if (matched) hit += 1;
  }
  const score = hit / docs.length;
  return parseFloat(score.toFixed(3));
}

/**
 * Run quiz_math metrics for a retrieval batch [web:42][web:39].
 *
 * @param {SearchBatchResult} batch
 * @param {string} query
 * @returns {QuizMathScores}
 */
function runQuizMath(batch, query) {
  const docs = batch.documents;
  const entropy = computeEntropyOfEvidence(docs);
  const agreement = computeCrossSourceAgreement(docs);
  const compression = computeCompressionRatio(docs);
  const coverage = computeCoverageScore(docs, query);

  /** @type {QuizMathScores} */
  const scores = {
    entropyOfEvidence: entropy,
    crossSourceAgreement: agreement,
    compressionRatio: compression,
    coverageScore: coverage,
  };
  return scores;
}

/* ===========================
   8. NeuralRope Logging & RoH Management
   =========================== */

/**
 * Start a NeuralRope for this conversation.
 *
 * @param {PromptEnvelope} env
 * @returns {NeuralRope}
 */
function startNeuralRope(env) {
  /** @type {NeuralRope} */
  const rope = {
    ropeId: vcGenerateId("rope"),
    profile: env.neurorightsProfile,
    cumulativeKsr: JSON.parse(JSON.stringify(env.ksr)),
    steps: [],
  };
  return rope;
}

/**
 * Update rope with a new step + adjust cumulative RoH.
 *
 * @param {NeuralRope} rope
 * @param {RopeStepTelemetry} step
 * @returns {NeuralRope}
 */
function pushRopeStep(rope, step) {
  rope.steps.push(step);
  const avgRoh =
    (rope.cumulativeKsr.rohFloat * (rope.steps.length - 1) + step.ksr.rohFloat) /
    rope.steps.length;
  rope.cumulativeKsr.rohFloat = clampRoh(avgRoh);
  rope.cumulativeKsr.riskOfHarm = rohToBand(rope.cumulativeKsr.rohFloat);
  return rope;
}

/**
 * Decide whether to insert cool-down step when RoH approaches ceiling.
 *
 * @param {NeuralRope} rope
 * @returns {{needCooldown: boolean, cooldownIntent: PromptIntentId|null}}
 */
function checkRopeCooldown(rope) {
  if (rope.cumulativeKsr.rohFloat >= 0.25) {
    return { needCooldown: true, cooldownIntent: "RetrievePolicy" };
  }
  return { needCooldown: false, cooldownIntent: null };
}

/* ===========================
   9. High-Level Orchestrator
   =========================== */

/**
 * Orchestrate a full research turn (minus actual HTTP search, which should
 * be provided as a callback). This function shows how to wire:
 * - PromptEnvelope
 * - ResearchActionPlan
 * - Diversity portfolio
 * - Malware guards
 * - quiz_math metrics
 * - NeuralRope logging
 *
 * Integrate in AI-chat backends by passing a searchFn that takes a query
 * and returns SearchDocument[] from your chosen search/RAG layer [web:38][web:41][web:42].
 *
 * @param {Object} opts
 * @param {Function} opts.searchFn async (query: string) => Promise<SearchDocument[]>
 * @param {Object} opts.identity
 * @param {string} opts.userText
 * @param {PromptIntentId} opts.intent
 * @param {PromptDomainId} opts.domain
 * @returns {Promise<{rope: NeuralRope, portfolio: SearchBatchResult, quizMath: QuizMathScores}>}
 */
async function runGovernedResearchTurn(opts) {
  const env = buildPromptEnvelope({
    intent: opts.intent,
    domain: opts.domain,
    userText: opts.userText,
    identity: opts.identity,
    neurorightsProfile: "citizen.v1",
    ksr: { knowledge: "low", socialImpact: "low", rohFloat: 0.1 },
    routerHints: { preferHashRag: "true" },
  });

  const rope = startNeuralRope(env);
  const plan = planResearchActions(env);

  /** @type {SearchDocument[][]} */
  const perQueryDocs = [];

  for (let i = 0; i < plan.queryPorts.length; i++) {
    const port = plan.queryPorts[i];
    const rawDocs = await opts.searchFn(port.query);

    const filteredDocs = [];
    const secFlags = [];

    for (let j = 0; j < rawDocs.length; j++) {
      const { doc, flags } = applyMalwareFiltersToDoc(rawDocs[j], port.malwareConfig);
      if (doc) {
        filteredDocs.push(doc);
      }
      secFlags.push(flags);
    }

    perQueryDocs.push(filteredDocs);

    // log step
    /** @type {RopeStepTelemetry} */
    const step = {
      traceId: vcGenerateId("trace"),
      stepId: vcGenerateId("step"),
      intent: env.intent,
      domain: env.domain,
      toolId: port.retrievalMode,
      query: port.query,
      urls: filteredDocs.map((d) => d.url),
      ksr: {
        knowledge: "medium",
        socialImpact: env.ksr.socialImpact,
        riskOfHarm: env.ksr.riskOfHarm,
        rohFloat: env.ksr.rohFloat,
      },
      quizMath: {},
      securityFlags: { perDoc: secFlags },
    };

    pushRopeStep(rope, step);

    const cooldown = checkRopeCooldown(rope);
    if (cooldown.needCooldown) {
      break;
    }
  }

  const portfolio = buildDiversePortfolio(perQueryDocs);
  const quizMath = runQuizMath(portfolio, env.userText);

  // attach quizMath to last step for traceability
  if (rope.steps.length) {
    rope.steps[rope.steps.length - 1].quizMath = quizMath;
  }

  return { rope, portfolio, quizMath };
}

/* ===========================
   10. Debug Console Output Helper
   =========================== */

/**
 * Render a console-style debug trace of the research turn.
 *
 * @param {NeuralRope} rope
 * @param {SearchBatchResult} portfolio
 * @param {QuizMathScores} quizMath
 */
function printDebugTrace(rope, portfolio, quizMath) {
  console.log("=== Governed Research Trace ===");
  console.log("RopeId:", rope.ropeId);
  console.log("Profile:", rope.profile);
  console.log("Cumulative KSR:", rope.cumulativeKsr);
  console.log("Steps:", rope.steps.length);

  for (let i = 0; i < rope.steps.length; i++) {
    const s = rope.steps[i];
    console.log(`--- Step ${i + 1} ---`);
    console.log("StepId:", s.stepId, "Tool:", s.toolId);
    console.log("Query:", s.query);
    console.log("URLs:", s.urls.slice(0, 5));
    console.log("RoH:", s.ksr.rohFloat);
    console.log("SecurityFlagsSample:", s.securityFlags.perDoc?.[0]);
  }

  console.log("PortfolioDocs:", portfolio.documents.length);
  console.log("QuizMath:", quizMath);
}

/* ===========================
   11. Exports
   =========================== */

const NeurorightsRagKernel = {
  buildPromptEnvelope,
  validatePromptEnvelope,
  planResearchActions,
  classifyUrlSecurity,
  stripInvisibleThreats,
  applyMalwareFiltersToDoc,
  buildDiversePortfolio,
  runQuizMath,
  startNeuralRope,
  pushRopeStep,
  checkRopeCooldown,
  runGovernedResearchTurn,
  printDebugTrace,
};

if (typeof module !== "undefined") {
  module.exports = NeurorightsRagKernel;
}
if (typeof window !== "undefined") {
  window.NeurorightsRagKernel = NeurorightsRagKernel;
}
