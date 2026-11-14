/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Pending Operator Questions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

// Third Party Imports
use async_trait::async_trait;
use prost::Message;
use tokio::sync::{Notify, oneshot};

// Local Imports
use crate::{Error, Result, SchedulerClock, host::lock};
use dokime_operations_next::Prompts;
use dokime_storage::Store;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct PromptManager {
    store: Store,
    clock: Arc<dyn SchedulerClock>,
    closed: std::sync::atomic::AtomicBool,
    pending: Mutex<HashMap<String, Pending>>,
    changed: Notify,
    runtime: tokio::sync::watch::Sender<u64>,
}
struct Pending {
    question: dokime_protocol::context::PromptsCreateRequest,
    answer: Option<oneshot::Sender<dokime_protocol::context::PromptsCreateResponse>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PromptManager {
    pub(crate) fn new(store: Store, clock: Arc<dyn SchedulerClock>, runtime: tokio::sync::watch::Sender<u64>) -> Self {
        Self {
            store,
            clock,
            runtime,
            pending: Mutex::new(HashMap::new()),
            changed: Notify::new(),
            closed: std::sync::atomic::AtomicBool::new(false),
        }
    }
    pub(crate) async fn next(
        &self, execution: &str, executions: &crate::execution_manager::ExecutionManager,
    ) -> Result<dokime_protocol::context::PromptsCreateRequest> {
        loop {
            let changed = self.changed.notified();
            if let Some(pending) = lock(&self.pending)?.get(execution) {
                return Ok(pending.question.clone());
            }
            if self.closed.load(std::sync::atomic::Ordering::Acquire) || !executions.is_active(execution)? {
                return Err(Error::Invalid(format!("Wait for Execution Prompt {execution}: Start an Active Run")));
            }
            changed.await;
        }
    }
    pub(crate) fn close_admission(&self) -> Result<()> {
        let _pending = lock(&self.pending)?;
        self.closed.store(true, std::sync::atomic::Ordering::Release);
        self.changed.notify_waiters();
        Ok(())
    }
    pub(crate) fn shutdown(&self) -> Result<()> {
        let mut pending = lock(&self.pending)?;
        self.closed.store(true, std::sync::atomic::Ordering::Release);
        pending.clear();
        self.changed.notify_waiters();
        self.runtime.send_modify(|revision| *revision += 1);
        Ok(())
    }
    pub fn pending_prompts(&self) -> Result<Vec<(String, dokime_protocol::context::PromptsCreateRequest)>> {
        Ok(lock(&self.pending)?
            .iter()
            .filter(|(_, pending)| pending.answer.as_ref().is_some_and(|answer| !answer.is_closed()))
            .map(|(execution, pending)| (execution.clone(), pending.question.clone()))
            .collect())
    }
    pub fn answer(&self, execution: &str, answer: dokime_protocol::context::PromptsCreateResponse) -> Result<()> {
        let mut pending = lock(&self.pending)?;
        let prompt = pending.get_mut(execution).ok_or_else(|| Error::Invalid("Answer Prompt: Wait for an Execution Question".into()))?;
        if answer.prompt != prompt.question.prompt {
            return Err(Error::Invalid("Answer Prompt: Select the Current Question".into()));
        }
        prompt
            .answer
            .take()
            .ok_or_else(|| Error::Invalid("Answer Prompt: Answer Each Question Only Once".into()))?
            .send(answer)
            .map_err(|_| Error::Invalid("Answer Prompt: The Execution Stopped; Inspect Its Evidence".into()))?;
        Ok(())
    }
    pub fn record_answer(
        &self, execution: &str, request_id: &str, fingerprint: &[u8], answer: dokime_protocol::context::PromptsCreateResponse,
    ) -> Result<harmos_protocol::v1::Receipt> {
        if let Some((_, receipt)) =
            self.store.requested_evidence(execution, "dokime.client.v1.prompts.Answers/Create", request_id, fingerprint)?
        {
            return Ok(receipt);
        }
        let mut pending = lock(&self.pending)?;
        let prompt = pending.get_mut(execution).ok_or_else(|| Error::Invalid("Answer Prompt: Wait for an Active Question".into()))?;
        if prompt.question.prompt != answer.prompt || prompt.answer.as_ref().is_none_or(oneshot::Sender::is_closed) {
            return Err(Error::Invalid("Answer Prompt: Inspect the Current Question Before Answering".into()));
        }
        let (_, receipt) = self.store.append_requested(
            execution,
            "dokime.client.v1.prompts.Answers/Create",
            request_id,
            fingerprint,
            dokime_storage::Evidence {
                kind: "client:answer".into(),
                receipt_ns: self.clock.now_ns(),
                payload: answer.encode_to_vec(),
                ..Default::default()
            },
        )?;
        let _ = prompt.answer.take().unwrap().send(answer);
        Ok(receipt)
    }
}
#[async_trait]
impl Prompts for PromptManager {
    async fn ask(
        &self, execution: &str, question: dokime_protocol::context::PromptsCreateRequest,
    ) -> dokime_operations_next::Result<dokime_protocol::context::PromptsCreateResponse> {
        let (send, receive) = oneshot::channel();
        {
            let mut pending = self
                .pending
                .lock()
                .map_err(|_| dokime_operations_next::Error::Invalid("Restart the Host After Its Prompt Store Failed".into()))?;
            if self.closed.load(std::sync::atomic::Ordering::Acquire) {
                return Err(dokime_operations_next::Error::Invalid("Ask Prompt: Reconnect to a Running Host".into()));
            }
            if pending.contains_key(execution) {
                return Err(dokime_operations_next::Error::Invalid("Answer the Current Prompt Before Asking Another".into()));
            }
            pending.insert(execution.into(), Pending { question, answer: Some(send) });
        }
        self.changed.notify_waiters();
        self.runtime.send_modify(|revision| *revision += 1);
        let answer =
            receive.await.map_err(|_| dokime_operations_next::Error::Invalid("Execution Stopped Before the Operator Answered".into()));
        self.clear(execution);
        answer
    }
}
impl PromptManager {
    pub(crate) fn clear(&self, execution: &str) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.remove(execution);
        }
        self.changed.notify_waiters();
        self.runtime.send_modify(|revision| *revision += 1);
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
