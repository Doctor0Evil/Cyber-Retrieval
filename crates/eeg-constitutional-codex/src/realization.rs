use super::{
    FrozenEegMap, ConstitutionalInvariants, PolicyEngine,
    NeuromorphicState, CandidateAction, SovereignEndpoint,
};
use cyberswarm_neurostack::{BciHostSnapshot, NeuralController};
use bioscale_upgrade_store::{HostBudget, UpgradeDescriptor};
use googolswarm_attest::{MeasuredBinaryHash, ConfigHash, AuditSink};

/// Unified interface: both deviceless and device-trusted realizations must implement this.
pub trait ConstitutionalRuntime {
    /// Given raw biosignals and host state, step controller one tick and optionally emit an endpoint.
    fn step(
        &mut self,
        raw_eeg_window: &[f32],
        host_budget: HostBudget,
        bci_snap: BciHostSnapshot,
        candidate: Option<CandidateAction>,
        descriptor: &UpgradeDescriptor,
    ) -> Option<SovereignEndpoint>;
}

/// Deviceless realization – pure math, no hardware dependencies.
pub struct DevicelessRuntime<M, C, P>
where
    M: FrozenEegMap,
    C: ConstitutionalInvariants + NeuralController,
    P: PolicyEngine,
{
    pub map: M,
    pub controller: C,
    pub policy: P,
    pub last_state: Option<NeuromorphicState>,
    pub last_v: f64,
}

impl<M, C, P> ConstitutionalRuntime for DevicelessRuntime<M, C, P>
where
    M: FrozenEegMap,
    C: ConstitutionalInvariants + NeuralController,
    P: PolicyEngine,
{
    fn step(
        &mut self,
        raw_eeg_window: &[f32],
        host_budget: HostBudget,
        bci_snap: BciHostSnapshot,
        descriptor: &UpgradeDescriptor,
        candidate: Option<CandidateAction>,
    ) -> Option<SovereignEndpoint> {
        let signal = self.map.f_eeg(raw_eeg_window);
        let next_ctrl = self.controller.step(&signal, &bci_snap);

        let prev_state = self.last_state.clone().unwrap_or_else(|| NeuromorphicState {
            signal: signal.clone(),
            controller: next_ctrl.clone(),
            host_budget: host_budget.clone(),
            t: std::time::SystemTime::now(),
        });

        let next_state = NeuromorphicState {
            signal,
            controller: next_ctrl,
            host_budget,
            t: std::time::SystemTime::now(),
        };

        let invariants = self.controller.evaluate_invariants(
            &prev_state,
            &next_state,
            &descriptor.thermo_envelope,
            &descriptor.ml_schedule,
            &bci_snap,
            self.last_v,
        );
        self.last_v = invariants.lyapunov_v;
        self.last_state = Some(next_state.clone());

        if let Some(action) = candidate {
            let state_hash = next_state_hash(&next_state, &bci_snap);
            crate::governance::gate_action_to_endpoint(
                &self.policy,
                invariants,
                &next_state,
                action,
                state_hash,
            )
        } else {
            None
        }
    }
}

/// Device-trusted realization – wraps the same math in an attested binary and immutable audit log.
pub struct AttestedRuntime<R: ConstitutionalRuntime> {
    pub inner: R,
    pub code_hash: MeasuredBinaryHash,
    pub config_hash: ConfigHash,
    pub audit_sink: Box<dyn AuditSink>,
}

impl<R: ConstitutionalRuntime> ConstitutionalRuntime for AttestedRuntime<R> {
    fn step(
        &mut self,
        raw_eeg_window: &[f32],
        host_budget: HostBudget,
        bci_snap: BciHostSnapshot,
        descriptor: &UpgradeDescriptor,
        candidate: Option<CandidateAction>,
    ) -> Option<SovereignEndpoint> {
        let endpoint = self.inner.step(
            raw_eeg_window,
            host_budget,
            bci_snap.clone(),
            descriptor,
            candidate,
        );

        // Log invariant outcomes and χ(t) to immutable audit trail (Googolswarm / EvolutionAuditRecord).
        if let Some(ref ep) = endpoint {
            self.audit_sink.record_endpoint(ep, &self.code_hash, &self.config_hash);
        }

        endpoint
    }
}
