/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

// Third Party Imports
use harmos::operations::{Host, NoStream, OperationDeclaration, OperationPolicy, Requires, RetryPolicy, Visibility};
use harmos::sidecar::operations::ServingError;
use harmos::sidecar::{Refusal, Supervisor};

// Local Imports
use super::{Caller, SampleOutput, SidecarTemplate, Workflow};
use crate::transport::buffer::lock;
use crate::transport::templated;
use crate::transport::{
    EvidenceBatch, EvidenceReceipt, ExecutionSignal, LeaseAsk, LeaseResolution, OperatorAnswer, OperatorQuestion, RUN_BOUND, RunExecution,
    SIGNAL_BOUND, SidecarExecutionOutcome, SignalReply, TransportFuture,
};

/* === Constants === */

const RUN_POLICY: OperationPolicy = OperationPolicy {
    timeout: RUN_BOUND,
    attempt_timeout: RUN_BOUND,
    cancellation_timeout: std::time::Duration::ZERO,
    retries: RetryPolicy::Never,
};
pub const SIGNAL_POLICY: OperationPolicy = OperationPolicy {
    timeout: SIGNAL_BOUND,
    attempt_timeout: SIGNAL_BOUND,
    cancellation_timeout: std::time::Duration::ZERO,
    retries: RetryPolicy::Never,
};

/* === Macros === */

templated!(Run, RunExecution);

templated!(Signal, ExecutionSignal);

/* === Type Aliases === */

/* === Structs === */

// One template's run operation: the request is the portable `RunExecution`;
// the type parameter only selects the template's operation identity.
pub struct Run<T>(pub RunExecution, PhantomData<fn() -> T>);

// One template's signal operation over the portable `ExecutionSignal`.
pub struct Signal<T>(pub ExecutionSignal, PhantomData<fn() -> T>);

// The resource the generic template pair reads; see `EmbeddedRegistry`.
#[derive(Clone)]
pub struct EmbeddedTemplates(pub Arc<dyn EmbeddedRegistry>);

struct InputAdmission {
    executions: crate::transport::buffer::Executions,
    token: String,
    changed: Arc<tokio::sync::Notify>,
}

/* === Enums === */

#[harmos::catalog]
pub enum Operations<T: SidecarTemplate> {
    Run(Run<T>),
    Signal(Signal<T>),
}

#[harmos::catalog]
pub enum Empty {}

enum Answered {
    Prompt { prompt_id: String, answer: Result<OperatorAnswer, String> },
    Lease { lease_token: String, resolution: Result<LeaseResolution, String> },
}

/* === Traits === */

// The host's embedded template registry behind the generic
// `dokime.template.run`/`dokime.template.signal` pair (`Run<()>`/`Signal<()>`):
// the host inserts `EmbeddedTemplates(Arc<dyn EmbeddedRegistry>)` as the
// registration resource and routes each request to its embedded template.
pub trait EmbeddedRegistry: Send + Sync + 'static {
    fn run(&self, request: RunExecution) -> TransportFuture<'_, Result<SidecarExecutionOutcome, Refusal>>;

    fn signal(&self, signal: ExecutionSignal) -> TransportFuture<'_, Result<SignalReply, Refusal>>;
}

pub trait ExecutionHost: Send + Sync + 'static {
    fn push(&self, batch: EvidenceBatch) -> TransportFuture<'_, Result<EvidenceReceipt, String>>;

    fn ask(&self, question: OperatorQuestion) -> TransportFuture<'_, Result<OperatorAnswer, String>>;

    fn lease(&self, ask: LeaseAsk) -> TransportFuture<'_, Result<LeaseResolution, String>>;
}

// The operation identities one served runner answers under: every
// `SidecarTemplate` is served, and hand-written runners declare their own.
pub trait Served: 'static {
    const RUN: &'static str;
    const SIGNAL: &'static str;
    const SIGNAL_POLICY: OperationPolicy = SIGNAL_POLICY;
}

pub trait Runnable: Send + Sync + 'static {
    fn template_identifier(&self) -> &str;

    fn run(
        &self, request: RunExecution, host: Arc<dyn ExecutionHost>, output: Option<SampleOutput>, caller: Caller,
    ) -> TransportFuture<'_, Result<SidecarExecutionOutcome, Refusal>>;

    // The caller owns the signal deadline. Dropping this future cancels a
    // cooperative delivery; implementations must not block the executor.
    fn signal(&self, signal: ExecutionSignal) -> TransportFuture<'_, Result<SignalReply, Refusal>>;

    fn take_local_samples(&self, token: &str) -> Result<Vec<crate::telemetry::TelemetrySample>, Refusal>;

    fn interrupt(&self);

    fn shutdown(&self) -> TransportFuture<'_, ()>;
}

/* === Implementations === */

impl Drop for InputAdmission {
    fn drop(&mut self) {
        if let Some(state) = lock(&self.executions).get_mut(&self.token)
            && Arc::ptr_eq(&state.changed, &self.changed)
        {
            state.settle_operation();
        }
    }
}

// The host's embedded runner serves the generic template identities.
impl Served for () {
    const RUN: &'static str = "dokime.template.run";
    const SIGNAL: &'static str = "dokime.template.signal";
}

impl<T: SidecarTemplate> Served for T {
    const RUN: &'static str = T::RUN;
    const SIGNAL: &'static str = T::SIGNAL;
}

impl<T: Served> OperationDeclaration for Run<T> {
    type Request = Self;
    type Response = SidecarExecutionOutcome;
    type Input = NoStream;
    type Output = NoStream;
    const ID: &'static str = T::RUN;
    const DESCRIPTION: &'static str = "Runs one case of this template to its outcome.";
    const POLICY: OperationPolicy = RUN_POLICY;
    const VISIBILITY: Visibility = Visibility::Host;
}

impl<T: Served> OperationDeclaration for Signal<T> {
    type Request = Self;
    type Response = SignalReply;
    type Input = NoStream;
    type Output = NoStream;
    const ID: &'static str = T::SIGNAL;
    const DESCRIPTION: &'static str = "Reaches one running case of this template.";
    const POLICY: OperationPolicy = T::SIGNAL_POLICY;
    const VISIBILITY: Visibility = Visibility::Host;
}

// A template's run and signal operations read the template's workflow as a
// declared resource; the run also reads the attempt's own host, so evidence,
// questions, leases and step calls travel under this attempt's call context.
impl<T: SidecarTemplate> Requires<Workflow<T>> for Run<T> {}

impl<T: SidecarTemplate> Requires<Supervisor> for Run<T> {}

impl<T: SidecarTemplate> Requires<Workflow<T>> for Signal<T> {}

impl<T: SidecarTemplate> harmos::Dependent for Run<T> {
    const RESOURCES: &'static [harmos::Resource] = &[harmos::Resource::of::<Workflow<T>>(), harmos::Resource::of::<Supervisor>()];
}

impl<T: SidecarTemplate> harmos::Dependent for Signal<T> {
    const RESOURCES: &'static [harmos::Resource] = &[harmos::Resource::of::<Workflow<T>>()];
}

impl<T: SidecarTemplate> harmos::Operation for Run<T> {
    type Error = Refusal;

    async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<SidecarExecutionOutcome>, Refusal> {
        let workflow = ctx.resource::<Workflow<T>>();
        let host = Supervisor::clone(&ctx.resource::<Supervisor>());
        let outcome = workflow.execute(self.0, Arc::new(host.clone()), Some(SampleOutput::Host(host.clone())), Caller::host(host)).await?;
        Ok(outcome.into())
    }
}

impl<T: SidecarTemplate> harmos::Operation for Signal<T> {
    type Error = Refusal;

    async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<SignalReply>, Refusal> {
        ctx.resource::<Workflow<T>>().signal(self.0).map(Into::into)
    }
}

impl Requires<EmbeddedTemplates> for Run<()> {}

impl Requires<EmbeddedTemplates> for Signal<()> {}

impl harmos::Dependent for Run<()> {
    const RESOURCES: &'static [harmos::Resource] = &[harmos::Resource::of::<EmbeddedTemplates>()];
}

impl harmos::Dependent for Signal<()> {
    const RESOURCES: &'static [harmos::Resource] = &[harmos::Resource::of::<EmbeddedTemplates>()];
}

impl harmos::Operation for Run<()> {
    type Error = Refusal;

    async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<SidecarExecutionOutcome>, Refusal> {
        ctx.resource::<EmbeddedTemplates>().0.run(self.0).await.map(Into::into)
    }
}

impl harmos::Operation for Signal<()> {
    type Error = Refusal;

    async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<SignalReply>, Refusal> {
        ctx.resource::<EmbeddedTemplates>().0.signal(self.0).await.map(Into::into)
    }
}

impl ExecutionHost for Supervisor {
    fn push(&self, batch: EvidenceBatch) -> TransportFuture<'_, Result<EvidenceReceipt, String>> {
        Box::pin(
            async move { self.call::<EvidenceBatch>(batch).await.map(|response| response.into_owned()).map_err(|error| error.to_string()) },
        )
    }

    fn ask(&self, question: OperatorQuestion) -> TransportFuture<'_, Result<OperatorAnswer, String>> {
        Box::pin(async move {
            self.call::<OperatorQuestion>(question).await.map(|response| response.into_owned()).map_err(|error| error.to_string())
        })
    }

    fn lease(&self, ask: LeaseAsk) -> TransportFuture<'_, Result<LeaseResolution, String>> {
        Box::pin(async move { self.call::<LeaseAsk>(ask).await.map(|response| response.into_owned()).map_err(|error| error.to_string()) })
    }
}

impl<T: SidecarTemplate> Workflow<T> {
    pub async fn execute(
        &self, request: RunExecution, host: Arc<dyn ExecutionHost>, output: Option<SampleOutput>, caller: Caller,
    ) -> Result<SidecarExecutionOutcome, Refusal> {
        let token = request.execution_token.clone();
        self.start_on(request, output, caller)?;
        let changed = lock(&self.executions)
            .get(&token)
            .map(|state| Arc::clone(&state.changed))
            .ok_or_else(|| Refusal::of("the started execution is not retained"))?;
        let _admission = InputAdmission {
            executions: Arc::clone(&self.executions),
            token: token.clone(),
            changed: Arc::clone(&changed),
        };
        let mut cursor = 0;
        let mut settling = false;
        let mut asked = BTreeSet::new();
        let mut leased = BTreeSet::new();
        let mut waiting = tokio::task::JoinSet::new();
        loop {
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let batch = self.drain(&token, cursor)?;
            let advanced = batch.cursor > cursor;
            let settle = batch.record_subscriptions_settling && !settling;
            if advanced || settle {
                let next = batch.cursor;
                let outcome = batch.outcome.clone();
                let settled = batch.cleanup_settled;
                let prompts = batch.prompt_requests.clone();
                let leases = batch.lease_requests.clone();
                let receipt =
                    host.push(batch).await.map_err(|reason| Refusal::of(format!("the host did not admit the run's evidence: {reason}")))?;
                cursor = next;
                settling |= settle;
                for admission in &receipt.admitted {
                    self.admit(&token, admission);
                }
                if receipt.settled {
                    self.settle_records(&token);
                }
                for prompt in prompts {
                    ask(&host, &mut waiting, &mut asked, prompt);
                }
                for lease in leases {
                    request_lease(&host, &mut waiting, &mut leased, lease);
                }
                if let Some(outcome) = outcome
                    && settled
                {
                    waiting.abort_all();
                    return Ok(outcome);
                }
                continue;
            }
            tokio::select! {
                () = &mut notified => {}
                Some(answered) = waiting.join_next(), if !waiting.is_empty() => {
                    if let Ok(answered) = answered {
                        self.answered(&token, answered);
                    }
                }
            }
        }
    }

    pub fn signal(&self, signal: ExecutionSignal) -> Result<SignalReply, Refusal> {
        let token = signal.execution_token.as_str();
        if signal.cancel {
            self.cancel(token);
        }
        if let Some(delivery) = &signal.record {
            let record: dokime_domain::model::records::evidence::Record = serde_json::from_str(&delivery.record_json)
                .map_err(|error| Refusal::of(format!("invalid delivered record JSON: {error}")))?;
            let mut scopes = Vec::with_capacity(delivery.captured_scopes.len());
            for encoded in &delivery.captured_scopes {
                scopes.push(
                    serde_json::from_str::<dokime_domain::model::execution::RecordScope>(encoded)
                        .map_err(|error| Refusal::of(error.to_string()))?,
                );
            }
            self.deliver_captured(delivery.ordinal, &record, &self.source, &scopes);
        }
        if let Some(samples) = &signal.samples {
            let scope: dokime_domain::model::execution::RecordScope =
                serde_json::from_str(&samples.scope_json).map_err(|error| Refusal::of(format!("invalid sample scope JSON: {error}")))?;
            let mut rows = samples
                .samples
                .iter()
                .map(|encoded| serde_json::from_str(encoded))
                .collect::<Result<Vec<crate::telemetry::TelemetrySample>, _>>()
                .map_err(|error| Refusal::of(format!("invalid delivered sample JSON: {error}")))?;
            for row in &mut rows {
                row.scope = Some(delivery_scope(row.scope.take(), &scope));
            }
            let gaps = samples
                .gaps
                .iter()
                .map(|encoded| serde_json::from_str(encoded))
                .collect::<Result<Vec<crate::telemetry::LeaseGap>, _>>()
                .map_err(|error| Refusal::of(format!("invalid delivered gap JSON: {error}")))?;
            self.deliver_samples(&scope, &rows, &gaps)?;
        }
        let command = match signal.command {
            Some(command) => Some(self.command(command)?),
            None => None,
        };
        let samples = crate::transport::buffer::lock(&self.executions).get(token).map(|state| state.sample_receipts()).unwrap_or_default();
        let record_ordinal = crate::transport::buffer::lock(&self.executions).get(token).map_or(0, |state| state.record_ordinal);
        Ok(SignalReply { command, samples, record_ordinal })
    }

    fn answered(&self, token: &str, answered: Answered) {
        match answered {
            Answered::Prompt { prompt_id, answer } => match answer {
                Ok(answer) => {
                    if let Err(refusal) = self.answer(token, &prompt_id, &answer.response_json) {
                        tracing::warn!(%prompt_id, "the operator answer was refused: {refusal}");
                    }
                }
                Err(reason) => {
                    tracing::warn!(%prompt_id, "the host did not answer the question: {reason}");
                    self.cancel(token);
                }
            },
            Answered::Lease { lease_token, resolution } => match resolution {
                Ok(resolution) => {
                    self.resolve_lease(token, &lease_token, resolution.granted, &resolution.message);
                }
                Err(reason) => self.resolve_lease(token, &lease_token, false, &reason),
            },
        }
    }
}

impl<T: SidecarTemplate> Runnable for Workflow<T> {
    fn template_identifier(&self) -> &str {
        Workflow::template_identifier(self)
    }

    fn run(
        &self, request: RunExecution, host: Arc<dyn ExecutionHost>, output: Option<SampleOutput>, caller: Caller,
    ) -> TransportFuture<'_, Result<SidecarExecutionOutcome, Refusal>> {
        Box::pin(self.execute(request, host, output, caller))
    }

    fn signal(&self, signal: ExecutionSignal) -> TransportFuture<'_, Result<SignalReply, Refusal>> {
        Box::pin(std::future::ready(Workflow::signal(self, signal)))
    }

    fn take_local_samples(&self, token: &str) -> Result<Vec<crate::telemetry::TelemetrySample>, Refusal> {
        Workflow::take_local_samples(self, token)
    }

    fn interrupt(&self) {
        Workflow::interrupt(self);
    }

    fn shutdown(&self) -> TransportFuture<'_, ()> {
        Workflow::shutdown(self)
    }
}

/* === Public Functions === */

pub fn contracts(template: &str) -> Result<[harmos::operations::Contract; 2], harmos::operations::ContractError> {
    contracts_with_settings(template, &harmos::config::SchemaSettings::default())
}

pub fn contracts_with_settings(
    template: &str, settings: &harmos::config::SchemaSettings,
) -> Result<[harmos::operations::Contract; 2], harmos::operations::ContractError> {
    // The identity fingerprint covers the operation id, so a renamed contract
    // carries the fingerprint of its own id over the shared request and response.
    fn named<O: OperationDeclaration>(
        template: &str, suffix: &str, settings: &harmos::config::SchemaSettings,
    ) -> Result<harmos::operations::Contract, harmos::operations::ContractError> {
        use harmos::Portable;
        let contract = harmos::operations::WireDeclaration::of::<O>().contract_with_settings(settings)?;
        let mut operation = contract.operation().clone();
        operation.id = harmos::Identifier::new(format!("{template}.{suffix}"))
            .map_err(|source| harmos::operations::ContractError::Operation(source.into()))?;
        let fingerprint = harmos::schema::identity::operation(
            operation.id.as_str(),
            <O::Request as Portable>::fingerprint(),
            <O::Response as Portable>::fingerprint(),
            None,
            None,
        );
        operation.fingerprint = fingerprint;
        harmos::operations::Contract::new_with_settings(operation, fingerprint, contract.codec(), contract.schema().clone(), settings)
    }
    Ok([named::<Run<()>>(template, "run", settings)?, named::<Signal<()>>(template, "signal", settings)?])
}

pub fn prepare<P, S>(setup: &mut harmos::Setup<P>) -> Result<(), Refusal>
where
    P: harmos::Plugin + harmos::plugin::Declared,
    S: crate::catalog::Catalog<crate::catalog::SuiteBindings<P>>,
{
    for bind in crate::catalog::entries::<S, crate::catalog::SuiteBindings<P>>() {
        crate::plugin::refusal(bind(setup, "", &[]))?;
    }
    Ok(())
}

pub async fn interrupt() {
    let served = std::mem::take(&mut *registry().lock().unwrap_or_else(PoisonError::into_inner));
    for workflow in &served {
        workflow.interrupt();
    }
    for workflow in served {
        workflow.shutdown().await;
    }
}

pub fn runner<T: SidecarTemplate>(workflow: &str, records: &[crate::records::RecordContract]) -> Arc<dyn Runnable> {
    Arc::new(Workflow::<T>::new(workflow, "").with_records(records))
}

pub fn bind<T, P>(setup: &mut harmos::Setup<P>, workflow: &str, records: &[crate::records::RecordContract]) -> Result<(), ServingError>
where
    T: SidecarTemplate,
    P: harmos::Plugin + harmos::plugin::Declared,
{
    let workflow = Arc::new(Workflow::<T>::new(workflow, P::ID).with_records(records).with_publisher(T::publish_sample::<P>));
    registry().lock().unwrap_or_else(PoisonError::into_inner).push(Arc::clone(&workflow) as Arc<dyn Runnable>);
    setup.resource::<Workflow<T>>(workflow);
    Ok(())
}

/* === Private Functions === */

fn registry() -> &'static Mutex<Vec<Arc<dyn Runnable>>> {
    static SERVED: OnceLock<Mutex<Vec<Arc<dyn Runnable>>>> = OnceLock::new();
    SERVED.get_or_init(Mutex::default)
}

fn ask(host: &Arc<dyn ExecutionHost>, waiting: &mut tokio::task::JoinSet<Answered>, asked: &mut BTreeSet<String>, prompt_json: String) {
    let Ok(request) = serde_json::from_str::<crate::prompts::PromptRequest>(&prompt_json) else {
        tracing::warn!("a raised prompt could not be read back");
        return;
    };
    let prompt_id = request.prompt_id.to_string();
    if !asked.insert(prompt_id.clone()) {
        return;
    }
    let host = Arc::clone(host);
    waiting.spawn(async move {
        let answer = host.ask(OperatorQuestion { prompt_json }).await;
        Answered::Prompt { prompt_id, answer }
    });
}

fn request_lease(
    host: &Arc<dyn ExecutionHost>, waiting: &mut tokio::task::JoinSet<Answered>, leased: &mut BTreeSet<String>, lease_json: String,
) {
    let Ok(request) = serde_json::from_str::<crate::telemetry::LeaseRequest>(&lease_json) else {
        tracing::warn!("a raised lease could not be read back");
        return;
    };
    let lease_token = request.lease_token.clone();
    if !leased.insert(lease_token.clone()) {
        return;
    }
    let host = Arc::clone(host);
    waiting.spawn(async move {
        let resolution = host.lease(LeaseAsk { lease_json }).await;
        Answered::Lease { lease_token, resolution }
    });
}

// One Delivered Frame Serves Every Leaseholder, So Each Run Stamps Rows With The Scope Its Request
// Names, Keeping The Producer's Own Scope Only For Rows That Execution Captured Itself.
fn delivery_scope(
    source: Option<dokime_domain::model::execution::RecordScope>, holder: &dokime_domain::model::execution::RecordScope,
) -> dokime_domain::model::execution::RecordScope {
    source
        .filter(|source| source.test_execution_id.is_some() && source.test_execution_id == holder.test_execution_id)
        .unwrap_or_else(|| holder.clone())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::delivery_scope;
    use dokime_domain::model::execution::EvidenceOrigin;
    use dokime_domain::model::execution::RecordScope;
    use dokime_domain::model::identity::TestExecutionId;

    #[tokio::test(start_paused = true)]
    async fn receipt_admission_closes_at_operation_policy_expiry() {
        use crate::transport::buffer::{ExecutionState, lock};
        use harmos::operations::OperationDeclaration;
        use std::sync::{Arc, Mutex};
        let (cancel, _) = tokio::sync::watch::channel(false);
        let state = ExecutionState::new(Default::default(), cancel);
        let changed = Arc::clone(&state.changed);
        let executions = Arc::new(Mutex::new(std::collections::BTreeMap::from([("fixture".into(), state)])));
        let admission = super::InputAdmission {
            executions: Arc::clone(&executions),
            token: "fixture".into(),
            changed,
        };
        let policy = crate::transport::EvidenceBatch::POLICY.timeout;
        let operation = tokio::spawn(async move {
            tokio::time::timeout(policy, async move {
                let _admission = admission;
                std::future::pending::<()>().await;
            })
            .await
        });
        tokio::task::yield_now().await;
        tokio::time::advance(policy / 2).await;
        assert!(lock(&executions)["fixture"].samples_open());
        tokio::time::advance(policy - policy / 2).await;
        assert!(operation.await.unwrap().is_err());
        let states = lock(&executions);
        let state = &states["fixture"];
        assert!(state.record_inputs_closed());
        assert!(!state.samples_open());
        assert!(state.cancelled());
    }

    #[test]
    fn delivery_preserves_producer_origin_for_its_execution() {
        let holder = RecordScope {
            test_execution_id: Some(TestExecutionId::from_seed("delivery-holder")),
            ..Default::default()
        };
        let source = RecordScope {
            origin: EvidenceOrigin::Initialization,
            ..holder.clone()
        };
        assert_eq!(delivery_scope(Some(source.clone()), &holder), source);
        assert_eq!(delivery_scope(None, &holder), holder);
        let other = RecordScope {
            test_execution_id: Some(TestExecutionId::from_seed("delivery-other")),
            ..Default::default()
        };
        assert_eq!(delivery_scope(Some(source), &other), other);
    }
}
