fn verify_audit_record(rec: &EvolutionAuditRecord) -> anyhow::Result<()> {
    HostDidLib::verify_signature(&rec.cryptographic_proof, &rec.pre_state_hash)?;

    LedgerClient::verify_tx_contains_hash(&rec.ledger_anchor_tx, &rec.pre_state_hash)?;

    // Optional: recompute post_state_hash from local snapshot.

    Ok(())
}
