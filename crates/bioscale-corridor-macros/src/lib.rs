#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr};

/// bioscaleupgrade! enforces:
/// - IngestionEnvGate::ingestion_env_gate(input).allowed == true
/// - Retrieval-only semantics (no mutation inside the expression by contract)
///
/// Usage (inside host-local crates):
///     let result = bioscaleupgrade!(env_input, compute_upgrade_plan());
#[proc_macro]
pub fn bioscaleupgrade(input: TokenStream) -> TokenStream {
    let exprs = parse_macro_input!(input as syn::ExprTuple);
    let env_expr: Expr = match &exprs.elems.first() {
        Some(e) => (*e).clone(),
        None => panic!("bioscaleupgrade! requires (env_input, expr) tuple"),
    };
    let value_expr: Expr = match &exprs.elems.iter().nth(1) {
        Some(e) => (*e).clone(),
        None => panic!("bioscaleupgrade! requires (env_input, expr) tuple"),
    };

    let expanded = quote! {{
        use corridor_guards::ingestion_env_gate;

        let _env_input = #env_expr;
        let _decision = ingestion_env_gate(&_env_input);
        if !_decision.allowed {
            Err(format!(
                "bioscaleupgrade blocked: {} (audit_hex={})",
                _decision.reason, _decision.audit_hex
            ))
        } else {
            // Retrieval-only: the called expression MUST respect host-local
            // invariants and is treated as pure for formal verification.
            let _value = #value_expr;
            Ok((_value, _decision.audit_hex))
        }
    }};

    TokenStream::from(expanded)
}
