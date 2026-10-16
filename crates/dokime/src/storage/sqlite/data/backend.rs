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
use std::{
    num::NonZeroUsize,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
// Third Party Imports
use harmos::storage::{ObjectReleaseCursor, ObjectReleasePage, ObjectRetentionRelease};
use harmos::{
    BackendFailure, MemorySize,
    catalog::Projection,
    storage::frame::{Scan, ScanCursor, ScanOrder, ScanPage},
    storage::*,
};
use tokio::sync::{Mutex, Notify};
// Local Imports
use super::super::{Error, State, memory, query};
use super::{Entry, SavedData};

/* === Constants === */

/* === Macros === */
macro_rules! read_entries {
    ($method:ident, $value:ty, $variant:ident, $kind:ident) => {
        fn $method<'a>(
            &'a self,
            view: &'a ReadView,
            selection: &'a ReadSelection,
            cursor: Option<&'a ReadCursor>,
            limits: ReadLimits,
        ) -> StorageFuture<'a, ReadPage<$value>> {
            Box::pin(async move {
                view.check_kind(ReadKind::$kind, selection, cursor)?;
                self.page(view, selection, cursor, limits, |entry| match entry {
                    Entry::$variant(value) => Ok(value),
                    _ => Err(invalid("stored collection differs from its index")),
                })
                .await
            })
        }
    };
}

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy)]
pub struct Limits {
    pub entry_bytes: NonZeroUsize,
    pub batch_bytes: NonZeroUsize,
    pub views: NonZeroUsize,
    pub memory_bytes: NonZeroUsize,
}

pub struct Backend<C: ObjectCatalog> {
    catalog: std::marker::PhantomData<fn() -> C>,
    root: PathBuf,
    history: StoreHistory,
    limits: Limits,
    owner: Arc<Mutex<Owner>>,
}

struct Owner {
    state: Option<State>,
    views: Box<[Option<Slot>]>,
    registry_memory: Option<memory::Reservation>,
    primary_active: bool,
    bootstrap_limits: Option<QueryLimits>,
    retention_through: u64,
}

struct Slot {
    token: [u8; 16],
    pin: Arc<Pin>,
    protection: Option<Arc<super::super::DataPin>>,
}

struct ActivePin {
    pin: Arc<Pin>,
    protection: Option<Arc<super::super::DataPin>>,
    computation: Option<StorageViewComputation>,
}

struct OwnedRead<T> {
    result: Result<T, Error>,
    _pin: ActivePin,
}

struct Pin {
    cut: SavedData,
    released: AtomicBool,
    active_reads: AtomicUsize,
    settled: Notify,
    execution: Option<Arc<query::Budget>>,
    runs: super::scan::Runs,
    _memory: memory::Reservation,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl std::ops::Deref for ActivePin {
    type Target = Pin;

    fn deref(&self) -> &Self::Target {
        &self.pin
    }
}

impl Drop for ActivePin {
    fn drop(&mut self) {
        // Release Physical Protection Before Acknowledging Read Settlement.
        drop(self.protection.take());
        drop(self.computation.take());
        if self.pin.active_reads.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.pin.settled.notify_waiters();
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            entry_bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
            batch_bytes: NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
            views: NonZeroUsize::new(64).unwrap(),
            memory_bytes: NonZeroUsize::new(32 * 1024 * 1024).unwrap(),
        }
    }
}

impl Pin {
    fn prepare<T>(&self, operation: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
        match &self.execution {
            Some(execution) => execution.prepare(operation),
            None => operation(),
        }
    }
}

impl Owner {
    fn state(&self) -> harmos::Result<State> {
        if !self.primary_active {
            return Err(invalid("SQLite primary backend is closed"));
        }
        self.state
            .clone()
            .ok_or_else(|| invalid("SQLite backend is closed"))
    }
    async fn close_if_idle(&mut self) -> Result<(), Error> {
        if !self.primary_active {
            if let Some(state) = self.state.as_ref() {
                state.close().await?;
            }
            self.state = None;
            for slot in &mut self.views {
                *slot = None;
            }
            self.registry_memory = None;
            self.bootstrap_limits = None;
            self.retention_through = 0;
        }
        Ok(())
    }
    fn pins(&self) -> impl Iterator<Item = &Arc<Pin>> {
        self.views
            .iter()
            .filter_map(|slot| slot.as_ref().map(|slot| &slot.pin))
    }
    fn sweep(&mut self) {
        for slot in &mut self.views {
            if slot.as_ref().is_some_and(|slot| {
                slot.pin.released.load(Ordering::Acquire)
                    && slot.pin.active_reads.load(Ordering::Acquire) == 0
            }) {
                *slot = None;
            }
        }
    }
}

impl<C: ObjectCatalog> Backend<C> {
    pub async fn primary(root: PathBuf, limits: Limits) -> harmos::Result<Self> {
        Self::initialize(root, limits, None).await
    }

    pub async fn replica(
        root: PathBuf,
        limits: Limits,
        history: StoreHistory,
    ) -> harmos::Result<Self> {
        Self::initialize(root, limits, Some(history)).await
    }

    async fn initialize(
        root: PathBuf,
        limits: Limits,
        history: Option<StoreHistory>,
    ) -> harmos::Result<Self> {
        C::validate()?;
        if limits.entry_bytes > limits.batch_bytes {
            return Err(invalid("entry bound exceeds batch bound"));
        }
        let state = State::open_with_memory(&root, limits.memory_bytes)
            .await
            .map_err(|error| failure("open SQLite", error))?;
        let cut = state
            .data_open(history)
            .await
            .map_err(|error| failure("initialize SQLite", error))?;
        let bootstrap_limits = state
            .data_bootstrap_limits()
            .await
            .map_err(|error| failure("restore private import policy", error))?;
        let root = root.canonicalize().map_err(|error| {
            harmos::Error::backend("resolve SQLite root", BackendFailure::Permanent, error)
        })?;
        let registry_bytes = limits
            .views
            .get()
            .checked_mul(std::mem::size_of::<Option<Slot>>())
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<Owner>() + 2 * std::mem::size_of::<usize>())
            })
            .ok_or_else(|| invalid("SQLite view registry size overflow"))?;
        let registry_memory = state
            .reserve_query(registry_bytes, limits.memory_bytes)
            .map_err(|error| failure("reserve SQLite view registry", error))?;
        let mut views = Box::<[Option<Slot>]>::new_uninit_slice(limits.views.get());
        for slot in &mut views {
            slot.write(None);
        }
        // SAFETY: Every slot is initialized to None before assuming the slice.
        let views = unsafe { views.assume_init() };
        Ok(Self {
            catalog: std::marker::PhantomData,
            root,
            history: cut.history,
            limits,
            owner: Arc::new(Mutex::new(Owner {
                state: Some(state),
                views,
                registry_memory: Some(registry_memory),
                primary_active: true,
                bootstrap_limits,
                retention_through: 0,
            })),
        })
    }

    pub async fn control(&self) -> harmos::Result<State> {
        self.owner.lock().await.state()
    }

    async fn view(
        &self,
        through: &StorageCursor,
        limits: Option<QueryLimits>,
        retained: bool,
    ) -> harmos::Result<(BackendSnapshot, SavedData)> {
        self.history.check(&through.history)?;
        let mut owner = self.owner.lock().await;
        owner.sweep();
        if owner.pins().count() >= self.limits.views.get() {
            return Err(invalid("SQLite view capacity exhausted"));
        }
        let state = owner.state()?;
        let ceiling = limits.map_or(self.limits.memory_bytes, |limits| limits.memory_bytes);
        // History is capped at HISTORY_BYTES before decoding. Pin and query
        // Arc headers are charged before allocating any retained view state.
        let pin_bytes = std::mem::size_of::<Pin>()
            + 2 * std::mem::size_of::<usize>()
            + 4 * super::HISTORY_BYTES
            + 2 * self.root.as_os_str().len()
            + std::mem::size_of::<query::Budget>()
            + 4 * std::mem::size_of::<usize>()
            + std::mem::size_of::<super::super::DataPin>();
        let reservation = state
            .reserve_query(pin_bytes, ceiling)
            .map_err(|error| failure("reserve SQLite view state", error))?;
        let execution = limits
            .map(query::Budget::new)
            .transpose()
            .map_err(|error| failure("initialize SQLite query budget", error))?
            .map(Arc::new);
        let through = through.clone();
        let root = self.root.clone();
        let memory = state.0.memory.clone();
        let maximum = self.limits.entry_bytes;
        let pin_state = state.clone();
        let (cut, reservation, protection) = state
            .call_read(execution.clone(), move |connection| {
                Ok((|| {
                    let protection = pin_state.pin_data(connection)?;
                    let mut cut = super::saved(connection)?;
                    if through.sequence > cut.sequence {
                        return Err(Error::Data("view exceeds saved frontier"));
                    }
                    if retained {
                        // Verify The Owner Before Restricting Its Requested Cut
                        // Later saved admissions remain legitimate owner history.
                        super::inventory::verify(connection, &cut)?;
                    }
                    cut.sequence = through.sequence;
                    if retained {
                        super::retained::verify(connection, &root, &cut, &memory, maximum)?;
                    }
                    Ok((cut, reservation, protection))
                })())
            })
            .await
            .map_err(|error| failure("pin SQLite view", error))?
            .map_err(|error| failure("verify SQLite view", error))?;
        let prepare = || {
            let token = *uuid::Uuid::now_v7().as_bytes();
            let snapshot = BackendSnapshot {
                saved: StorageCursor {
                    history: cut.history.clone(),
                    sequence: cut.sequence,
                },
                token: token.to_vec(),
            };
            let pin = Arc::new(Pin {
                cut: cut.clone(),
                released: AtomicBool::new(false),
                active_reads: AtomicUsize::new(0),
                settled: Notify::new(),
                execution: execution.clone(),
                runs: super::scan::Runs::default(),
                _memory: reservation,
            });
            Ok((token, snapshot, pin, Arc::new(protection)))
        };
        let (token, snapshot, pin, protection) = match &execution {
            Some(execution) => execution.prepare(prepare),
            None => prepare(),
        }
        .map_err(|error| failure("prepare SQLite view registration", error))?;
        // No Suspension After Registration
        let slot = owner
            .views
            .iter_mut()
            .find(|slot| slot.is_none())
            .expect("checked SQLite view capacity");
        *slot = Some(Slot {
            token,
            pin,
            protection: Some(protection),
        });
        Ok((snapshot, cut))
    }

    async fn pin(&self, view: &ReadView) -> harmos::Result<(State, ActivePin)> {
        self.history.check(&view.saved().history)?;
        let computation = view.retain_computation()?;
        let owner = self.owner.lock().await;
        let slot = owner
            .views
            .iter()
            .filter_map(Option::as_ref)
            .find(|slot| slot.token.as_slice() == view.snapshot().token)
            .filter(|slot| {
                let pin = &slot.pin;
                !pin.released.load(Ordering::Acquire)
                    && pin.cut.sequence == view.saved().sequence
                    && pin.cut.history == view.saved().history
                    && pin.execution.as_ref().map(|query| query.limits) == view.execution_limits()
            })
            .ok_or(harmos::Error::ViewExpired)?;
        let state = owner.state()?;
        let protection = slot
            .protection
            .as_ref()
            .ok_or(harmos::Error::ViewExpired)?
            .clone();
        slot.pin
            .active_reads
            .try_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                active.checked_add(1)
            })
            .map_err(|_| harmos::Error::ResourceExhausted {
                resource: "SQLite active view reads",
            })?;
        let pin = ActivePin {
            pin: slot.pin.clone(),
            protection: Some(protection),
            computation,
        };
        Ok((state, pin))
    }

    async fn page<T: Send + 'static>(
        &self,
        view: &ReadView,
        selection: &ReadSelection,
        cursor: Option<&ReadCursor>,
        limits: ReadLimits,
        convert: fn(Entry) -> harmos::Result<T>,
    ) -> harmos::Result<ReadPage<T>> {
        let (state, pin) = self.pin(view).await?;
        let (read, framing, previous, view) = pin
            .prepare(|| {
                let request = selection.clone();
                let framing = request.clone();
                let previous = cursor.cloned();
                let view = view.clone();
                let after = cursor.map(|cursor| cursor.position().sequence);
                let read = super::PageRead {
                    cut: pin.cut.clone(),
                    selection: request,
                    after,
                    limits,
                    entry_bytes: self.limits.entry_bytes,
                };
                Ok((read, framing, previous, view))
            })
            .map_err(|error| failure("prepare SQLite page", error))?;
        // The physical reader retains the pin through cursor
        // framing and typed conversion, including cancellation and failure.
        owned_read("read SQLite page", async move {
            let result = state
                .data_page(read, pin.execution.clone(), move |page| {
                    let next = if page.has_more {
                        Some(ReadCursor::new(
                            &view,
                            framing.clone(),
                            page.entries
                                .last()
                                .ok_or_else(|| invalid("empty SQLite continuation"))?
                                .position()
                                .clone(),
                        )?)
                    } else {
                        None
                    };
                    let framed = ReadPage {
                        entries: page.entries,
                        next,
                    };
                    framed.check_positions(&view, &framing, previous.as_ref(), Entry::position)?;
                    let entries = framed
                        .entries
                        .into_iter()
                        .map(convert)
                        .collect::<harmos::Result<Vec<_>>>()?;
                    Ok(ReadPage {
                        entries,
                        next: framed.next,
                    })
                })
                .await;
            Ok(OwnedRead { result, _pin: pin })
        })
        .await
    }
}

impl<C: ObjectCatalog> StorageBackend for Backend<C> {
    type Catalog = C;

    fn history(&self) -> &StoreHistory {
        &self.history
    }
    fn validate_query_limits(&self, limits: QueryLimits) -> harmos::Result<()> {
        limits.check()
    }
    fn validate_object_versions(&self, _versions: ObjectVersions) -> harmos::Result<()> {
        Ok(())
    }

    fn validate_object_browse(
        &self,
        request: &ObjectBrowseRequest,
        limits: QueryLimits,
    ) -> harmos::Result<()> {
        request.check()?;
        let mut found = false;
        C::definitions().try_for_each(|definition| {
            if Some(&definition.schema()?) == request.selection.schema.as_ref() {
                let schema = definition.graph(&harmos::config::SchemaSettings::default())?;
                let root = schema.root().map_err(|error| {
                    harmos::Error::backend(
                        "validate catalog scalar schema",
                        harmos::BackendFailure::Permanent,
                        error,
                    )
                })?;
                request.check_schema(root.fields())?;
                found = true;
            }
            Ok::<(), harmos::Error>(())
        })?;
        if !found {
            return Err(invalid("scalar schema is absent from the object catalog"));
        }
        self.validate_query_limits(limits)
    }

    fn read_object_browse<'a>(
        &'a self,
        view: &'a ReadView,
        request: &'a ObjectBrowseRequest,
        cursor: Option<&'a ObjectBrowseCursor>,
        limits: ReadLimits,
    ) -> StorageFuture<'a, ObjectBrowsePage> {
        Box::pin(async move {
            request.check_view(view, cursor)?;
            let execution = view
                .execution_limits()
                .ok_or_else(|| invalid("scalar query requires execution limits"))?;
            let (state, pin) = self.pin(view).await?;
            let (cut, read) = pin
                .prepare(|| {
                    self.validate_object_browse(request, execution)?;
                    let read = super::browse::Read {
                        view: view.clone(),
                        request: request.clone(),
                        cursor: cursor.cloned(),
                        limits,
                        maximum: self.limits.entry_bytes,
                    };
                    Ok((pin.cut.clone(), read))
                })
                .map_err(|error| failure("prepare scalar query", error))?;
            let (result, _pin) = owned("read SQLite scalar metadata", async move {
                let result = state.data_browse(cut, read, pin.execution.clone()).await;
                Ok((result, pin))
            })
            .await?;
            result.map_err(|error| failure("read SQLite scalar metadata", error))
        })
    }

    fn supports_scan_order(&self, scan: &Scan, order: &ScanOrder) -> bool {
        super::scan::admission(scan, order).is_ok()
    }

    fn scan_ordered<'a>(
        &'a self,
        view: &'a ReadView,
        scan: &'a Scan,
        order: &'a ScanOrder,
        cursor: Option<&'a ScanCursor>,
        limits: ReadLimits,
    ) -> StorageFuture<'a, ScanPage> {
        Box::pin(async move {
            view.check_kind(scan.selection.kind, &scan.selection, None)?;
            super::scan::admission(scan, order)?;
            view.execution_limits()
                .ok_or_else(|| invalid("frame scans require execution limits"))?;
            let (state, pin) = self.pin(view).await?;
            let (cut, read) = pin
                .prepare(|| {
                    let read = super::scan::Read {
                        scan: scan.clone(),
                        order: order.clone(),
                        cursor: cursor.cloned(),
                        limits,
                        maximum: self.limits.entry_bytes,
                    };
                    Ok((pin.cut.clone(), read))
                })
                .map_err(|error| failure("prepare frame scan", error))?;
            let cancellation = query::Cancellation::new();
            let cancelled = cancellation.flag();
            let memory = state.0.memory.clone();
            let owner = state.0.clone();
            let execution = pin.execution.clone();
            // Original Future Drop Signals The Worker. Its Actual Pin And Charge
            // Stay In The Physical Closure And Result Mailbox Until Settlement.
            let settled = state
                .call_read(execution, move |connection| {
                    let _cancel = query::cancellation(cancelled);
                    let result = super::scan::page(
                        connection,
                        &memory,
                        &owner.scanned,
                        &cut,
                        &read,
                        &pin.runs,
                        pin.computation.as_ref(),
                    );
                    Ok(Ok(OwnedRead { result, _pin: pin }))
                })
                .await
                .map_err(|error| failure("read SQLite frame scan", error))?
                .map_err(|error| failure("read SQLite frame scan", error))?;
            let result = settled
                .result
                .map_err(|error| failure("read SQLite frame scan", error));
            drop(cancellation);
            result
        })
    }

    fn open(&self) -> StorageFuture<'_, StorageCursor> {
        Box::pin(async move {
            let mut owner = self.owner.clone().lock_owned().await;
            let root = self.root.clone();
            let expected = self.history.clone();
            let memory = self.limits.memory_bytes;
            owned("open SQLite", async move {
                if owner.state.is_none() {
                    let state = State::open_with_memory(&root, memory).await?;
                    tracing::debug!(store = %expected.store, epoch = %expected.epoch, "SQLite storage owner opened");
                    state.data_open(Some(expected)).await?;
                    owner.bootstrap_limits = state.data_bootstrap_limits().await?;
                    let bytes = owner
                        .views
                        .len()
                        .checked_mul(std::mem::size_of::<Option<Slot>>())
                        .and_then(|bytes| {
                            bytes.checked_add(
                                std::mem::size_of::<Owner>() + 2 * std::mem::size_of::<usize>(),
                            )
                        })
                        .ok_or(Error::Data("SQLite view registry size overflow"))?;
                    owner.registry_memory = Some(state.reserve_query(bytes, memory)?);
                    owner.state = Some(state);
                }
                let cut = owner
                    .state
                    .as_ref()
                    .expect("opened SQLite owner")
                    .data_saved()
                    .await?;
                owner.primary_active = true;
                Ok(StorageCursor {
                    history: cut.history,
                    sequence: cut.sequence,
                })
            })
            .await
        })
    }
    fn cursor_saved(&self) -> StorageFuture<'_, StorageCursor> {
        Box::pin(async move {
            let state = self.owner.lock().await.state()?;
            let cut = state
                .data_saved()
                .await
                .map_err(|error| failure("read SQLite saved frontier", error))?;
            Ok(StorageCursor {
                history: cut.history,
                sequence: cut.sequence,
            })
        })
    }
    fn begin_bootstrap<'a>(
        &'a self,
        identity: &'a BootstrapId,
        saved: &'a StorageCursor,
        limits: QueryLimits,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.validate_query_limits(limits)?;
            self.history.check(&identity.cut().history)?;
            if !identity.cut().covers(saved)? {
                return Err(invalid(
                    "private import cut precedes its prior saved cursor",
                ));
            }
            let mut owner = self.owner.clone().lock_owned().await;
            let state = owner.state()?;
            let execution = Arc::new(
                query::Budget::new(limits)
                    .map_err(|error| failure("initialize private import budget", error))?,
            );
            let (reservation, import) = execution
                .prepare(|| {
                    super::entry::check_size(identity, super::bootstrap::METADATA_BYTES)?;
                    super::entry::check_size(saved, super::bootstrap::METADATA_BYTES)?;
                    let reservation = state
                        .reserve_query(super::bootstrap::WORKING_BYTES, limits.memory_bytes)?;
                    let import = super::bootstrap::Import {
                        identity: identity.clone(),
                        saved: saved.clone(),
                        limits,
                    };
                    Ok((reservation, import))
                })
                .map_err(|error| failure("prepare private replica import", error))?;
            owned("begin private replica import", async move {
                let result = state
                    .data_begin_bootstrap(import, execution, reservation)
                    .await;
                if result.is_ok() {
                    owner.bootstrap_limits = Some(limits);
                }
                drop(owner);
                result
            })
            .await
        })
    }

    fn write_bootstrap<'a>(&'a self, batch: RetainedBatch<'a>) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.history.check(&batch.identity().cut().history)?;
            if batch.encoded_bytes() > self.limits.batch_bytes.get() {
                return Err(invalid("private import batch exceeds encoded bound"));
            }
            let owner = self.owner.clone().lock_owned().await;
            let state = owner.state()?;
            let limits = owner
                .bootstrap_limits
                .ok_or_else(|| invalid("private import requires a reconciled begin"))?;
            let execution = Arc::new(
                query::Budget::new(limits)
                    .map_err(|error| failure("initialize private import write budget", error))?,
            );
            let (reservation, identity, entries) = execution
                .prepare(|| {
                    let planning = batch.part().entries().try_fold(0usize, |peak, value| {
                        super::entry::encoding_memory_bytes(value).map(|bytes| peak.max(bytes))
                    })?;
                    let _planning = state.reserve_query(planning, limits.memory_bytes)?;
                    super::entry::check_size(batch.identity(), super::bootstrap::METADATA_BYTES)?;
                    let mut bytes = batch
                        .len()
                        .checked_mul(std::mem::size_of::<super::entry::Retained>())
                        .and_then(|bytes| bytes.checked_add(super::bootstrap::WORKING_BYTES))
                        .ok_or(Error::Data("private import allocation size overflow"))?;
                    for value in batch.part().entries() {
                        let body = super::entry::Retained::encoded_bytes(
                            value,
                            self.limits.entry_bytes.get(),
                        )?;
                        bytes = body
                            .checked_mul(2)
                            .and_then(|body| bytes.checked_add(body))
                            .ok_or(Error::Data("private import allocation size overflow"))?;
                    }
                    let reservation = state.reserve_query(bytes, limits.memory_bytes)?;
                    let identity = batch.identity().clone();
                    let mut entries = Vec::with_capacity(batch.len());
                    for value in batch.part().entries() {
                        entries.push(super::entry::Retained::new(
                            value,
                            self.limits.entry_bytes.get(),
                        )?);
                    }
                    Ok((reservation, identity, entries))
                })
                .map_err(|error| failure("prepare private import rows", error))?;
            let maximum = self.limits.entry_bytes;
            owned("write private import rows", async move {
                let result = state
                    .data_write_bootstrap(identity, entries, execution, reservation, maximum)
                    .await;
                drop(owner);
                result
            })
            .await
        })
    }

    fn finish_bootstrap<'a>(
        &'a self,
        manifest: &'a BootstrapManifest,
    ) -> StorageFuture<'a, StorageCursor> {
        Box::pin(async move {
            manifest.check_complete()?;
            self.history.check(&manifest.identity().cut().history)?;
            let mut owner = self.owner.clone().lock_owned().await;
            owner.sweep();
            let state = owner.state()?;
            let limits = match owner.bootstrap_limits {
                Some(limits) => limits,
                None => state
                    .data_bootstrap_limits()
                    .await
                    .map_err(|error| failure("reconcile private import finish policy", error))?
                    .ok_or_else(|| invalid("private import requires a reconciled begin"))?,
            };
            let execution = Arc::new(
                query::Budget::new(limits)
                    .map_err(|error| failure("initialize private import verification", error))?,
            );
            let (reservation, manifest) = execution
                .prepare(|| {
                    super::entry::check_size(manifest, super::bootstrap::METADATA_BYTES)?;
                    let bytes = super::bootstrap_files::working_bytes(self.root.as_os_str().len())?;
                    let reservation = state.reserve_query(bytes, limits.memory_bytes)?;
                    Ok((reservation, manifest.clone()))
                })
                .map_err(|error| failure("prepare private import verification", error))?;
            let maximum = self.limits.entry_bytes;
            let root = self.root.clone();
            owner.retention_through = 0;
            owned("prepare private import commit", async move {
                let result = state
                    .data_finish_bootstrap(manifest, root, execution, reservation, maximum)
                    .await;
                drop(owner);
                result
            })
            .await
        })
    }

    fn abort_bootstrap<'a>(&'a self, identity: &'a BootstrapId) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.history.check(&identity.cut().history)?;
            super::entry::check_size(identity, super::bootstrap::METADATA_BYTES)
                .map_err(|error| failure("validate private import abort", error))?;
            let mut owner = self.owner.clone().lock_owned().await;
            let state = owner.state()?;
            let bytes = super::bootstrap_files::working_bytes(self.root.as_os_str().len())
                .map_err(|error| failure("size private import cleanup", error))?;
            let reservation = state
                .reserve_memory(bytes)
                .map_err(|error| failure("reserve private import abort", error))?;
            let identity = identity.clone();
            let root = self.root.clone();
            owned("abort private replica import", async move {
                let result = state
                    .data_abort_bootstrap(identity, root, reservation)
                    .await;
                if result.is_ok() {
                    owner.bootstrap_limits = state.data_bootstrap_limits().await?;
                }
                drop(owner);
                result
            })
            .await
        })
    }

    fn write<'a>(&'a self, batch: StorageBatch<'a>) -> StorageFuture<'a, StorageCursor> {
        Box::pin(async move {
            self.history.check(&batch.first().history)?;
            let state = self.owner.lock().await.state()?;
            let cut = state
                .data_write(
                    &self.root,
                    batch,
                    self.limits.entry_bytes,
                    self.limits.batch_bytes,
                )
                .await
                .map_err(|error| failure("write SQLite batch", error))?;
            Ok(StorageCursor {
                history: cut.history,
                sequence: cut.sequence,
            })
        })
    }

    fn read_view<'a>(&'a self, through: &'a StorageCursor) -> StorageFuture<'a, BackendSnapshot> {
        Box::pin(async move {
            self.view(through, None, false)
                .await
                .map(|(snapshot, _)| snapshot)
        })
    }
    fn read_query_view<'a>(
        &'a self,
        through: &'a StorageCursor,
        limits: QueryLimits,
    ) -> StorageFuture<'a, BackendSnapshot> {
        Box::pin(async move {
            self.validate_query_limits(limits)?;
            self.view(through, Some(limits), false)
                .await
                .map(|(snapshot, _)| snapshot)
        })
    }
    fn read_retained_view<'a>(
        &'a self,
        through: &'a StorageCursor,
        limits: QueryLimits,
    ) -> StorageFuture<'a, RetainedSnapshot> {
        Box::pin(async move {
            self.validate_query_limits(limits)?;
            if !self.supports_retained_snapshots() {
                return Err(invalid("retained file-pin protection is not implemented"));
            }
            let (snapshot, cut) = self.view(through, Some(limits), true).await?;
            // The verified cut encodes its zero-based generation injectively.
            let generation = cut
                .retention_generation
                .checked_add(1)
                .and_then(std::num::NonZeroU64::new)
                .expect("verified retained generation");
            Ok(RetainedSnapshot {
                coverage: RetainedCoverage {
                    cut: snapshot.saved.clone(),
                    complete_after: cut.lost_objects_through,
                    retention_generation: generation,
                },
                snapshot,
            })
        })
    }

    fn release_view<'a>(&'a self, snapshot: &'a BackendSnapshot) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.history.check(&snapshot.saved.history)?;
            let pin = {
                let owner = self.owner.lock().await;
                let Some(slot) = owner
                    .views
                    .iter()
                    .filter_map(Option::as_ref)
                    .find(|slot| slot.token.as_slice() == snapshot.token)
                else {
                    return Ok(());
                };
                if slot.pin.cut.sequence != snapshot.saved.sequence {
                    return Err(invalid("snapshot frontier differs from its pin"));
                }
                // Close Read Admission; A Cancelled Release Keeps This Same Pin.
                slot.pin.released.store(true, Ordering::Release);
                slot.pin.clone()
            };
            loop {
                let settled = pin.settled.notified();
                tokio::pin!(settled);
                settled.as_mut().enable();
                if pin.active_reads.load(Ordering::Acquire) == 0 {
                    break;
                }
                settled.await;
            }
            // Sorted Copies Belong To This Pin; Drop Them Before Its Slot.
            let runs = pin.runs.take();
            let state = self.owner.lock().await.state.clone();
            if let Some(state) = state.filter(|_| !runs.is_empty())
                && let Err(error) = state
                    .call(move |connection| Ok(super::scan::release(connection, &runs)))
                    .await
                    .and_then(|result| result)
            {
                tracing::warn!(%error, "SQLite sorted frame scans outlived their view");
            }
            // The Last Accepted Read Has Released Its Physical Data Protection.
            self.owner.lock().await.sweep();
            Ok(())
        })
    }

    read_entries!(read_schemas, StorageSchema, Schema, Schemas);
    read_entries!(read_objects, ObjectEntry, Object, Objects);
    fn validate_record_filter(
        &self,
        selection: &ReadSelection,
        limits: QueryLimits,
    ) -> harmos::Result<()> {
        super::record_filter::admission(selection, limits)
    }

    read_entries!(read_records, StoredRecord, Record, Records);
    read_entries!(read_stream, StoredRow, Stream, Stream);
    read_entries!(read_files, FileReference, File, Files);

    fn read_object<'a>(
        &'a self,
        view: &'a ReadView,
        query: &'a ObjectPoint,
        maximum: NonZeroUsize,
    ) -> StorageFuture<'a, Option<ObjectEntry>> {
        Box::pin(async move {
            let (state, pin) = self.pin(view).await?;
            let (through, query) = pin
                .prepare(|| Ok((view.saved().clone(), query.clone())))
                .map_err(|error| failure("prepare SQLite point query", error))?;
            let maximum = maximum.min(self.limits.entry_bytes);
            owned_read("read SQLite object", async move {
                let result = state
                    .data_object(
                        through,
                        pin.cut.retention_generation,
                        query,
                        maximum,
                        pin.execution.clone(),
                    )
                    .await;
                Ok(OwnedRead { result, _pin: pin })
            })
            .await
        })
    }
    fn create_bootstrap_file<'a>(
        &'a self,
        identity: &'a BootstrapId,
        metadata: &'a FileMetadata,
    ) -> StorageFuture<'a, StagedFile> {
        Box::pin(async move {
            self.history.check(&identity.cut().history)?;
            let owner = self.owner.clone().lock_owned().await;
            let state = owner.state()?;
            let limits = owner
                .bootstrap_limits
                .ok_or_else(|| invalid("private import requires a reconciled begin"))?;
            let execution = Arc::new(
                query::Budget::new(limits)
                    .map_err(|error| failure("initialize private file staging", error))?,
            );
            let (reservation, identity, metadata, root) = execution
                .prepare(|| {
                    super::entry::check_size(identity, super::bootstrap::METADATA_BYTES)?;
                    super::entry::check_size(metadata, super::bootstrap_files::METADATA_BYTES)?;
                    let bytes = super::bootstrap_files::working_bytes(self.root.as_os_str().len())?;
                    let reservation = state.reserve_query(bytes, limits.memory_bytes)?;
                    Ok((
                        reservation,
                        identity.clone(),
                        metadata.clone(),
                        self.root.clone(),
                    ))
                })
                .map_err(|error| failure("prepare private file staging", error))?;
            owned("create private import file", async move {
                let result = state
                    .data_create_bootstrap_file(root, identity, metadata, execution, reservation)
                    .await;
                drop(owner);
                result
            })
            .await
        })
    }

    fn write_bootstrap_file<'a>(
        &'a self,
        identity: &'a BootstrapId,
        staging: &'a StagedFile,
        offset: u64,
        bytes: &'a [u8],
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            if bytes.is_empty() || bytes.len() > self.limits.batch_bytes.get() {
                return Err(invalid(
                    "private file batch exceeds the configured byte limit",
                ));
            }
            self.history.check(&identity.cut().history)?;
            let owner = self.owner.clone().lock_owned().await;
            let state = owner.state()?;
            let limits = owner
                .bootstrap_limits
                .ok_or_else(|| invalid("private import requires a reconciled begin"))?;
            let execution = Arc::new(
                query::Budget::new(limits)
                    .map_err(|error| failure("initialize private file write", error))?,
            );
            let (reservation, identity, staging, root, bytes) = execution
                .prepare(|| {
                    super::entry::check_size(identity, super::bootstrap::METADATA_BYTES)?;
                    super::entry::check_size(
                        &(&staging.history, &staging.id),
                        super::bootstrap_files::METADATA_BYTES,
                    )?;
                    let working =
                        super::bootstrap_files::working_bytes(self.root.as_os_str().len())?
                            .checked_add(bytes.len())
                            .ok_or(Error::Data("private file write allocation overflow"))?;
                    let reservation = state.reserve_query(working, limits.memory_bytes)?;
                    Ok((
                        reservation,
                        identity.clone(),
                        staging.clone(),
                        self.root.clone(),
                        bytes.to_vec(),
                    ))
                })
                .map_err(|error| failure("prepare private file write", error))?;
            owned("write private import file", async move {
                let chunk = super::bootstrap_files::Chunk {
                    offset,
                    bytes: &bytes,
                };
                let result = state
                    .data_write_bootstrap_file(
                        root,
                        identity,
                        staging,
                        chunk,
                        execution,
                        reservation,
                    )
                    .await;
                drop(owner);
                result
            })
            .await
        })
    }

    fn finish_bootstrap_file<'a>(
        &'a self,
        identity: &'a BootstrapId,
        staging: &'a StagedFile,
        position: &'a StorageCursor,
        length: u64,
        sha256: [u8; 32],
    ) -> StorageFuture<'a, FileReference> {
        Box::pin(async move {
            self.history.check(&identity.cut().history)?;
            let owner = self.owner.clone().lock_owned().await;
            let state = owner.state()?;
            let limits = owner
                .bootstrap_limits
                .ok_or_else(|| invalid("private import requires a reconciled begin"))?;
            let execution = Arc::new(
                query::Budget::new(limits)
                    .map_err(|error| failure("initialize private file finish", error))?,
            );
            let (reservation, identity, staging, root, finish) = execution
                .prepare(|| {
                    super::entry::check_size(identity, super::bootstrap::METADATA_BYTES)?;
                    super::entry::check_size(
                        &(&staging.history, &staging.id),
                        super::bootstrap_files::METADATA_BYTES,
                    )?;
                    super::entry::check_size(position, super::bootstrap::METADATA_BYTES)?;
                    let working =
                        super::bootstrap_files::working_bytes(self.root.as_os_str().len())?;
                    let reservation = state.reserve_query(working, limits.memory_bytes)?;
                    let finish = super::bootstrap_files::Finish {
                        position: position.clone(),
                        length,
                        sha256,
                    };
                    Ok((
                        reservation,
                        identity.clone(),
                        staging.clone(),
                        self.root.clone(),
                        finish,
                    ))
                })
                .map_err(|error| failure("prepare private file finish", error))?;
            owned("finish private import file", async move {
                let result = state
                    .data_finish_bootstrap_file(
                        root,
                        identity,
                        staging,
                        finish,
                        execution,
                        reservation,
                    )
                    .await;
                drop(owner);
                result
            })
            .await
        })
    }

    fn abort_bootstrap_file<'a>(
        &'a self,
        identity: &'a BootstrapId,
        staging: &'a StagedFile,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.history.check(&identity.cut().history)?;
            let owner = self.owner.clone().lock_owned().await;
            let state = owner.state()?;
            // Cleanup Has No Query Execution Cutoff
            super::entry::check_size(identity, super::bootstrap::METADATA_BYTES)
                .map_err(|error| failure("validate private cleanup identity", error))?;
            super::entry::check_size(
                &(&staging.history, &staging.id),
                super::bootstrap_files::METADATA_BYTES,
            )
            .map_err(|error| failure("validate private cleanup staging", error))?;
            let working = super::bootstrap_files::working_bytes(self.root.as_os_str().len())
                .map_err(|error| failure("size private cleanup", error))?;
            let reservation = state
                .reserve_query(working, self.limits.memory_bytes)
                .map_err(|error| failure("reserve private cleanup", error))?;
            let identity = identity.clone();
            let staging = staging.clone();
            let root = self.root.clone();
            owned("abort private import file", async move {
                let result = state
                    .data_abort_bootstrap_file(root, identity, staging, reservation)
                    .await;
                drop(owner);
                result
            })
            .await
        })
    }

    fn supports_retained_snapshots(&self) -> bool {
        true
    }
    fn supports_file_deletion(&self) -> bool {
        true
    }
    fn delete_file<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            let state = self.owner.lock().await.state()?;
            let root = self.root.clone();
            let file = file.clone();
            owned("delete file", async move {
                state.production_delete(root, file).await
            })
            .await
        })
    }
    fn supports_file_sessions(&self) -> bool {
        true
    }
    fn supports_mutation_receipts(&self) -> bool {
        true
    }

    fn create_production<'a>(
        &'a self,
        metadata: &'a FileMetadata,
    ) -> StorageFuture<'a, StagedFile> {
        Box::pin(async move {
            super::entry::check_size(metadata, 16 * 1024)
                .map_err(|error| failure("validate production", error))?;
            let state = self.owner.lock().await.state()?;
            let root = self.root.clone();
            let metadata = metadata.clone();
            owned("create production", async move {
                state.production_create(root, metadata).await
            })
            .await
        })
    }
    fn fence_production<'a>(
        &'a self,
        file: &'a StagedFile,
        expected: u64,
        next: u64,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.history.check(&file.history)?;
            let state = self.owner.lock().await.state()?;
            let file = file.clone();
            owned("fence production", async move {
                state.production_fence(file, expected, next).await
            })
            .await
        })
    }
    fn save_production<'a>(&'a self, checkpoint: &'a FileCheckpoint) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            super::entry::check_size(checkpoint, 16 * 1024)
                .map_err(|error| failure("validate checkpoint", error))?;
            let state = self.owner.lock().await.state()?;
            let checkpoint = checkpoint.clone();
            let root = self.root.clone();
            owned("save production", async move {
                state.production_save(root, checkpoint).await
            })
            .await
        })
    }
    fn recover_productions(&self, limits: ReadLimits) -> StorageFuture<'_, Vec<FileCheckpoint>> {
        Box::pin(async move {
            let state = self.owner.lock().await.state()?;
            let root = self.root.clone();
            owned("recover productions", async move {
                state.production_recover(root, limits).await
            })
            .await
        })
    }
    fn file_progress<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, FileProgress> {
        Box::pin(async move {
            let state = self.owner.lock().await.state()?;
            let root = self.root.clone();
            let file = file.clone();
            owned("read file progress", async move {
                state.production_progress(root, file).await
            })
            .await
        })
    }
    fn read_production<'a>(
        &'a self,
        file: &'a StagedFile,
        offset: u64,
        maximum: NonZeroUsize,
    ) -> StorageFuture<'a, Vec<u8>> {
        Box::pin(async move {
            let state = self.owner.lock().await.state()?;
            let root = self.root.clone();
            let file = file.clone();
            owned("read production", async move {
                state.production_read(root, file, offset, maximum).await
            })
            .await
        })
    }
    fn abandon_file<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            let state = self.owner.lock().await.state()?;
            let root = self.root.clone();
            let file = file.clone();
            owned("abandon production", async move {
                state.production_abandon(root, file).await
            })
            .await
        })
    }
    fn save_mutations<'a>(&'a self, receipts: &'a [MutationReceipt]) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            let mut bytes = 0usize;
            for receipt in receipts {
                let encoded = super::entry::check_size(receipt, 32 * 1024)
                    .map_err(|error| failure("validate receipt", error))?;
                bytes = bytes
                    .checked_add(encoded)
                    .and_then(|bytes| bytes.checked_add(receipt.memory_size()))
                    .filter(|bytes| *bytes <= self.limits.batch_bytes.get())
                    .ok_or_else(|| invalid("mutation batch exceeds byte bound"))?;
            }
            let state = self.owner.lock().await.state()?;
            let receipts = receipts.to_vec();
            owned("save mutation receipts", async move {
                state.mutations_save(receipts).await
            })
            .await
        })
    }
    fn recover_mutations(&self, limits: ReadLimits) -> StorageFuture<'_, Vec<MutationReceipt>> {
        Box::pin(async move {
            let state = self.owner.lock().await.state()?;
            owned("recover mutation receipts", async move {
                state.mutations_recover(limits).await
            })
            .await
        })
    }

    fn create_file<'a>(&'a self, metadata: &'a FileMetadata) -> StorageFuture<'a, StagedFile> {
        Box::pin(async move {
            super::entry::check_size(metadata, 16 * 1024)
                .map_err(|error| failure("validate SQLite file metadata", error))?;
            let state = self.owner.lock().await.state()?;
            let root = self.root.clone();
            let metadata = metadata.clone();
            owned("create SQLite file", async move {
                state.data_create_file(root, metadata).await
            })
            .await
        })
    }
    fn write_file<'a>(
        &'a self,
        staging: &'a StagedFile,
        offset: u64,
        bytes: &'a [u8],
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            if bytes.is_empty() || bytes.len() > self.limits.batch_bytes.get() {
                return Err(invalid("file batch exceeds the configured byte limit"));
            }
            let state = self.owner.lock().await.state()?;
            let root = self.root.clone();
            let staging = staging.clone();
            // data_write_file reserves its copied chunk before allocation and
            // transfers that charge into the physical SQLite callback.
            state
                .data_write_file(root, staging, offset, bytes)
                .await
                .map_err(|error| failure("write SQLite file", error))
        })
    }
    fn finish_file<'a>(
        &'a self,
        staging: &'a StagedFile,
        position: &'a StorageCursor,
        length: u64,
        sha256: [u8; 32],
    ) -> StorageFuture<'a, FileReference> {
        Box::pin(async move {
            let state = self.owner.lock().await.state()?;
            let root = self.root.clone();
            let staging = staging.clone();
            let position = position.clone();
            owned("finish SQLite file", async move {
                state
                    .data_finish_file(root, staging, position, length, sha256)
                    .await
            })
            .await
        })
    }
    fn abort_file<'a>(&'a self, staging: &'a StagedFile) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            let state = self.owner.lock().await.state()?;
            let root = self.root.clone();
            let staging = staging.clone();
            owned("abort SQLite file", async move {
                state.data_abort_file(root, staging).await
            })
            .await
        })
    }
    fn read_file<'a>(
        &'a self,
        view: &'a ReadView,
        file: &'a FileReference,
        offset: u64,
        maximum: NonZeroUsize,
    ) -> StorageFuture<'a, Vec<u8>> {
        Box::pin(async move {
            if !view.accepted().covers(&file.position)? {
                return Err(invalid("file exceeds view admission frontier"));
            }
            let (state, pin) = self.pin(view).await?;
            let (root, reference) = pin
                .prepare(|| Ok((self.root.clone(), file.clone())))
                .map_err(|error| failure("prepare SQLite file read", error))?;
            owned_read("read SQLite file", async move {
                let read = super::files::Read {
                    root,
                    reference,
                    through: pin.cut.sequence,
                    offset,
                    maximum,
                };
                let result = state.data_read_file(read, pin.execution.clone()).await;
                Ok(OwnedRead { result, _pin: pin })
            })
            .await
        })
    }
    fn read_object_releases<'a>(
        &'a self,
        view: &'a ReadView,
        cursor: Option<&'a ObjectReleaseCursor>,
        limits: ReadLimits,
    ) -> StorageFuture<'a, ObjectReleasePage> {
        Box::pin(async move {
            let (state, pin) = self.pin(view).await?;
            let cursor = cursor.cloned();
            let checked_view = view.clone();
            owned_read("read SQLite object releases", async move {
                let result = state
                    .data_object_releases(
                        pin.cut.clone(),
                        cursor.clone(),
                        limits,
                        pin.execution.clone(),
                    )
                    .await
                    .and_then(|page| {
                        page.check(&checked_view, cursor.as_ref(), limits)?;
                        Ok(page)
                    });
                Ok(OwnedRead { result, _pin: pin })
            })
            .await
        })
    }

    fn write_bootstrap_releases<'a>(
        &'a self,
        identity: &'a BootstrapId,
        releases: &'a [ObjectRetentionRelease],
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.history.check(&identity.cut().history)?;
            let owner = self.owner.clone().lock_owned().await;
            let state = owner.state()?;
            let limits = owner
                .bootstrap_limits
                .ok_or_else(|| invalid("private release import requires begin"))?;
            let bytes = super::entry::check_size(&releases, self.limits.batch_bytes.get())
                .map_err(|error| failure("measure private object releases", error))?
                .checked_mul(4)
                .and_then(|bytes| bytes.checked_add(super::bootstrap::WORKING_BYTES))
                .ok_or_else(|| invalid("private release import allocation overflow"))?;
            let reservation = state
                .reserve_query(bytes, limits.memory_bytes)
                .map_err(|error| failure("reserve private object releases", error))?;
            let identity = identity.clone();
            let releases = releases.to_vec();
            owned("write private object releases", async move {
                let result = state
                    .data_bootstrap_releases(identity, releases, reservation)
                    .await;
                drop(owner);
                result
            })
            .await
        })
    }

    fn release_object_history<'a>(
        &'a self,
        schema: &'a SchemaKey,
        item: &'a ObjectKey,
        through: &'a StorageCursor,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.history.check(&through.history)?;
            let mut owner = self.owner.clone().lock_owned().await;
            let state = owner.state()?;
            let bytes = schema
                .memory_size()
                .checked_add(item.as_str().len())
                .and_then(|bytes| bytes.checked_add(through.memory_size()))
                .and_then(|bytes| bytes.checked_add(512))
                .ok_or_else(|| invalid("object history release allocation overflow"))?;
            let reservation = state
                .reserve_memory(bytes)
                .map_err(|error| failure("reserve SQLite object history release", error))?;
            let schema = schema.clone();
            let item = item.clone();
            let through = through.clone();
            owned("release SQLite object history", async move {
                let _reservation = reservation;
                state
                    .data_release_object_history(schema, item, through)
                    .await?;
                // Revisit Previously Saved Revisions After A New Lifecycle Release.
                owner.retention_through = 0;
                Ok(())
            })
            .await
        })
    }
    fn prune_objects<'a>(
        &'a self,
        through: &'a StorageCursor,
        maximum: NonZeroUsize,
    ) -> StorageFuture<'a, bool> {
        Box::pin(async move {
            self.history.check(&through.history)?;
            let mut owner = self.owner.clone().lock_owned().await;
            owner.sweep();
            let oldest = owner.pins().map(|pin| pin.cut.retention_generation).min();
            let through = through.clone();
            owned("prune SQLite objects", async move {
                let state = owner.state().map_err(Error::Runtime)?;
                let after = if owner.retention_through <= through.sequence {
                    owner.retention_through
                } else {
                    0
                };
                let sequence = through.sequence;
                let (logical_complete, complete) = state
                    .data_prune::<C>(through, after, oldest, maximum)
                    .await?;
                if logical_complete {
                    owner.retention_through = sequence;
                }
                Ok(complete)
            })
            .await
        })
    }
    fn close(&self) -> StorageFuture<'_, ()> {
        Box::pin(async move {
            let mut owner = self.owner.clone().lock_owned().await;
            owned("close SQLite", async move {
                owner.primary_active = false;
                owner.close_if_idle().await
            })
            .await
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn invalid(reason: &'static str) -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "SQLite storage",
        reason,
    }
}
fn failure(operation: &'static str, error: Error) -> harmos::Error {
    match error {
        Error::Runtime(error) => error,
        Error::Data(reason) => invalid(reason),
        other => {
            let kind = match &other {
                Error::Owned | Error::Conflict(_) => BackendFailure::Conflict,
                Error::Json(_) | Error::Corrupt(_) => BackendFailure::Corrupt,
                Error::Io(_) | Error::Sqlite(_) | Error::Publication(_) => {
                    BackendFailure::Transient
                }
                _ => BackendFailure::Permanent,
            };
            harmos::Error::backend(operation, kind, other)
        }
    }
}
async fn owned_read<T: Send + 'static>(
    operation: &'static str,
    future: impl std::future::Future<Output = Result<OwnedRead<T>, Error>> + Send + 'static,
) -> harmos::Result<T> {
    let settled = owned(operation, future).await?;
    settled.result.map_err(|error| failure(operation, error))
}

async fn owned<T: Send + 'static>(
    operation: &'static str,
    future: impl std::future::Future<Output = Result<T, Error>> + Send + 'static,
) -> harmos::Result<T> {
    tokio::spawn(future)
        .await
        .map_err(|error| harmos::Error::backend(operation, BackendFailure::Permanent, error))?
        .map_err(|error| failure(operation, error))
}

/* === Tests === */
