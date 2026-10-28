/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod native;

pub use dokime_domain::model::execution::request::AttemptClaim;
pub use dokime_domain::model::execution::request::RunReceipt;

use std::collections::BTreeMap;

use harmos::storage::StorageCursor;
use tokio::sync::Mutex;

use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::request::RunPhase;
use dokime_domain::model::execution::request::RunSummary;
use dokime_domain::model::execution::request::StepCommandReceipt;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::NodeId;
use dokime_domain::model::time::Timestamp;

const RECOVERY_INTERRUPTION: &str =
    "Previous runtime ended before request completion. Device actions are not replayed; claimed acquisition may be interrupted.";
const INITIAL_INTERRUPTION: &str = "Initial receipt publication was interrupted; no provider work was submitted.";
const TERMINAL_INTERRUPTION: &str = "Run settled before command completion was recorded.";

#[derive(Clone)]
pub struct Receipt {
    value: harmos::storage::ReadBatch<RunReceipt>,
}

#[derive(Clone)]
pub struct StepReceipt {
    receipt: Receipt,
    index: usize,
}

struct Active {
    receipt: Receipt,
    pending: Option<Pending<Receipt>>,
    admitted: bool,
}

struct ActiveRequests {
    entries: BTreeMap<CommandId, Active>,
}

pub(crate) struct Pending<R> {
    receipt: R,
    previous: Option<R>,
    position: Option<StorageCursor>,
}

pub(crate) struct RequestStorage {
    storage: crate::storage::Manager,
    reads: crate::storage::Manager,
    node: NodeId,
    now: fn() -> Timestamp,
    active: Mutex<ActiveRequests>,
}

impl std::ops::Deref for Receipt {
    type Target = RunReceipt;

    fn deref(&self) -> &RunReceipt {
        &self.value
    }
}

impl AsRef<RunReceipt> for Receipt {
    fn as_ref(&self) -> &RunReceipt {
        self
    }
}

impl std::ops::Deref for StepReceipt {
    type Target = StepCommandReceipt;

    fn deref(&self) -> &StepCommandReceipt {
        self.receipt.commands.values().nth(self.index).expect("immutable command selection")
    }
}

impl AsRef<StepCommandReceipt> for StepReceipt {
    fn as_ref(&self) -> &StepCommandReceipt {
        self
    }
}

impl Receipt {
    pub fn command(&self, id: &CommandId) -> Option<StepReceipt> {
        let index = self.commands.keys().position(|key| key == id)?;
        Some(StepReceipt { receipt: self.clone(), index })
    }

    fn new(receipt: RunReceipt) -> Result<Self> {
        Ok(Self {
            value: harmos::storage::ReadBatch::new(receipt),
        })
    }
}

impl<R: Clone + std::ops::Deref<Target = RunReceipt>> Pending<R> {
    fn new(receipt: R) -> Self {
        Self { receipt, previous: None, position: None }
    }

    fn changed(receipt: R, previous: &R) -> Self {
        Self {
            receipt,
            previous: Some(previous.clone()),
            position: None,
        }
    }

    fn changed_commands(&self) -> impl Iterator<Item = (&CommandId, &StepCommandReceipt)> {
        self.receipt
            .commands
            .iter()
            .filter(|(id, command)| self.previous.as_ref().is_none_or(|previous| previous.commands.get(*id) != Some(*command)))
    }
}

impl ActiveRequests {
    fn new() -> Self {
        Self { entries: BTreeMap::new() }
    }

    fn insert(&mut self, id: &CommandId, value: Active) -> Result<()> {
        if self.entries.contains_key(id) {
            return Err(error("request already has an active owner"));
        }
        self.entries.insert(id.clone(), value);
        Ok(())
    }

    fn remove(&mut self, id: &CommandId) {
        self.entries.remove(id);
    }
}

impl RequestStorage {
    pub(crate) fn new(storage: crate::storage::Manager, node: &NodeId) -> Result<Self> {
        let reads = storage.clone();
        Ok(Self {
            storage,
            reads,
            node: node.clone(),
            now: crate::model::time::now_utc,
            active: Mutex::new(ActiveRequests::new()),
        })
    }

    pub(crate) async fn recover<'a>(&self, completed: impl Fn(&CommandId) -> Option<&'a RunSummary>) -> Result<()> {
        let session = self.reads.clone();
        let ids = native::pending(&self.storage, &session, &self.node).await?;
        for id in ids.iter() {
            self.get_recovered(&session, id, completed(id), true).await?;
        }
        Ok(())
    }

    pub(crate) async fn get(&self, id: &CommandId) -> Result<Option<Receipt>> {
        self.get_recovered(&self.reads, id, None, false).await
    }

    async fn get_recovered(
        &self, session: &crate::storage::Manager, id: &CommandId, completed: Option<&RunSummary>, boot: bool,
    ) -> Result<Option<Receipt>> {
        let (mut active, stored) = self.lock_read(session, id).await?;
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
                return Err(error("saved completion differs from receipt identity or phase"));
            }
            let recovered = revise(&receipt, |receipt| {
                if let Some(summary) = completed {
                    receipt.summary = summary.clone();
                    receipt.interruption = None;
                    receipt.interrupt_step_commands(TERMINAL_INTERRUPTION, &now);
                } else if boot && receipt.summary.provider_started_at.is_none() && receipt.summary.prepared == 0 {
                    // Never dispatched: the stop that ended the session stopped it before it began.
                    receipt.summary.phase = RunPhase::Stopped;
                    receipt.summary.finished_at = Some(now);
                } else {
                    interrupt(receipt, RECOVERY_INTERRUPTION, now);
                }
                Ok(())
            })?;
            active.insert(
                id,
                Active {
                    pending: Some(Pending::changed(recovered, &receipt)),
                    receipt,
                    admitted: false,
                },
            )?;
        }
        let owner = active.entries.get_mut(id).expect("request owner exists");
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

    pub(crate) async fn pending_command(&self, id: &CommandId) -> Result<Option<StepReceipt>> {
        let mut active = self.active.lock().await;
        let owner = active.entries.get_mut(id).ok_or_else(|| error("request has no live command owner"))?;
        self.publish(owner).await?;
        if owner.receipt.summary.phase.terminal() {
            return Ok(None);
        }
        let mut pending = owner.receipt.commands.values().enumerate().filter(|(_, command)| !command.phase.terminal());
        let first = pending.next().map(|(index, _)| index);
        if pending.next().is_some() {
            return Err(error("multiple unsettled commands"));
        }
        Ok(first.map(|index| StepReceipt { receipt: owner.receipt.clone(), index }))
    }

    pub(crate) async fn command(&self, id: &CommandId, command: &CommandId) -> Result<Option<StepReceipt>> {
        let Some(receipt) = self.get(id).await? else {
            return Ok(None);
        };
        Ok(receipt.command(command))
    }

    pub(crate) async fn begin(&self, receipt: RunReceipt) -> Result<()> {
        if receipt.node_id != self.node || receipt.summary.phase.terminal() {
            return Err(error("new request must belong to this node and be nonterminal"));
        }
        let session = self.reads.clone();
        let receipt = Receipt::new(receipt)?;
        let id = &receipt.summary.request_id;
        let (mut active, stored) = self.lock_read(&session, id).await?;
        if stored.is_none_or(|stored| stored.is_some()) {
            return Err(error("request identity already exists"));
        }
        active.insert(
            id,
            Active {
                pending: Some(Pending::new(receipt.clone())),
                receipt: receipt.clone(),
                admitted: false,
            },
        )?;
        let owner = active.entries.get_mut(id).expect("request owner inserted");
        self.publish(owner).await?;
        owner.admitted = true;
        Ok(())
    }

    pub(crate) async fn update(&self, id: &CommandId, change: impl FnOnce(&mut RunReceipt)) -> Result<()> {
        let mut active = self.active.lock().await;
        let Some(owner) = active.entries.get_mut(id) else {
            if self.read(&self.reads, id).await?.is_some_and(|receipt| receipt.summary.phase.terminal()) {
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
                if receipt.summary.request_id != *id || receipt.node_id != self.node || receipt.fingerprint != owner.receipt.fingerprint {
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

    async fn lock_read(
        &self, session: &crate::storage::Manager, id: &CommandId,
    ) -> Result<(tokio::sync::MutexGuard<'_, ActiveRequests>, Option<Option<Receipt>>)> {
        let active = self.active.lock().await;
        if active.entries.contains_key(id) {
            return Ok((active, None));
        }
        drop(active);
        let stored = self.read(session, id).await;
        let active = self.active.lock().await;
        if active.entries.contains_key(id) {
            return Ok((active, None));
        }
        Ok((active, Some(stored?)))
    }

    async fn read(&self, session: &crate::storage::Manager, id: &CommandId) -> Result<Option<Receipt>> {
        let receipt = native::read(&self.storage, session, id).await?.map(|value| Receipt { value });
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

fn revise(receipt: &Receipt, change: impl FnOnce(&mut RunReceipt) -> Result<()>) -> Result<Receipt> {
    let seed = harmos::storage::ReadBatch::new(());
    let value = seed.try_map(|_| -> Result<_> {
        let mut revised = (**receipt).clone();
        change(&mut revised)?;
        Ok(revised)
    })?;
    Ok(Receipt { value })
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
