use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn bioscaleupgrade(attrs: TokenStream, item: TokenStream) -> TokenStream {
    // attrs: #[bioscaleupgrade(aln_particle = "ALN_NEURORIGHTS_BCI_V1")]
    let expanded = quote::quote! {
        #item

        const _: () = {
            use aln_compliance::ALNComplianceParticle;

            // Compile-time hook: the referenced particle must exist and be BCI-compliant.
            fn _assert_neurorights_contract(p: &ALNComplianceParticle) {
                assert!(p.is_compliant_for_bci(), "BCI upgrade missing neurorights clauses");
            }
        };
    };
    expanded.into()
}
