/* ***
 *
 * Dokime
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod index;

/* === Re-Exports === */
pub(crate) use index::{AnnotationKind, Index, Member, Sample};

/* === Imports === */
// Standard Library Imports
use std::{
    fmt::Write as _,
    io::{self, Write},
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};
// Third Party Imports
use harmos::{
    MemorySize,
    storage::{FileMetadata, FileReference, QueryValue, StorageFile, StorageProductionPreparation},
};
// Local Imports
use super::html::HtmlReport;
use crate::{
    Error, Result,
    runtime::{progress::Operation, storage::cycle::CycleSnapshot},
};
use dokime::{core::Identifier, storage::Manager};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct CycleReport {
    source: Arc<HtmlReport>,
    file: Arc<Mutex<File>>,
    batch_bytes: NonZeroUsize,
    _memory: Arc<StorageProductionPreparation>,
}
struct File {
    metadata: QueryValue<FileMetadata>,
    producer: Option<StorageFile>,
    written: bool,
    memory: Option<StorageProductionPreparation>,
}
struct Batches<'a> {
    producer: &'a mut StorageFile,
    executor: &'a tokio::runtime::Handle,
    manager: &'a Manager,
    bytes: Vec<u8>,
    length: usize,
    total: usize,
    failure: Option<Error>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl CycleReport {
    pub(crate) async fn prepare(
        manager: &Manager,
        snapshot: CycleSnapshot,
        captured_members: Vec<QueryValue<super::CapturedExecutionReportInput>>,
        batch_bytes: NonZeroUsize,
        operation: Option<&Operation>,
    ) -> Result<Self> {
        let bytes = std::mem::size_of::<File>()
            + std::mem::size_of::<Batches<'_>>()
            + 8 * std::mem::size_of::<usize>();
        let bytes = bytes
            .checked_add(batch_bytes.get())
            .ok_or_else(|| Error::InvalidReportInputs("report batch memory overflows".into()))?;
        let memory = manager.prepare_production(allowance(bytes)?)?;
        let source =
            Arc::new(HtmlReport::from_cycle(manager, snapshot, captured_members, operation).await?);
        let id_length =
            source.header().cycle.test_cycle_id.as_str().len() + 36 + ".cycle-report.".len();
        let metadata = manager
            .produce(
                allowance(
                    std::mem::size_of::<FileMetadata>()
                        + id_length
                        + source.header().cycle.test_cycle_id.as_str().len()
                        + "text/html".len(),
                )?,
                |_| {
                    let mut name = String::with_capacity(id_length);
                    write!(name, "{}.", source.header().cycle.test_cycle_id).expect("String write");
                    write!(name, "{}", uuid::Uuid::now_v7()).expect("String write");
                    name.push_str(".cycle-report");
                    Ok(FileMetadata {
                        id: Identifier::new(name).map_err(|source| {
                            harmos::Error::backend(
                                "cycle report identity",
                                harmos::BackendFailure::Permanent,
                                source,
                            )
                        })?,
                        scope: Some(
                            Identifier::new(source.header().cycle.test_cycle_id.to_string())
                                .map_err(|source| {
                                    harmos::Error::backend(
                                        "cycle report scope",
                                        harmos::BackendFailure::Permanent,
                                        source,
                                    )
                                })?,
                        ),
                        media_type: "text/html".into(),
                        expected_length: None,
                        expected_sha256: None,
                    })
                },
            )?
            .try_into_value()?;
        Ok(Self {
            source,
            batch_bytes,
            file: Arc::new(Mutex::new(File {
                metadata,
                producer: None,
                written: false,
                memory: None,
            })),
            _memory: Arc::new(memory),
        })
    }
    pub(crate) async fn finish(&mut self) -> Result<FileReference> {
        let source = self.source.clone();
        let file = self.file.clone();
        let batch_bytes = self.batch_bytes;
        let memory = self._memory.clone();
        tokio::task::spawn_blocking(move || {
            let _memory = memory;
            let (_, executor) = source.saved_source()?;
            let mut file = file.lock().unwrap();
            if !file.written {
                if let Some(producer) = file.producer.as_mut() {
                    executor.block_on(producer.abort())?;
                }
                if file.memory.is_none() {
                    file.memory = Some(
                        source.manager.prepare_production(allowance(
                            file.metadata
                                .memory_size()
                                .saturating_add(std::mem::size_of::<StorageFile>()),
                        )?)?,
                    );
                }
                let create = source.manager.create_file((*file.metadata).clone());
                let future_memory = source
                    .manager
                    .prepare_production(allowance(std::mem::size_of_val(&create))?)?;
                file.producer = Some(executor.block_on(create)?);
                drop(future_memory);
                let mut bytes = Vec::new();
                bytes
                    .try_reserve_exact(batch_bytes.get())
                    .map_err(|error| Error::Operation(error.to_string()))?;
                if bytes.capacity() > batch_bytes.get() {
                    return Err(Error::InvalidReportInputs(
                        "report batch exceeds reserved capacity".into(),
                    ));
                }
                bytes.resize(batch_bytes.get(), 0);
                let mut writer = Batches {
                    producer: file.producer.as_mut().unwrap(),
                    executor,
                    manager: &source.manager,
                    bytes,
                    length: 0,
                    total: 0,
                    failure: None,
                };
                let outcome = source.write(&mut writer);
                if let Some(source) = writer.failure.take() {
                    return Err(source);
                }
                outcome?;
                let outcome = writer.flush();
                if let Some(source) = writer.failure.take() {
                    return Err(source);
                }
                outcome.map_err(|source| Error::Operation(source.to_string()))?;
                file.written = true;
            }
            let finish = file.producer.as_mut().unwrap().finish();
            let _finish_memory = source
                .manager
                .prepare_production(allowance(std::mem::size_of_val(&finish))?)?;
            Ok(executor.block_on(finish)?)
        })
        .await?
    }
}
impl Write for Batches<'_> {
    fn write(&mut self, mut bytes: &[u8]) -> io::Result<usize> {
        let length = bytes.len();
        self.total = self
            .total
            .checked_add(length)
            .ok_or_else(|| io::Error::other("report output length overflows"))?;
        while !bytes.is_empty() {
            let count = bytes.len().min(self.bytes.len() - self.length);
            self.bytes[self.length..self.length + count].copy_from_slice(&bytes[..count]);
            self.length += count;
            bytes = &bytes[count..];
            if self.length == self.bytes.len() {
                self.flush()?;
            }
        }
        Ok(length)
    }
    fn flush(&mut self) -> io::Result<()> {
        let outcome = (|| -> Result<()> {
            let write = self.producer.write_all(&self.bytes[..self.length]);
            let _write_memory = self
                .manager
                .prepare_production(allowance(std::mem::size_of_val(&write))?)?;
            self.executor.block_on(write)?;
            Ok(())
        })();
        match outcome {
            Ok(()) => {
                self.length = 0;
                Ok(())
            }
            Err(source) => {
                let message = source.to_string();
                self.failure = Some(source);
                Err(io::Error::other(message))
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(crate) fn allowance(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes)
        .filter(|_| bytes != usize::MAX)
        .ok_or_else(|| Error::InvalidReportInputs("cycle report memory overflows".into()))
}

/* === Tests === */
