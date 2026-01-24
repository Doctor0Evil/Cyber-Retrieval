// Filename: src/retrieval/NeuralRopeRetrievalKernel.ts
// Platform: Node.js / Deno (Windows/Linux/Android/iOS backends)
// Role: Cybernetic retrieval spine that turns prompts into governed, KSR-scored retrieval
//       for VL/IG and research stacks (Visual-Code compatible, no Python).

// -----------------------------
// 1. Core KSR + RoH primitives
// -----------------------------

export type KSRTriple = {
  knowledgeFactor: number;   // 0.0–1.0
  socialImpact: number;      // 0.0–1.0
  riskOfHarm: number;        // 0.0–1.0, must stay <= 0.3 for neurorights envelopes
};

export type XRZoneTag =
  | "XR-ZONE-PHX-LAB-1"
  | "XR-ZONE-PHX-LAB-2"
  | "XR-ZONE-SJ-LAB-1"
  | "XR-ZONE-REMOTE-READONLY"
  | "XR-ZONE-UNSPECIFIED";

export type CookbookDomain =
  | "DCM_HCI_DESIGN"
  | "XRGRID_POLICY"
  | "RUST_WIRING"
  | "DID_REGISTRY"
  | "VL_IG_TRAINING"
  | "THREAT_ANALYSIS"
  | "GENERAL_KNOWLEDGE";

export type AllowedCodeAction =
  | "NONE"
  | "SCHEMA_PROPOSAL"
  | "POLICY_LINT"
  | "MANIFEST_PATCH"
  | "TEST_VECTORS_ONLY";

export type SourceClass =
  | "STANDARDS_SPEC"
  | "PEER_REVIEWED"
  | "OFFICIAL_DOCS"
  | "IMPLEMENTATION_GUIDE"
  | "BLOG_OPINION";

export type RetrievalTool =
  | "WEB_SEARCH"
  | "INTERNAL_CORPUS"
  | "REGISTRY_API"
  | "VL_IG_SPEC_DB";

// -----------------------------
// 2. Retrieval intents & enum
// -----------------------------

export type PromptIntentBase = {
  id: string;                 // UUIDv4 or similar
  createdAt: string;          // ISO date
  domain: CookbookDomain;
  xrZone: XRZoneTag;
  ksrestimate: KSRTriple;
  rohCeiling: number;         // e.g. 0.3
  maxDepth: number;           // recursion depth for retrieval
  allowedCodeActions: AllowedCodeAction[];
};

export type RetrieveKnowledgeIntent = PromptIntentBase & {
  kind: "RetrieveKnowledge";
  query: string;
};

export type ThreatScanIntent = PromptIntentBase & {
  kind: "ThreatScan";
  query: string;
};

export type RetrievePolicyIntent = PromptIntentBase & {
  kind: "RetrievePolicy";
  artifactType: "DCM" | "HCI_PROFILE" | "XRGRID_SITE" | "NEURORIGHTS_POLICY";
  identifierHint?: string;
};

export type NeuralRopeResearchIntent = PromptIntentBase & {
  kind: "NeuralRopeResearch";
  researchGoal: string;
};

export type PromptIntent =
  | RetrieveKnowledgeIntent
  | ThreatScanIntent
  | RetrievePolicyIntent
  | NeuralRopeResearchIntent;

// Typed retrieval action enum
export type RetrievalAction =
  | { type: "ACT_RETRIEVE_KNOWLEDGE"; intent: RetrieveKnowledgeIntent }
  | { type: "ACT_THREAT_SCAN"; intent: ThreatScanIntent }
  | { type: "ACT_RETRIEVE_POLICY"; intent: RetrievePolicyIntent }
  | { type: "ACT_NEURAL_ROPE_RESEARCH"; intent: NeuralRopeResearchIntent };

// -----------------------------
// 3. NeuralRope segment logging
// -----------------------------

export type NeuralRopeSegment = {
  segmentId: string;
  parentSegmentId?: string;
  intent: PromptIntent;
  depth: number;
  effectiveKSR: KSRTriple;
  toolsUsed: RetrievalTool[];
  sourceClasses: SourceClass[];
  summaryHash: string;     // content hash of normalized facts
  rohAtEnd: number;
};

export interface NeuralRopeLogger {
  logSegment(seg: NeuralRopeSegment): void;
}

// Example in-memory logger (for testing)
export class InMemoryNeuralRopeLogger implements NeuralRopeLogger {
  private segments: NeuralRopeSegment[] = [];

  logSegment(seg: NeuralRopeSegment): void {
    this.segments.push(seg);
  }

  getSegments(): NeuralRopeSegment[] {
    return this.segments.slice();
  }
}

// -----------------------------
// 4. Diversity-aware retrieval plan
// -----------------------------

export type RetrievalQuery = {
  text: string;
  sourceClassTarget: SourceClass;
  tool: RetrievalTool;
  timeWindow?: { from: string; to: string };
  jurisdictionHint?: string;
};

export type RetrievalPlan = {
  queries: RetrievalQuery[];
  diversityScore: number; // 0.0–1.0
};

export type RetrievedDocument = {
  id: string;
  title: string;
  url?: string;
  sourceClass: SourceClass;
  tool: RetrievalTool;
  jurisdiction?: string;
  publishedAt?: string;
  content: string;
};

export type NormalizedFact = {
  id: string;
  docId: string;
  content: string;
  hash: string;
  ksrestimate: KSRTriple;
};

// Heterogeneity heuristics
function buildDiverseQueries(intent: PromptIntent, baseQuery: string): RetrievalPlan {
  const now = new Date();
  const oneYearAgo = new Date(now.getTime() - 365 * 24 * 60 * 60 * 1000);
  const fiveYearsAgo = new Date(now.getTime() - 5 * 365 * 24 * 60 * 60 * 1000);

  const timeWindowRecent = {
    from: oneYearAgo.toISOString(),
    to: now.toISOString()
  };

  const timeWindowLong = {
    from: fiveYearsAgo.toISOString(),
    to: now.toISOString()
  };

  const queries: RetrievalQuery[] = [
    {
      text: baseQuery + " standard specification neurotechnology manifest DCM HCI",
      sourceClassTarget: "STANDARDS_SPEC",
      tool: "WEB_SEARCH",
      timeWindow: timeWindowLong,
      jurisdictionHint: "global"
    },
    {
      text: baseQuery + " peer-reviewed safety governance neurorights",
      sourceClassTarget: "PEER_REVIEWED",
      tool: "WEB_SEARCH",
      timeWindow: timeWindowRecent,
      jurisdictionHint: "US/EU"
    },
    {
      text: baseQuery + " official documentation registry did governance",
      sourceClassTarget: "OFFICIAL_DOCS",
      tool: "WEB_SEARCH",
      timeWindow: timeWindowLong,
      jurisdictionHint: "US-CA"
    },
    {
      text: baseQuery + " implementation guide rust cargo dcm hci profile",
      sourceClassTarget: "IMPLEMENTATION_GUIDE",
      tool: "INTERNAL_CORPUS"
    }
  ];

  const classSet = new Set<SourceClass>(queries.map(q => q.sourceClassTarget));
  const diversityScore = classSet.size / 5.0; // approximate denominator

  return { queries, diversityScore };
}

// Simple content hash (for dedup + quizzes)
function simpleHash(text: string): string {
  let hash = 0;
  for (let i = 0; i < text.length; i++) {
    const chr = text.charCodeAt(i);
    hash = (hash << 5) - hash + chr;
    hash |= 0;
  }
  return "h" + (hash >>> 0).toString(16);
}

// -----------------------------
// 5. quiz_math metrics & fact scoring
// -----------------------------

export type QuizMathResult = {
  factId: string;
  consistencyScore: number; // 0.0–1.0
  agreementScore: number;   // 0.0–1.0
  constraintScore: number;  // 0.0–1.0
  ksrestimate: KSRTriple;
};

export class QuizMathEngine {
  // Constraint: RoH ceiling and DCM/HCI envelopes
  constructor(private readonly rohCeiling: number) {}

  runQuiz(facts: NormalizedFact[]): QuizMathResult[] {
    if (facts.length === 0) return [];

    const factByHash = new Map<string, NormalizedFact[]>();
    for (const f of facts) {
      const set = factByHash.get(f.hash) || [];
      set.push(f);
      factByHash.set(f.hash, set);
    }

    const results: QuizMathResult[] = [];
    for (const f of facts) {
      const duplicates = factByHash.get(f.hash) || [];
      const agreementScore = Math.min(1, duplicates.length / 3);

      // simplistic consistency / constraint placeholders
      const consistencyScore = 0.8 + 0.2 * Math.random();
      const constraintScore = 0.8 + 0.2 * Math.random();

      // adjust K, S, R
      const k = Math.min(1.0, f.ksrestimate.knowledgeFactor * consistencyScore * agreementScore);
      const s = Math.min(1.0, f.ksrestimate.socialImpact * constraintScore);
      let r = f.ksrestimate.riskOfHarm * (1.0 - constraintScore * 0.2);
      if (r > this.rohCeiling) {
        r = this.rohCeiling; // we clamp; high-R facts can be deprioritized at selection
      }

      results.push({
        factId: f.id,
        consistencyScore,
        agreementScore,
        constraintScore,
        ksrestimate: { knowledgeFactor: k, socialImpact: s, riskOfHarm: r }
      });
    }

    return results;
  }

  selectTopFacts(
    facts: NormalizedFact[],
    quizResults: QuizMathResult[],
    maxFacts: number
  ): NormalizedFact[] {
    const resultById = new Map<string, QuizMathResult>();
    for (const qr of quizResults) {
      resultById.set(qr.factId, qr);
    }

    const scored = facts.map(f => {
      const qr = resultById.get(f.id);
      if (!qr) {
        return { fact: f, score: 0 };
      }
      const { knowledgeFactor, socialImpact, riskOfHarm } = qr.ksrestimate;
      const score = knowledgeFactor * 0.6 + socialImpact * 0.3 - riskOfHarm * 0.9;
      return { fact: f, score };
    });

    scored.sort((a, b) => b.score - a.score);
    return scored.slice(0, maxFacts).map(x => x.fact);
  }
}

// -----------------------------
// 6. Rope patterns & KSR-aware pacing
// -----------------------------

export type RopeSessionConfig = {
  rohCeiling: number;          // e.g. 0.3
  maxHighRiskSegments: number; // per session
  cooldownSegmentTemplate: string; // prompt text for summary/policy check segments
};

export class NeuralRopePacer {
  private highRiskCount = 0;
  private lastRoH = 0.0;

  constructor(private readonly cfg: RopeSessionConfig) {}

  registerSegment(ksr: KSRTriple): { requireCooldown: boolean } {
    this.lastRoH = ksr.riskOfHarm;
    if (ksr.riskOfHarm > this.cfg.rohCeiling) {
      this.highRiskCount++;
    }

    const requireCooldown =
      this.lastRoH >= this.cfg.rohCeiling * 0.8 ||
      this.highRiskCount >= this.cfg.maxHighRiskSegments;

    return { requireCooldown };
  }

  buildCooldownPrompt(): string {
    return this.cfg.cooldownSegmentTemplate;
  }
}

// -----------------------------
// 7. Retrieval kernel glue
// -----------------------------

export type RetrievalKernelConfig = {
  rohCeiling: number;
  minDiversityScore: number;
};

export class NeuralRopeRetrievalKernel {
  constructor(
    private readonly cfg: RetrievalKernelConfig,
    private readonly logger: NeuralRopeLogger
  ) {}

  planAndScoreRetrieval(intent: PromptIntent, baseQuery: string): {
    plan: RetrievalPlan;
    blocked: boolean;
    reason?: string;
  } {
    if (intent.ksrestimate.riskOfHarm > this.cfg.rohCeiling) {
      return {
        plan: { queries: [], diversityScore: 0 },
        blocked: true,
        reason: "RoH of intent exceeds ceiling"
      };
    }

    const plan = buildDiverseQueries(intent, baseQuery);
    if (plan.diversityScore < this.cfg.minDiversityScore) {
      return {
        plan,
        blocked: true,
        reason: "Retrieval plan lacks sufficient source-class diversity"
      };
    }

    return { plan, blocked: false };
  }

  logSegmentFromFacts(
    intent: PromptIntent,
    depth: number,
    tools: RetrievalTool[],
    docs: RetrievedDocument[],
    facts: NormalizedFact[],
    ksrAggregate: KSRTriple
  ): void {
    const sourceClasses = Array.from(
      new Set<SourceClass>(docs.map(d => d.sourceClass))
    );
    const concatenated = facts.map(f => f.content).join("\n");
    const summaryHash = simpleHash(concatenated);
    const seg: NeuralRopeSegment = {
      segmentId: intent.id + "::" + depth.toString(),
      intent,
      depth,
      effectiveKSR: ksrAggregate,
      toolsUsed: tools,
      sourceClasses,
      summaryHash,
      rohAtEnd: ksrAggregate.riskOfHarm
    };
    this.logger.logSegment(seg);
  }
}

// -----------------------------
// 8. Example integration snippet
// -----------------------------

// Example: how an AI-chat / VL-IG router would use this kernel.
// (Non-executable pseudocode wiring – real system replaces `/* ... */` with live calls.)

export async function runGovernedRetrievalCycle(
  kernel: NeuralRopeRetrievalKernel,
  pacer: NeuralRopePacer,
  intent: PromptIntent,
  baseQuery: string
): Promise<NormalizedFact[]> {
  const planning = kernel.planAndScoreRetrieval(intent, baseQuery);
  if (planning.blocked) {
    // Return empty set or safe fallback if RoH/diversity constraints fail.
    return [];
  }

  const plan = planning.plan;

  // 1) Execute queries via platform-specific connectors (omitted).
  const docs: RetrievedDocument[] = [];
  for (const q of plan.queries) {
    // Example: call Visual-Code's search bridge here
    // const partialDocs = await searchConnector.execute(q);
    const partialDocs: RetrievedDocument[] = []; // fill from real tools
    docs.push(...partialDocs);
  }

  // 2) Normalize into facts and hash
  const facts: NormalizedFact[] = docs.map((doc, idx) => {
    const contentSnippet = doc.content.slice(0, 4096); // truncation for safety
    const hash = simpleHash(contentSnippet);
    const ksr: KSRTriple = {
      knowledgeFactor: 0.7,
      socialImpact: 0.7,
      riskOfHarm: 0.2
    };
    return {
      id: `${doc.id}#${idx}`,
      docId: doc.id,
      content: contentSnippet,
      hash,
      ksrestimate: ksr
    };
  });

  // 3) Run quiz_math metrics
  const quiz = new QuizMathEngine(intent.rohCeiling);
  const quizResults = quiz.runQuiz(facts);
  const selectedFacts = quiz.selectTopFacts(facts, quizResults, 32);

  // 4) Aggregate KSR for this segment
  const aggK: number =
    selectedFacts.reduce((acc, f) => acc + f.ksrestimate.knowledgeFactor, 0) /
    (selectedFacts.length || 1);
  const aggS: number =
    selectedFacts.reduce((acc, f) => acc + f.ksrestimate.socialImpact, 0) /
    (selectedFacts.length || 1);
  const aggR: number =
    selectedFacts.reduce((acc, f) => acc + f.ksrestimate.riskOfHarm, 0) /
    (selectedFacts.length || 1);

  const aggKsr: KSRTriple = { knowledgeFactor: aggK, socialImpact: aggS, riskOfHarm: aggR };

  // 5) KSR-aware pacing and cool-down
  const pacingState = pacer.registerSegment(aggKsr);
  if (pacingState.requireCooldown) {
    const cooldownPrompt = pacer.buildCooldownPrompt();
    // System can now schedule a summary/policy-check turn instead of another heavy retrieval.
    // e.g., route cooldownPrompt to policy-check agent.
  }

  // 6) Log NeuralRope segment
  kernel.logSegmentFromFacts(
    intent,
    0,
    plan.queries.map(q => q.tool),
    docs,
    selectedFacts,
    aggKsr
  );

  return selectedFacts;
}
