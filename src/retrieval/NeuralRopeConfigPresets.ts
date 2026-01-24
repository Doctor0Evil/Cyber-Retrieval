// Filename: src/retrieval/NeuralRopeConfigPresets.ts
// Presets and helper constructors to make this retrieval spine easy to drop into
// any VL/IG or research agent environment (Visual-Code style).

import {
  InMemoryNeuralRopeLogger,
  NeuralRopePacer,
  NeuralRopeRetrievalKernel,
  RopeSessionConfig,
  RetrievalKernelConfig,
  PromptIntent,
  KSRTriple
} from "./NeuralRopeRetrievalKernel";

export function mkDefaultKSR(): KSRTriple {
  return { knowledgeFactor: 0.8, socialImpact: 0.75, riskOfHarm: 0.2 };
}

export function mkDefaultRetrievalKernel(): {
  kernel: NeuralRopeRetrievalKernel;
  logger: InMemoryNeuralRopeLogger;
} {
  const logger = new InMemoryNeuralRopeLogger();
  const cfg: RetrievalKernelConfig = {
    rohCeiling: 0.3,
    minDiversityScore: 0.5
  };
  const kernel = new NeuralRopeRetrievalKernel(cfg, logger);
  return { kernel, logger };
}

export function mkDefaultRopePacer(): NeuralRopePacer {
  const cfg: RopeSessionConfig = {
    rohCeiling: 0.3,
    maxHighRiskSegments: 3,
    cooldownSegmentTemplate:
      "Summarize current findings, check against DCM/HCI/neurorights policy, and state RoH/KSR before any new high-risk retrieval."
  };
  return new NeuralRopePacer(cfg);
}

export function mkRetrieveKnowledgeIntent(
  id: string,
  query: string,
  domain: "GENERAL_KNOWLEDGE" | "DCM_HCI_DESIGN" | "VL_IG_TRAINING"
): PromptIntent {
  return {
    id,
    createdAt: new Date().toISOString(),
    kind: "RetrieveKnowledge",
    query,
    domain,
    xrZone: "XR-ZONE-REMOTE-READONLY",
    ksrestimate: mkDefaultKSR(),
    rohCeiling: 0.3,
    maxDepth: 2,
    allowedCodeActions: ["SCHEMA_PROPOSAL"]
  };
}
