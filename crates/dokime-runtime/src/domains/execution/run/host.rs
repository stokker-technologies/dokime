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
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, PoisonError};

// Third Party Imports
use harmos::Context;
use harmos::operations::HostContext;
use tonic::Status;

// Local Imports
use super::session::Session;
use crate::execution::ExecutionScope;
use crate::operations::hosting::{Host, Serve, Served};
use crate::transport::{EvidenceBatch, EvidenceReceipt, LeaseAsk, LeaseResolution, OperatorAnswer, OperatorQuestion, TransportFuture};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub(crate) struct Sessions {
    open: Mutex<BTreeMap<String, Arc<Session>>>,
}

pub(crate) struct Embedded {
    sessions: Arc<Sessions>,
    execution: String,
}

#[derive(Clone)]
pub(crate) struct ObserverEvidence {
    sessions: Arc<Sessions>,
    source: String,
}

struct Serving(Arc<Sessions>);

struct AttemptEvidence(Arc<Sessions>);

pub(super) struct AttemptOwner(pub(super) String);

/* === Enums === */

#[harmos::catalog]
enum PreparedOperations {
    Operation0(EvidenceBatch),
    Operation1(OperatorQuestion),
    Operation2(LeaseAsk),
}

/* === Traits === */

/* === Implementations === */

impl harmos::operations::middleware::Middleware for AttemptEvidence {
    fn enter<'a>(&'a self, call: &'a mut harmos::operations::middleware::Invocation) -> harmos::operations::middleware::Enter<'a> {
        Box::pin(async move {
            if let (Some(owner), Some(scope)) = (call.context().get::<AttemptOwner>(), ExecutionScope::current(call.context()))
                && owner.0 != scope.execution
            {
                return Err(harmos::Error::PermissionDenied);
            }
            Ok(())
        })
    }

    fn attempt<'a>(
        &'a self, call: &'a harmos::operations::middleware::Invocation, observation: harmos::operations::middleware::AttemptObservation,
    ) -> harmos::operations::middleware::Enter<'a> {
        Box::pin(async move {
            let Some(scope) = ExecutionScope::current(call.context()) else { return Ok(()) };
            let Some(session) = self.0.get(&scope.execution) else { return Ok(()) };
            // The Host Retains This Local Owner Across Carriers And Child Providers.
            // Portable Execution Metadata Alone Never Grants Evidence Ownership.
            if call.context().get::<AttemptOwner>().is_none_or(|owner| owner.0 != scope.execution) {
                return Ok(());
            }
            session.record_operation_attempt(&scope.stage, call, observation).await;
            Ok(())
        })
    }
}

impl ObserverEvidence {
    pub(crate) fn new(sessions: Arc<Sessions>, source: String) -> Self {
        Self { sessions, source }
    }
}

impl harmos::sidecar::ObserverHost for ObserverEvidence {
    fn bind(resources: Arc<harmos::ExecutionContext>, _: harmos::sidecar::Supervisor) -> Self {
        resources.resource::<Self>().expect("observer evidence is registered before binding").as_ref().clone()
    }

    fn report_cadence(&self, report: &harmos::__private::sidecar::CadenceReport) {
        let sessions = self.sessions.lock().values().cloned().collect::<Vec<_>>();
        for session in sessions {
            if session.observe_cadence(&self.source, report) {
                tokio::spawn(async move { session.flush_cadence().await });
            }
        }
    }
}

impl Sessions {
    pub(super) fn insert(&self, execution: &str, session: Arc<Session>) {
        self.lock().insert(execution.to_owned(), session);
    }

    pub(super) fn remove(&self, execution: &str) {
        self.lock().remove(execution);
    }

    pub(crate) fn signal_command(&self, execution: &str) {
        if let Some(session) = self.get(execution) {
            session.signal_command();
        }
    }

    pub(crate) async fn deliver_records(&self, execution: &str) -> bool {
        let Some(session) = self.get(execution) else { return false };
        if let Err(error) = session.deliver_records().await {
            session.refuse(error).await;
        }
        true
    }

    pub(crate) fn get(&self, execution: &str) -> Option<Arc<Session>> {
        self.lock().get(execution).cloned()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, Arc<Session>>> {
        self.open.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn serving<O: harmos::Operation + harmos::operations::Requires<HostContext>>(
        &self, context: &Context<'_, O>,
    ) -> Result<Arc<Session>, Status> {
        let scope = context
            .call()
            .value::<ExecutionScope>()
            .ok()
            .flatten()
            .ok_or_else(|| Status::invalid_argument("the call is not part of a template run"))?;
        let session = self
            .get(&scope.execution)
            .ok_or_else(|| Status::not_found(format!("execution session {} is not registered", scope.execution)))?;
        let host = context.resource::<HostContext>();
        if let Some(caller) = host.borrow().caller_scope()
            && caller.owner() != session.source()
        {
            return Err(Status::permission_denied("the caller does not own this execution"));
        }
        Ok(session)
    }
}

impl Embedded {
    pub(crate) fn new(sessions: Arc<Sessions>, execution: impl Into<String>) -> Self {
        Self { sessions, execution: execution.into() }
    }

    fn session(&self) -> Result<Arc<Session>, String> {
        self.sessions.get(&self.execution).ok_or_else(|| format!("execution {} is not running", self.execution))
    }
}

impl crate::execution::ExecutionHost for Embedded {
    fn push(&self, batch: EvidenceBatch) -> TransportFuture<'_, Result<EvidenceReceipt, String>> {
        Box::pin(async move { Ok(self.session()?.accept(batch).await) })
    }

    fn ask(&self, question: OperatorQuestion) -> TransportFuture<'_, Result<OperatorAnswer, String>> {
        Box::pin(async move { self.session()?.ask(question).await })
    }

    fn lease(&self, ask: LeaseAsk) -> TransportFuture<'_, Result<LeaseResolution, String>> {
        Box::pin(async move { Ok(self.session()?.lease(ask).await) })
    }
}

impl Serve<EvidenceBatch> for Serving {
    fn serve<'a>(&'a self, batch: EvidenceBatch, ctx: Context<'a, EvidenceBatch>) -> Served<'a, EvidenceReceipt> {
        Box::pin(async move {
            let session = self.0.serving(&ctx)?;
            Ok(session.accept(batch).await.into())
        })
    }
}

impl Serve<OperatorQuestion> for Serving {
    fn serve<'a>(&'a self, question: OperatorQuestion, ctx: Context<'a, OperatorQuestion>) -> Served<'a, OperatorAnswer> {
        Box::pin(async move {
            let session = self.0.serving(&ctx)?;
            session
                .ask(question)
                .await
                .map(Into::into)
                .map_err(|_| Status::failed_precondition("the question was refused or the execution was cancelled"))
        })
    }
}

impl Serve<LeaseAsk> for Serving {
    fn serve<'a>(&'a self, ask: LeaseAsk, ctx: Context<'a, LeaseAsk>) -> Served<'a, LeaseResolution> {
        Box::pin(async move {
            let session = self.0.serving(&ctx)?;
            Ok(session.lease(ask).await.into())
        })
    }
}

/* === Public Functions === */

pub(crate) fn prepare(setup: &mut harmos::Setup<crate::app::Dokime>, sessions: &Arc<Sessions>) -> harmos::Result<()> {
    setup.settings().operation_middleware(Arc::new(AttemptEvidence(Arc::clone(sessions))));
    let served = Arc::new(Serving(Arc::clone(sessions)));

    setup.resource::<Host<EvidenceBatch>>(Host::<EvidenceBatch>::new(served.clone()));
    setup.resource::<Host<OperatorQuestion>>(Host::<OperatorQuestion>::new(served.clone()));
    setup.resource::<Host<LeaseAsk>>(Host::<LeaseAsk>::new(served));

    {
        setup.operations::<PreparedOperations>(crate::app::Lanes::Open);
        Ok(())
    }
}

/* === Private Functions === */

/* === Tests === */

#[cfg(feature = "testing")]
impl crate::RuntimeHandle {
    pub fn retained_attempt_context(&self, execution: crate::TestExecution, operation: &str) -> harmos::operations::CallContext {
        let token = execution.test_execution_id.to_string();
        let session = Arc::new(super::session::Session::new(super::session::Opened {
            context: super::ExecutionFixtureContext::new(self.clone()),
            execution,
            source: "host".into(),
            target: super::Target::of(self, operation),
            prompts: super::PromptBridge::new(Vec::new()),
            phase: Default::default(),
        }));
        self.execution_sessions.insert(&token, session);
        let mut context = harmos::operations::CallContext::default();
        context.insert(AttemptOwner(token)).expect("fixture context has no owner");
        context
    }
    pub fn remove_retained_attempt(&self, token: &str) {
        self.execution_sessions.remove(token);
    }
}
