/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod native;

/* === Re-Exports === */
pub use dokime::execution::request::{AttemptClaim, RunReceipt};

/* === Imports === */
// Standard Library Imports
use std::{collections::BTreeMap, sync::Arc};
// Third Party Imports
use harmos::storage::StorageCursor;
use tokio::sync::Mutex;
// Local Imports
use crate::{Error, Result};
use dokime::core::{CommandId, NodeId, Timestamp};
use dokime::execution::request::RunSummary;
use dokime::execution::request::{RunPhase, StepCommandReceipt};

/* === Constants === */
const RECOVERY_INTERRUPTION: &str = "Previous runtime ended before request completion. Device actions are not replayed; claimed acquisition may be interrupted.";
const INITIAL_INTERRUPTION: &str =
    "Initial receipt publication was interrupted; no provider work was submitted.";
const TERMINAL_INTERRUPTION: &str = "Run settled before command completion was recorded.";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Active {
    receipt: Arc<RunReceipt>,
    pending: Option<Pending<Arc<RunReceipt>>>,
    admitted: bool,
}

pub(crate) struct Pending<R> {
    receipt: R,
    previous: Option<R>,
    position: Option<StorageCursor>,
}

pub(crate) struct RequestStorage {
    storage: dokime::storage::Manager,
    node: NodeId,
    now: fn() -> Timestamp,
    active: Mutex<BTreeMap<CommandId, Active>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<R: Clone + std::ops::Deref<Target = RunReceipt>> Pending<R> {
    fn new(receipt: R) -> Self {
        Self {
            receipt,
            previous: None,
            position: None,
        }
    }

    fn changed(receipt: R, previous: &R) -> Self {
        Self {
            receipt,
            previous: Some(previous.clone()),
            position: None,
        }
    }

    fn changed_commands(&self) -> impl Iterator<Item = (&CommandId, &StepCommandReceipt)> {
        self.receipt.commands.iter().filter(|(id, command)| {
            self.previous
                .as_ref()
                .is_none_or(|previous| previous.commands.get(*id) != Some(*command))
        })
    }
}

impl RequestStorage {
    pub(crate) fn new(storage: dokime::storage::Manager, node: NodeId) -> Self {
        Self {
            storage,
            node,
            now: Timestamp::now_utc,
            active: Mutex::new(BTreeMap::new()),
        }
    }

    pub(crate) async fn recover<'a>(
        &self,
        completed: impl Fn(&CommandId) -> Option<&'a RunSummary>,
    ) -> Result<()> {
        for id in native::pending(&self.storage, &self.node).await? {
            self.get_recovered(&id, completed(&id)).await?;
        }
        Ok(())
    }

    pub(crate) async fn get(&self, id: &CommandId) -> Result<Option<Arc<RunReceipt>>> {
        self.get_recovered(id, None).await
    }

    async fn get_recovered(
        &self,
        id: &CommandId,
        completed: Option<&RunSummary>,
    ) -> Result<Option<Arc<RunReceipt>>> {
        let (mut active, stored) = self.lock_read(id).await?;
        if let Some(stored) = stored {
            let Some(receipt) = stored else {
                return Ok(None);
            };
            if receipt.summary.phase.terminal() {
                return Ok(Some(receipt));
            }
            let now = (self.now)();
            if let Some(summary) = completed
                && (summary.request_id != *id
                    || summary.cycle_id != receipt.summary.cycle_id
                    || !summary.phase.terminal()
                    || summary.phase == RunPhase::Interrupted
                    || summary.finished_at.is_none())
            {
                return Err(error(
                    "saved completion differs from receipt identity or phase",
                ));
            }
            let recovered = revise(&receipt, |receipt| {
                if let Some(summary) = completed {
                    receipt.summary = summary.clone();
                    receipt.interruption = None;
                    receipt.interrupt_step_commands(TERMINAL_INTERRUPTION, &now);
                } else {
                    interrupt(receipt, RECOVERY_INTERRUPTION, now);
                }
                Ok(())
            })?;
            active.insert(
                id.clone(),
                Active {
                    pending: Some(Pending::changed(recovered, &receipt)),
                    receipt,
                    admitted: false,
                },
            );
        }
        let owner = active.get_mut(id).expect("request owner exists");
        self.publish(owner).await?;
        if !owner.admitted && !owner.receipt.summary.phase.terminal() {
            let now = (self.now)();
            let interrupted = revise(&owner.receipt, |receipt| {
                interrupt(receipt, INITIAL_INTERRUPTION, now);
                Ok(())
            })?;
            owner.pending = Some(Pending::changed(interrupted, &owner.receipt));
            self.publish(owner).await?;
        }
        let receipt = owner.receipt.clone();
        if receipt.summary.phase.terminal() {
            active.remove(id);
        }
        Ok(Some(receipt))
    }

    pub(crate) async fn pending_command(
        &self,
        id: &CommandId,
    ) -> Result<Option<StepCommandReceipt>> {
        let mut active = self.active.lock().await;
        let owner = active
            .get_mut(id)
            .ok_or_else(|| error("request has no live command owner"))?;
        self.publish(owner).await?;
        if owner.receipt.summary.phase.terminal() {
            return Ok(None);
        }
        let mut pending = owner
            .receipt
            .commands
            .values()
            .filter(|command| !command.phase.terminal());
        let first = pending.next();
        if pending.next().is_some() {
            return Err(error("multiple unsettled commands"));
        }
        Ok(first.cloned())
    }

    pub(crate) async fn command(
        &self,
        id: &CommandId,
        command: &CommandId,
    ) -> Result<Option<StepCommandReceipt>> {
        let Some(receipt) = self.get(id).await? else {
            return Ok(None);
        };
        Ok(receipt.commands.get(command).cloned())
    }

    pub(crate) async fn begin(&self, receipt: RunReceipt) -> Result<()> {
        if receipt.node_id != self.node || receipt.summary.phase.terminal() {
            return Err(error(
                "new request must belong to this node and be nonterminal",
            ));
        }
        let receipt = Arc::new(receipt);
        let id = &receipt.summary.request_id;
        let (mut active, stored) = self.lock_read(id).await?;
        if stored.is_none_or(|stored| stored.is_some()) {
            return Err(error("request identity already exists"));
        }
        active.insert(
            id.clone(),
            Active {
                pending: Some(Pending::new(receipt.clone())),
                receipt: receipt.clone(),
                admitted: false,
            },
        );
        let owner = active.get_mut(id).expect("request owner inserted");
        self.publish(owner).await?;
        owner.admitted = true;
        Ok(())
    }

    pub(crate) async fn update(
        &self,
        id: &CommandId,
        change: impl FnOnce(&mut RunReceipt),
    ) -> Result<()> {
        let mut active = self.active.lock().await;
        let Some(owner) = active.get_mut(id) else {
            if self
                .read(id)
                .await?
                .is_some_and(|receipt| receipt.summary.phase.terminal())
            {
                return Ok(());
            }
            return Err(error("request has no active owner"));
        };
        self.publish(owner).await?;
        if !owner.admitted {
            return Err(error("request was not admitted"));
        }
        if !owner.receipt.summary.phase.terminal() {
            let now = (self.now)();
            let receipt = revise(&owner.receipt, |receipt| {
                change(receipt);
                if receipt.summary.request_id != *id
                    || receipt.node_id != self.node
                    || receipt.fingerprint != owner.receipt.fingerprint
                {
                    return Err(error("request identity cannot change"));
                }
                if receipt.summary.phase.terminal() {
                    receipt.interrupt_step_commands(TERMINAL_INTERRUPTION, &now);
                }
                Ok(())
            })?;
            if *receipt == *owner.receipt {
                return Ok(());
            }
            owner.pending = Some(Pending::changed(receipt, &owner.receipt));
            self.publish(owner).await?;
        }
        if owner.receipt.summary.phase.terminal() {
            active.remove(id);
        }
        Ok(())
    }

    /// Reads Storage Without Holding The Request Registry, Then Rechecks Ownership.
    async fn lock_read(
        &self,
        id: &CommandId,
    ) -> Result<(
        tokio::sync::MutexGuard<'_, BTreeMap<CommandId, Active>>,
        Option<Option<Arc<RunReceipt>>>,
    )> {
        let active = self.active.lock().await;
        if active.contains_key(id) {
            return Ok((active, None));
        }
        drop(active);
        let stored = self.read(id).await;
        let active = self.active.lock().await;
        if active.contains_key(id) {
            return Ok((active, None));
        }
        Ok((active, Some(stored?)))
    }

    async fn read(&self, id: &CommandId) -> Result<Option<Arc<RunReceipt>>> {
        let receipt = native::read(&self.storage, id).await?.map(Arc::new);
        if let Some(receipt) = &receipt
            && (receipt.summary.request_id != *id || receipt.node_id != self.node)
        {
            return Err(error("stored request identity or owner differs"));
        }
        Ok(receipt)
    }

    async fn publish(&self, owner: &mut Active) -> Result<()> {
        let Some(pending) = owner.pending.as_mut() else {
            return Ok(());
        };
        native::save(&self.storage, pending).await?;
        owner.receipt = owner.pending.take().expect("saved receipt").receipt;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn revise(
    receipt: &Arc<RunReceipt>,
    change: impl FnOnce(&mut RunReceipt) -> Result<()>,
) -> Result<Arc<RunReceipt>> {
    let mut revised = (**receipt).clone();
    change(&mut revised)?;
    Ok(Arc::new(revised))
}

fn interrupt(receipt: &mut RunReceipt, reason: &str, now: Timestamp) {
    receipt.summary.phase = RunPhase::Interrupted;
    receipt.interrupt_step_commands(reason, &now);
    receipt.summary.finished_at = Some(now);
    receipt.interruption = Some(reason.into());
}

fn receipt_error(source: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Error {
    Error::RequestReceipt(source.into())
}

fn error(value: impl std::fmt::Display) -> Error {
    Error::Operation(format!("execution request receipt: {value}"))
}

/* === Tests === */
