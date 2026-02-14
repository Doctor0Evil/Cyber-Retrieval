pub enum DowngradeReason {
    HostRequestUnwantedMutation(DidSignedVoucher),
    SafetyBreach(ComplianceFieldOutput),
    Impossible,
}

pub struct ChangeProposal {
    pub evolution_id: ChainId,
    pub change_type: ChangeType,            // Upgrade | Downgrade | Disable
    pub downgrade_reason: Option<DowngradeReason>,
    pub target_rights_kernel: RightsKernelSnapshot,
    pub target_neurorights: NeurorightsProfile,
    pub governance_context: JurisdictionContext,
    pub pre_state_hash: [u8; 64],
    pub post_state_hash: [u8; 64],
}

pub fn evaluate(
    proposal: &ChangeProposal,
    current_ctx: &HostContinuityContext,
    rights_kernel: &RightsKernelSnapshot,
    cybostate_scalar: CybostateFactor,             // C
    nanoswarm_field: &NanoswarmComplianceFieldV1,
    host_did_lib: &dyn HostDidVerifier,
    now: SystemTime,
) -> Result<SafetyStatus, RejectionReason> {
    // 1. Enforce host-signed downgrade vouchers.
    if let Some(DowngradeReason::HostRequestUnwantedMutation(v)) = &proposal.downgrade_reason {
        host_did_lib.verify(v, &current_ctx.host_did)
            .map_err(RejectionReason::InvalidHostSignature)?;
    }

    // 2. For SafetyBreach, require nanoswarm rollback decision.
    if let Some(DowngradeReason::SafetyBreach(cf_out)) = &proposal.downgrade_reason {
        if !cf_out.rollback_required {
            return Err(RejectionReason::NoBiophysicalRollbackRequired);
        }
    }

    // 3. Rights monotonicity: target must not weaken neurorights.
    rights_kernel
        .check_monotonic_neurorights(&current_ctx.neurorights_profile, &proposal.target_neurorights)
        .map_err(RejectionReason::NeurorightsRegression)?;

    // 4. cybostate-factor veto: C < Cmin => forbid new upgrades, never forced downgrade.
    if proposal.change_type == ChangeType::Upgrade && cybostate_scalar.value < cybostate_scalar.cmin {
        return Err(RejectionReason::CybostateVeto);
    }

    // 5. For downgrades, require:
    //    - Host-request OR safety-breach
    //    - cybostate logic: never treat low C as a reason to downgrade.
    if proposal.change_type == ChangeType::Downgrade {
        match &proposal.downgrade_reason {
            Some(DowngradeReason::HostRequestUnwantedMutation(_)) => { /* ok */ }
            Some(DowngradeReason::SafetyBreach(cf_out)) if cf_out.rollback_required => { /* ok */ }
            _ => return Err(RejectionReason::PolicyOnlyDowngradeForbidden),
        }
    }

    // 6. Jurisdiction check: off_world_jurisdiction cannot override rights/bio limits.
    current_ctx
        .jurisdiction
        .assert_subordinate_to_rights_and_bio(&rights_kernel, nanoswarm_field)
        .map_err(RejectionReason::JurisdictionViolation)?;

    Ok(SafetyStatus::Allowed { evaluated_at: now })
}
