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

#[cfg(test)]
mod tests {
    use super::*;
    use harmos::operations::OperationDeclaration;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    static ATTEMPTS: AtomicUsize = AtomicUsize::new(0);

    #[harmos::operation(resources = [harmos::operations::HostContext], id = "invocation.evidence", response = u64,
        replay = harmos::operations::ReplaySafety::Idempotent,
        policy = harmos::OperationPolicy { timeout: Duration::from_secs(5), attempt_timeout: Duration::from_secs(1),
            cancellation_timeout: Duration::ZERO, retries: harmos::RetryPolicy::Never })]
    struct Read {}
    impl harmos::Operation for Read {
        type Error = std::io::Error;
        async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<u64>, Self::Error> {
            ATTEMPTS.fetch_add(1, Ordering::SeqCst);
            if ctx.resource::<harmos::operations::HostContext>().borrow().attempt() == 0 {
                return Err(std::io::Error::other("transient read failure"));
            }
            Ok(42.into())
        }
        fn classify(_: &Self::Error) -> harmos::operations::FailureKind {
            harmos::operations::FailureKind::Transient
        }
    }
    #[harmos::catalog]
    enum Operations {
        Read(Read),
    }

    #[tokio::test]
    async fn retry_attempts_are_persisted_under_the_retained_execution_owner() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../report/fixtures/execution-report-input.json")).unwrap();
        let mut cycle: dokime_domain::model::execution::result::CycleView =
            serde_json::from_value(fixture["input"]["cycle"].clone()).unwrap();
        cycle.test_executions[0].saved = None;
        let execution = &cycle.test_executions[0];
        let root = tempfile::tempdir().unwrap();
        let mut owner = crate::Runtime::start(crate::RuntimeConfig::new(root.path(), cycle.metadata.clone())).await.unwrap();
        let runtime = owner.handle();
        runtime.execution_storage.acquire(&cycle, execution).await.unwrap();
        runtime.operations().register::<Operations>(crate::app::Lanes::Open).await.unwrap();
        let token = execution.test_execution_id.to_string();
        let session = Arc::new(super::super::session::Session::new(super::super::session::Opened {
            context: super::super::ExecutionFixtureContext::new(runtime.clone()),
            execution: execution.clone(),
            source: "host".into(),
            target: super::super::Target::of(&runtime, "invocation.evidence"),
            prompts: super::super::PromptBridge::new(Vec::new()),
            phase: Default::default(),
        }));
        runtime.execution_sessions.insert(&token, session.clone());
        let mut context = harmos::operations::CallContext::default();
        context.insert(AttemptOwner(token.clone())).unwrap();
        context.insert_portable(ExecutionScope::execution(token.clone()).staged("step/read")).unwrap();
        let configured = harmos::RetryPolicy::OnTransient {
            max_retries: 1,
            backoff: harmos::Backoff::Fixed {
                delay: Duration::from_millis(1),
                jitter: harmos::Jitter::None,
            },
        };
        let operations = runtime.operations().with_context(context.clone());
        assert_eq!(*operations.operation::<Read>(Read {}).retry(configured).await.unwrap(), 42);
        assert!(operations.operation::<Read>(Read {}).await.is_err());
        let before = ATTEMPTS.load(Ordering::SeqCst);
        let tampered = context.with_value(ExecutionScope::execution("another-execution")).unwrap();
        assert!(matches!(
            runtime.operations().with_context(tampered).operation::<Read>(Read {}).await,
            Err(harmos::Error::PermissionDenied)
        ));
        assert_eq!(ATTEMPTS.load(Ordering::SeqCst), before);
        let mut unowned = harmos::operations::CallContext::default();
        unowned.insert_portable(ExecutionScope::execution(token.clone()).staged("step/forged")).unwrap();
        assert!(runtime.operations().with_context(unowned).operation::<Read>(Read {}).await.is_err());
        let content =
            runtime.execution_storage.native_read(&execution.test_execution_id, "artifacts/execution.log", 0, 8192).await.unwrap().unwrap();
        let text = std::str::from_utf8(&content.bytes).unwrap();
        let entries = text
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(&line[line.find('{').unwrap()..]).unwrap())
            .collect::<Vec<_>>();
        let attempts =
            entries.iter().map(|entry| (entry["attempt"].as_u64().unwrap(), entry["outcome"].as_str().unwrap())).collect::<Vec<_>>();
        assert_eq!(attempts, [(0, "failed"), (1, "succeeded"), (0, "failed")]);
        assert!(entries.iter().all(|entry| entry["stage"] == "step/read" && entry["operation"] == Read::ID));
        assert_eq!(entries[0]["instance"], entries[1]["instance"]);
        assert_ne!(entries[1]["instance"], entries[2]["instance"]);
        drop(content);
        runtime.execution_sessions.remove(&token);
        drop(session);
        drop(runtime);
        owner.shutdown().await.unwrap();
    }
}
