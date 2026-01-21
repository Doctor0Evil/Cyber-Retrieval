use std::{env, fs, path::PathBuf};

fn main() {
    // Location of ALN shards (configurable via env var)
    let aln_dir = env::var("ALN_DIR").unwrap_or_else(|_| "aln".to_string());

    let asset_path = PathBuf::from(&aln_dir).join("asset.chat.stake.v1.aln");
    let gov_path = PathBuf::from(&aln_dir).join("governance.chat.website.v1.aln");

    println!("cargo:rerun-if-changed={}", asset_path.display());
    println!("cargo:rerun-if-changed={}", gov_path.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
    let dest_path = out_dir.join("aln_generated.rs");

    let asset_yaml = fs::read_to_string(&asset_path)
        .expect("failed to read asset.chat.stake.v1.aln");
    let gov_yaml = fs::read_to_string(&gov_path)
        .expect("failed to read governance.chat.website.v1.aln");

    let generated = generate_code(&asset_yaml, &gov_yaml)
        .expect("failed to generate ALN-derived Rust code");

    fs::write(&dest_path, generated).expect("failed to write aln_generated.rs");
}

fn generate_code(asset_yaml: &str, gov_yaml: &str) -> Result<String, String> {
    #[derive(serde::Deserialize)]
    struct MinStake {
        stakeholder: String,
        council: String,
        superchair: String,
    }

    #[derive(serde::Deserialize)]
    struct AssetRoot {
        id: String,
        version: String,
        min_stake: MinStake,
        risk: RiskSection,
    }

    #[derive(serde::Deserialize)]
    struct RiskSection {
        risk_of_harm_ceiling: f64,
        default_risk_of_harm: f64,
        default_knowledge_factor: f64,
        default_cybostate_factor: f64,
    }

    #[derive(serde::Deserialize)]
    struct RolesRoot {
        stakeholder: RoleDef,
        council: RoleDef,
        superchair: RoleDef,
    }

    #[derive(serde::Deserialize)]
    struct RoleDef {
        permissions: Vec<String>,
    }

    #[derive(serde::Deserialize)]
    struct Constraints {
        neurorights_required: bool,
        risk_of_harm_ceiling: f64,
        retrieval_only_for_generation: bool,
    }

    #[derive(serde::Deserialize)]
    struct GovernanceRoot {
        id: String,
        version: String,
        roles: RolesRoot,
        constraints: Constraints,
    }

    let asset: AssetRoot = serde_yaml::from_str(asset_yaml)
        .map_err(|e| format!("asset yaml parse error: {e}"))?;
    let gov: GovernanceRoot = serde_yaml::from_str(gov_yaml)
        .map_err(|e| format!("governance yaml parse error: {e}"))?;

    let s_min = &asset.min_stake.stakeholder;
    let c_min = &asset.min_stake.council;
    let sc_min = &asset.min_stake.superchair;

    let risk = &asset.risk;
    let constraints = &gov.constraints;

    let code = format!(
        r#"
/// ALN id for CHAT stake asset shard.
pub const ASSET_CHAT_STAKE_ID: &str = "{asset_id}";
/// ALN id for website governance shard.
pub const GOV_CHAT_WEBSITE_ID: &str = "{gov_id}";

/// Neurorights-governed global risk ceilings derived from ALN.
pub const RISK_OF_HARM_CEILING: f64 = {roh_ceiling};
pub const DEFAULT_RISK_OF_HARM: f64 = {roh_default};
pub const DEFAULT_KNOWLEDGE_FACTOR: f64 = {kf_default};
pub const DEFAULT_CYBOSTATE_FACTOR: f64 = {cs_default};

/// Minimum CHAT stake thresholds for each governance role, in decimal-string form.
pub mod stake_thresholds {{
    /// Stake required to be a stakeholder.
    pub const STAKEHOLDER_MIN: &str = "{stakeholder_min}";
    /// Stake required to be a council member.
    pub const COUNCIL_MIN: &str = "{council_min}";
    /// Stake required to be a superchair.
    pub const SUPERCHAIR_MIN: &str = "{superchair_min}";
}}

/// Governance constraints derived from governance.chat.website.v1.
pub mod governance_constraints {{
    pub const NEURORIGHTS_REQUIRED: bool = {neurorights_required};
    pub const RISK_OF_HARM_CEILING: f64 = {gov_roh_ceiling};
    pub const RETRIEVAL_ONLY_FOR_GENERATION: bool = {retrieval_only};
}}
"#,
        asset_id = asset.id,
        gov_id = gov.id,
        roh_ceiling = risk.risk_of_harm_ceiling,
        roh_default = risk.default_risk_of_harm,
        kf_default = risk.default_knowledge_factor,
        cs_default = risk.default_cybostate_factor,
        stakeholder_min = s_min,
        council_min = c_min,
        superchair_min = sc_min,
        neurorights_required = constraints.neurorights_required,
        gov_roh_ceiling = constraints.risk_of_harm_ceiling,
        retrieval_only = constraints.retrieval_only_for_generation,
    );

    Ok(code)
}
