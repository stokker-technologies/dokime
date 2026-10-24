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
use std::time::Duration;

// Third Party Imports
use harmos::operations::{Host, NoStream, OperationDeclaration, OperationPolicy, RestartPolicy, RetryPolicy, Visibility};
use harmos::sidecar::operations::{ServingError, WireTable};
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

const IDLE: Duration = Duration::from_millis(250);
const PUSH_RETRIES: usize = 5;
const RUN_POLICY: OperationPolicy =
    OperationPolicy { timeout: RUN_BOUND, attempt_timeout: RUN_BOUND, retries: RetryPolicy::Never, restart: RestartPolicy::Never };
pub const SIGNAL_POLICY: OperationPolicy =
    OperationPolicy { timeout: SIGNAL_BOUND, attempt_timeout: SIGNAL_BOUND, retries: RetryPolicy::Never, restart: RestartPolicy::Never };

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

pub struct Running<T: SidecarTemplate>(Arc<Workflow<T>>);

pub struct Signalling<T: SidecarTemplate>(Arc<Workflow<T>>);

/* === Enums === */

enum Answered {
    Prompt { prompt_id: String, answer: Result<OperatorAnswer, String> },
    Lease { lease_token: String, resolution: Result<LeaseResolution, String> },
}

/* === Traits === */

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
    type Response = SidecarExecutionOutcome;
    type Input = NoStream;
    type Output = NoStream;
    const ID: &'static str = T::RUN;
    const VERSION: u32 = 1;
    const DESCRIPTION: &'static str = "Runs one case of this template to its outcome.";
    const POLICY: OperationPolicy = RUN_POLICY;
    const VISIBILITY: Visibility = Visibility::Host;
}

impl<T: Served> OperationDeclaration for Signal<T> {
    type Response = SignalReply;
    type Input = NoStream;
    type Output = NoStream;
    const ID: &'static str = T::SIGNAL;
    const VERSION: u32 = 1;
    const DESCRIPTION: &'static str = "Reaches one running case of this template.";
    const POLICY: OperationPolicy = T::SIGNAL_POLICY;
    const VISIBILITY: Visibility = Visibility::Host;
}

impl<T: SidecarTemplate> harmos::operations::Operation for Running<T> {
    type Declaration = Run<T>;
    type Resources = Workflow<T>;
    type Error = Refusal;
    type Context<'a> = &'a Supervisor;
    type Endpoints = ();

    async fn initialize<'ctx>(workflow: Arc<Workflow<T>>, _: Self::Context<'ctx>) -> Result<Self, Refusal> {
        Ok(Self(workflow))
    }

    async fn run<'ctx>(
        &mut self, host: Self::Context<'ctx>, request: &mut Run<T>, _: &mut (),
    ) -> Result<harmos::operations::Response<SidecarExecutionOutcome>, Refusal> {
        let request = std::mem::take(&mut request.0);
        let outcome =
            self.0.execute(request, Arc::new(host.clone()), Some(SampleOutput::Host(host.clone())), Caller::host(host.clone())).await?;
        Ok(outcome.into())
    }
}

impl<T: SidecarTemplate> harmos::operations::Operation for Signalling<T> {
    type Declaration = Signal<T>;
    type Resources = Workflow<T>;
    type Error = Refusal;
    type Context<'a> = &'a Supervisor;
    type Endpoints = ();

    async fn initialize<'ctx>(workflow: Arc<Workflow<T>>, _: Self::Context<'ctx>) -> Result<Self, Refusal> {
        Ok(Self(workflow))
    }

    async fn run<'ctx>(
        &mut self, _: Self::Context<'ctx>, signal: &mut Signal<T>, _: &mut (),
    ) -> Result<harmos::operations::Response<SignalReply>, Refusal> {
        self.0.signal(std::mem::take(&mut signal.0)).map(Into::into)
    }
}

impl ExecutionHost for Supervisor {
    fn push(&self, batch: EvidenceBatch) -> TransportFuture<'_, Result<EvidenceReceipt, String>> {
        Box::pin(async move { self.call(batch).await.map(|response| response.into_owned()).map_err(|error| error.to_string()) })
    }

    fn ask(&self, question: OperatorQuestion) -> TransportFuture<'_, Result<OperatorAnswer, String>> {
        Box::pin(async move { self.call(question).await.map(|response| response.into_owned()).map_err(|error| error.to_string()) })
    }

    fn lease(&self, ask: LeaseAsk) -> TransportFuture<'_, Result<LeaseResolution, String>> {
        Box::pin(async move { self.call(ask).await.map(|response| response.into_owned()).map_err(|error| error.to_string()) })
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
                let receipt = push(host.as_ref(), batch).await?;
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
                () = tokio::time::sleep(IDLE) => {}
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
            self.deliver_captured(&record, &self.source, &scopes);
        }
        if let Some(samples) = &signal.samples {
            let scope: dokime_domain::model::execution::RecordScope =
                serde_json::from_str(&samples.scope_json).map_err(|error| Refusal::of(format!("invalid sample scope JSON: {error}")))?;
            let rows = samples
                .samples
                .iter()
                .map(|encoded| serde_json::from_str(encoded))
                .collect::<Result<Vec<crate::telemetry::TelemetrySample>, _>>()
                .map_err(|error| Refusal::of(format!("invalid delivered sample JSON: {error}")))?;
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
        Ok(SignalReply { command })
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
    let named = |wire: harmos::operations::WireDeclaration, suffix: &str| {
        let contract = wire.contract()?;
        let mut operation = contract.operation().clone();
        operation.id = harmos::Identifier::new(format!("{template}.{suffix}"))
            .map_err(|source| harmos::operations::ContractError::Operation(source.into()))?;
        harmos::operations::Contract::new(operation, contract.version(), contract.codec(), contract.schema().clone())
    };
    Ok([
        named(harmos::operations::WireDeclaration::of::<Run<()>>(), "run")?,
        named(harmos::operations::WireDeclaration::of::<Signal<()>>(), "signal")?,
    ])
}

pub fn serve<P, S>(table: &mut WireTable<Supervisor>) -> Result<(), ServingError>
where
    P: harmos::plugin::Declared,
    S: crate::catalog::Catalog<crate::catalog::SuiteBindings<P>>,
{
    for bind in crate::catalog::entries::<S, crate::catalog::SuiteBindings<P>>() {
        bind(table, "", &[])?;
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

pub fn bind<T, P>(table: &mut WireTable<Supervisor>, workflow: &str, records: &[crate::records::RecordContract]) -> Result<(), ServingError>
where
    T: SidecarTemplate,
    P: harmos::plugin::Declared,
{
    let workflow = Arc::new(Workflow::<T>::new(workflow, P::ID).with_records(records).with_publisher(T::publish_sample::<P>));
    registry().lock().unwrap_or_else(PoisonError::into_inner).push(Arc::clone(&workflow) as Arc<dyn Runnable>);
    table.bind_operation::<Running<T>>(Arc::clone(&workflow))?;
    table.bind_operation::<Signalling<T>>(workflow)
}

/* === Private Functions === */

fn registry() -> &'static Mutex<Vec<Arc<dyn Runnable>>> {
    static SERVED: OnceLock<Mutex<Vec<Arc<dyn Runnable>>>> = OnceLock::new();
    SERVED.get_or_init(Mutex::default)
}

async fn push(host: &dyn ExecutionHost, batch: EvidenceBatch) -> Result<EvidenceReceipt, Refusal> {
    let mut failure = String::new();
    for attempt in 0..PUSH_RETRIES {
        match host.push(batch.clone()).await {
            Ok(receipt) => return Ok(receipt),
            Err(reason) => failure = reason,
        }
        tokio::time::sleep(Duration::from_millis(50 << attempt)).await;
    }
    Err(Refusal::of(format!("the host did not admit the run's evidence: {failure}")))
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

/* === Tests === */
