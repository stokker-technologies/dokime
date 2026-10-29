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

/* === Enums === */

#[harmos::catalog]
enum PreparedOperations {
    Operation0(EvidenceBatch),
    Operation1(OperatorQuestion),
    Operation2(LeaseAsk),
}

/* === Traits === */

/* === Implementations === */

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

    fn get(&self, execution: &str) -> Option<Arc<Session>> {
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
