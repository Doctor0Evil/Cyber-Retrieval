use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(BioscaleUpgrade)]
pub fn derive_bioscale_upgrade(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name  = &input.ident;

    // Static structural check: require ReversalConditions and EvidenceBundle fields.
    let has_reversal = /* inspect fields for type ReversalConditions */;
    let has_evidence = /* inspect fields for type EvidenceBundle */;

    if !has_reversal || !has_evidence {
        return quote! {
            compile_error!(
                "BioscaleUpgrade requires ReversalConditions and EvidenceBundle fields \
                 (rollbackanytime + 10-tag evidence chain)."
            );
        }
        .into();
    }

    quote! {
        impl #name {
            pub fn into_descriptor(self) -> UpgradeDescriptor {
                UpgradeDescriptor::from_upgrade_struct(self)
            }
        }
    }
    .into()
}
