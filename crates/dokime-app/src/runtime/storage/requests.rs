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
use std::{collections::BTreeMap, ops::Deref};
// Third Party Imports
use harmos::{
    MemorySize,
    storage::{SharedQueryValue, StorageCursor},
};
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
pub type ReceiptLoan = SharedQueryValue<RunReceipt>;

/* === Structs === */
struct Active<S: ReceiptStore> {
    receipt: S::Receipt,
    pending: Option<Pending<S::Receipt>>,
    admitted: bool,
    // Keeps This Entry's Original Node And Key Preclaim Until Removal.
    _registry: S::Registry,
}

pub(crate) struct Pending<R> {
    receipt: R,
    previous: Option<R>,
    position: Option<StorageCursor>,
}

pub(crate) struct RequestStorage<S: ReceiptStore = native::Receipts> {
    store: S,
    node: NodeId,
    now: fn() -> Timestamp,
    /// How long a receipt read backs off while other readers fill the read
    /// ledger: the interactive deadline.
    patience: std::time::Duration,
    active: Mutex<BTreeMap<CommandId, Active<S>>>,
}

/* === Enums === */

/* === Traits === */
pub(crate) trait ReceiptStore: Send + Sync {
    type Receipt: Clone + Deref<Target = RunReceipt> + Send + Sync;
    type Keys: Deref<Target = Vec<CommandId>> + Send;
    type Command: Deref<Target = StepCommandReceipt> + Send;
    type Registry: Send + Sync;

    fn retain(&self, receipt: RunReceipt) -> Result<Self::Receipt>;
    fn reserve_entry(&self, id: &CommandId) -> Result<Self::Registry>;
    fn revise(
        &self,
        receipt: &Self::Receipt,
        growth: usize,
        change: impl FnOnce(&mut RunReceipt) -> Result<()>,
    ) -> Result<Self::Receipt>;
    fn command(&self, receipt: &Self::Receipt, id: &CommandId) -> Result<Self::Command>;
    fn read(
        &self,
        id: &CommandId,
    ) -> impl std::future::Future<Output = Result<Option<Self::Receipt>>> + Send;
    fn pending(
        &self,
        node: &NodeId,
        after: Option<&CommandId>,
    ) -> impl std::future::Future<Output = Result<Option<Self::Keys>>> + Send;
    fn save(
        &self,
        pending: &mut Pending<Self::Receipt>,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

/* === Implementations === */
impl<R: Clone + Deref<Target = RunReceipt>> Pending<R> {
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
    pub(crate) fn new(
        storage: dokime::storage::Manager,
        node: NodeId,
        patience: std::time::Duration,
    ) -> Self {
        Self::with_clock(
            native::Receipts::new(storage),
            node,
            Timestamp::now_utc,
            patience,
        )
    }
}

impl<S: ReceiptStore> RequestStorage<S> {
    fn with_clock(
        store: S,
        node: NodeId,
        now: fn() -> Timestamp,
        patience: std::time::Duration,
    ) -> Self {
        Self {
            store,
            node,
            now,
            patience,
            active: Mutex::new(BTreeMap::new()),
        }
    }

    pub(crate) async fn recover<'a>(
        &self,
        completed: impl Fn(&CommandId) -> Option<&'a RunSummary>,
    ) -> Result<()> {
        let mut previous: Option<S::Keys> = None;
        loop {
            let after = previous.as_ref().and_then(|page| page.last());
            let Some(page) = self.store.pending(&self.node, after).await? else {
                return Ok(());
            };
            if page.is_empty() {
                return Ok(());
            }
            for id in page.iter() {
                if after.is_some_and(|after| after >= id) {
                    return Err(error("request recovery page did not advance"));
                }
                self.get_recovered(id, completed(id)).await?;
            }
            previous = Some(page);
        }
    }

    pub(crate) async fn get(&self, id: &CommandId) -> Result<Option<S::Receipt>> {
        self.get_recovered(id, None).await
    }

    async fn get_recovered(
        &self,
        id: &CommandId,
        completed: Option<&RunSummary>,
    ) -> Result<Option<S::Receipt>> {
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
            let growth = interruption_memory(&receipt, RECOVERY_INTERRUPTION, &now)?
                .saturating_add(completed.map_or(0, MemorySize::memory_size));
            let recovered = self.store.revise(&receipt, growth, |receipt| {
                if let Some(summary) = completed {
                    receipt.summary = summary.clone();
                    receipt.interruption = None;
                    receipt.interrupt_step_commands(TERMINAL_INTERRUPTION, &now);
                } else {
                    interrupt(receipt, RECOVERY_INTERRUPTION, now);
                }
                Ok(())
            })?;
            let registry = self.store.reserve_entry(id)?;
            active.insert(
                id.clone(),
                Active {
                    pending: Some(Pending::changed(recovered, &receipt)),
                    receipt,
                    admitted: false,
                    _registry: registry,
                },
            );
        }
        let owner = active.get_mut(id).expect("request owner exists");
        self.publish(owner).await?;
        if !owner.admitted && !owner.receipt.summary.phase.terminal() {
            let now = (self.now)();
            let interrupted = self.store.revise(
                &owner.receipt,
                interruption_memory(&owner.receipt, INITIAL_INTERRUPTION, &now)?,
                |receipt| {
                    interrupt(receipt, INITIAL_INTERRUPTION, now);
                    Ok(())
                },
            )?;
            owner.pending = Some(Pending::changed(interrupted, &owner.receipt));
            self.publish(owner).await?;
        }
        let receipt = owner.receipt.clone();
        if receipt.summary.phase.terminal() {
            active.remove(id);
        }
        Ok(Some(receipt))
    }

    pub(crate) async fn pending_command(&self, id: &CommandId) -> Result<Option<S::Command>> {
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
        first
            .map(|command| self.store.command(&owner.receipt, &command.command_id))
            .transpose()
    }

    pub(crate) async fn command(
        &self,
        id: &CommandId,
        command: &CommandId,
    ) -> Result<Option<S::Command>> {
        let Some(receipt) = self.get(id).await? else {
            return Ok(None);
        };
        if !receipt.commands.contains_key(command) {
            return Ok(None);
        }
        self.store.command(&receipt, command).map(Some)
    }

    pub(crate) async fn begin(&self, receipt: RunReceipt) -> Result<()> {
        if receipt.node_id != self.node || receipt.summary.phase.terminal() {
            return Err(error(
                "new request must belong to this node and be nonterminal",
            ));
        }
        let receipt = self.store.retain(receipt)?;
        let id = &receipt.summary.request_id;
        let (mut active, stored) = self.lock_read(id).await?;
        if stored.is_none_or(|stored| stored.is_some()) {
            return Err(error("request identity already exists"));
        }
        let registry = self.store.reserve_entry(id)?;
        active.insert(
            id.clone(),
            Active {
                pending: Some(Pending::new(receipt.clone())),
                receipt: receipt.clone(),
                admitted: false,
                _registry: registry,
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
        growth: usize,
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
            let terminal_growth = interruption_memory(&owner.receipt, TERMINAL_INTERRUPTION, &now)?;
            let growth = growth
                .checked_add(terminal_growth)
                .ok_or_else(|| error("receipt update size overflow"))?;
            let receipt = self.store.revise(&owner.receipt, growth, |receipt| {
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
            if receipt.deref() == owner.receipt.deref() {
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

    /// Locks the request owners, reading `id`'s stored receipt first when no
    /// owner holds it; `None` means an owner does. A receipt is small, so a
    /// full read ledger is other readers: the read backs off for them with
    /// the lock released, so receipt updates never wait behind it.
    async fn lock_read(
        &self,
        id: &CommandId,
    ) -> Result<(
        tokio::sync::MutexGuard<'_, BTreeMap<CommandId, Active<S>>>,
        Option<Option<S::Receipt>>,
    )> {
        let deadline = tokio::time::Instant::now() + self.patience;
        let mut backoff = super::record_capture::CapacityBackoff::new();
        loop {
            let active = self.active.lock().await;
            if active.contains_key(id) {
                return Ok((active, None));
            }
            match self.read(id).await {
                Err(error) if super::record_capture::read_capacity(&error) => {
                    drop(active);
                    if !backoff.wait_until(deadline).await {
                        return Err(error);
                    }
                }
                stored => return Ok((active, Some(stored?))),
            }
        }
    }

    async fn read(&self, id: &CommandId) -> Result<Option<S::Receipt>> {
        let receipt = self.store.read(id).await?;
        if let Some(receipt) = &receipt
            && (receipt.summary.request_id != *id || receipt.node_id != self.node)
        {
            return Err(error("stored request identity or owner differs"));
        }
        Ok(receipt)
    }

    async fn publish(&self, owner: &mut Active<S>) -> Result<()> {
        let Some(pending) = owner.pending.as_mut() else {
            return Ok(());
        };
        self.store.save(pending).await?;
        owner.receipt = owner.pending.take().expect("saved receipt").receipt;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn interruption_memory(receipt: &RunReceipt, reason: &str, now: &Timestamp) -> Result<usize> {
    let count = receipt
        .commands
        .values()
        .filter(|command| !command.phase.terminal())
        .count();
    let per_command = reason
        .len()
        .checked_add(now.memory_size())
        .ok_or_else(|| error("interruption size overflow"))?;
    count
        .checked_mul(per_command)
        .and_then(|bytes| bytes.checked_add(per_command))
        .ok_or_else(|| error("interruption size overflow"))
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
