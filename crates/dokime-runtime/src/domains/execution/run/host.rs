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
use harmos::operations::{Context, Operation, OperationManager};

// Local Imports
use super::session::Session;
use crate::execution::ExecutionScope;
use crate::transport::{EvidenceBatch, EvidenceReceipt, LeaseAsk, LeaseResolution, OperatorAnswer, OperatorQuestion, TransportFuture};

/* === Constants === */

const OWNER: &str = "dokime.execution";

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

struct Evidence(Arc<Sessions>);

struct Question(Arc<Sessions>);

struct Lease(Arc<Sessions>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Sessions {
    pub(super) fn insert(&self, execution: &str, session: Arc<Session>) {
        self.lock().insert(execution.to_owned(), session);
    }

    pub(super) fn remove(&self, execution: &str) {
        self.lock().remove(execution);
    }

    pub(crate) fn signal_change(&self, execution: &str) {
        if let Some(session) = self.get(execution) {
            session.signal_change();
        }
    }

    fn get(&self, execution: &str) -> Option<Arc<Session>> {
        self.lock().get(execution).cloned()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, Arc<Session>>> {
        self.open.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn serving(&self, context: &Context<'_>) -> harmos::Result<Arc<Session>> {
        let scope = context
            .context()
            .value::<ExecutionScope>()
            .ok()
            .flatten()
            .ok_or(harmos::Error::InvalidInput { field: "execution scope", reason: "the call is not part of a template run" })?;
        let session = self
            .get(&scope.execution)
            .ok_or_else(|| harmos::Error::NotRegistered { domain: "execution session", name: scope.execution.clone() })?;
        if let Some(caller) = context.caller_scope()
            && caller.owner() != session.source()
        {
            return Err(harmos::Error::InvalidInput { field: "execution scope", reason: "the caller does not own this execution" });
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

impl harmos::operations::Declares for Evidence {
    type Declaration = EvidenceBatch;
}

impl Operation for Evidence {
    type Declaration = EvidenceBatch;
    type Resources = Sessions;
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(sessions: Arc<Sessions>, _: Self::Context<'ctx>) -> harmos::Result<Self> {
        Ok(Self(sessions))
    }

    async fn run<'ctx>(
        &mut self, context: Self::Context<'ctx>, batch: &mut EvidenceBatch, _: &mut (),
    ) -> harmos::Result<harmos::operations::Response<EvidenceReceipt>> {
        let session = self.0.serving(&context)?;
        Ok(session.accept(std::mem::take(batch)).await.into())
    }
}

impl harmos::operations::Declares for Question {
    type Declaration = OperatorQuestion;
}

impl Operation for Question {
    type Declaration = OperatorQuestion;
    type Resources = Sessions;
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(sessions: Arc<Sessions>, _: Self::Context<'ctx>) -> harmos::Result<Self> {
        Ok(Self(sessions))
    }

    async fn run<'ctx>(
        &mut self, context: Self::Context<'ctx>, question: &mut OperatorQuestion, _: &mut (),
    ) -> harmos::Result<harmos::operations::Response<OperatorAnswer>> {
        let session = self.0.serving(&context)?;
        session.ask(std::mem::take(question)).await.map(Into::into).map_err(|_| harmos::Error::InvalidInput {
            field: "operator question",
            reason: "the question was refused or the execution was cancelled",
        })
    }
}

impl harmos::operations::Declares for Lease {
    type Declaration = LeaseAsk;
}

impl Operation for Lease {
    type Declaration = LeaseAsk;
    type Resources = Sessions;
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(sessions: Arc<Sessions>, _: Self::Context<'ctx>) -> harmos::Result<Self> {
        Ok(Self(sessions))
    }

    async fn run<'ctx>(
        &mut self, context: Self::Context<'ctx>, ask: &mut LeaseAsk, _: &mut (),
    ) -> harmos::Result<harmos::operations::Response<LeaseResolution>> {
        let session = self.0.serving(&context)?;
        Ok(session.lease(std::mem::take(ask)).await.into())
    }
}

/* === Public Functions === */

pub(crate) fn register(operations: &OperationManager, sessions: &Arc<Sessions>) -> harmos::Result<()> {
    operations.register_wire_operation::<Evidence>(OWNER, crate::app::Lanes::Open, Arc::clone(sessions))?;
    operations.register_wire_operation::<Question>(OWNER, crate::app::Lanes::Open, Arc::clone(sessions))?;
    operations.register_wire_operation::<Lease>(OWNER, crate::app::Lanes::Open, Arc::clone(sessions))
}

/* === Private Functions === */

/* === Tests === */
