use bioscale_corridor_macros::bioscaleupgrade;
use corridor_guards::{IngestionEnvGateInput, ingestion_env_gate};

pub fn apply_retrieval_only_upgrade(
    env_input: IngestionEnvGateInput,
) -> Result<(String, String), String> {
    // Example retrieval computation: returns a new ALN shard string,
    // but does not commit it. Commit is done by an external, audited tool.
    bioscaleupgrade!(
        env_input,
        String::from("aln shard: retrieval-only upgrade descriptor v1")
    )
}
