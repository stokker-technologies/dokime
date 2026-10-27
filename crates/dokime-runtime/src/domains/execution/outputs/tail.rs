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
    collections::BTreeMap,
    io::{Read, Seek},
    path::PathBuf,
    sync::Arc,
};

// Third Party Imports
use harmos::{ServiceContext, services::Resident};
use sha2::{Digest, Sha256};

// Local Imports
use crate::runtime::storage::executions::{ExecutionStorage, TailStop};
use crate::{RuntimeHandle, runtime::Error, runtime::Result};
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type LogBuffer = Vec<u8>;
type FileJob = tokio::task::JoinHandle<(std::fs::File, std::io::Result<FileResult>)>;

/* === Structs === */

struct LogTail {
    poll_interval: std::time::Duration,
    storage: Arc<ExecutionStorage>,
    source: PathBuf,
    execution: TestExecutionId,
    path: String,
    offset: u64,
    opening: Option<tokio::task::JoinHandle<std::io::Result<std::fs::File>>>,
    file: Option<std::fs::File>,
    pending: Option<FileJob>,
    read_bytes: usize,
    source_failure: Option<std::io::Error>,
    final_end: Option<u64>,
    drained: bool,
}

/* === Enums === */

enum FileResult {
    End(u64),
    Bytes(LogBuffer),
}

#[derive(Debug, thiserror::Error)]
enum TailError {
    #[error("log source failed: {0}")]
    Source(#[source] std::io::Error),

    #[error("log source acquisition failed: {0}")]
    Opening(#[source] tokio::task::JoinError),

    #[error("log source I/O task failed: {0}")]
    IoTask(#[source] tokio::task::JoinError),

    #[error("log storage failed: {0}")]
    Storage(#[source] Error),

    #[error("log state is invalid: {0}")]
    State(&'static str),
}

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn start_execution_log_tail(
        &self, execution: TestExecutionId, relative_path: impl Into<String>, source: impl Into<PathBuf>,
    ) -> Result<String> {
        let runtime = self.clone();
        let relative_path = relative_path.into();
        let source = source.into();
        tokio::spawn(async move {
            let path = relative_path;
            super::validate_relative_path(&path)?;
            let digest = Sha256::digest(path.as_bytes());
            let mut hexadecimal = [0; 32 * 2];
            for (byte, digits) in digest.iter().zip(hexadecimal.as_chunks_mut::<2>().0) {
                digits[0] = b"0123456789abcdef"[(byte >> 4) as usize];
                digits[1] = b"0123456789abcdef"[(byte & 15) as usize];
            }
            let key = std::str::from_utf8(&hexadecimal).expect("ASCII SHA-256 digest");
            let name = format!("execution-log:{execution}:{key}");
            let preparation = runtime.execution_storage.prepare_tail_stop(&name)?;
            let mut stops = runtime.execution_storage.tail_stops.lock().await;
            runtime.execution_storage.reserve_output(&execution, &path).await?;
            let storage = runtime.execution_storage.clone();
            let poll_interval = runtime.storage().settings().filesystem.poll_interval;
            let read_bytes = 64 * 1024;
            let definition = harmos::services::ServiceDefinition { id: name.clone(), policy: harmos::services::POLICY };
            let factory = move |_: Arc<()>| LogTail {
                poll_interval,
                storage: Arc::clone(&storage),
                source: source.clone(),
                execution: execution.clone(),
                path: path.clone(),
                offset: 0,
                opening: None,
                file: None,
                pending: None,
                read_bytes,
                source_failure: None,
                final_end: None,
                drained: false,
            };
            runtime.operations.register_service(definition, factory).map_err(operation_error)?;
            let lease = match runtime.operations.submit_resident(&name).await {
                Ok(lease) => lease,
                Err(error) => {
                    runtime.operations.unregister_resident(&name).await.map_err(operation_error)?;
                    return Err(operation_error(error));
                }
            };
            let (key, barrier) = preparation.into_entry(lease);
            stops.insert(key, barrier);
            Ok(name)
        })
        .await
        .map_err(operation_error)?
    }

    pub async fn stop_execution_log_tail(&self, name: &str) -> Result<()> {
        let runtime = self.clone();
        let name = name.to_owned();
        tokio::spawn(async move {
            let barrier = runtime.execution_storage.tail_stops.lock().await.get(&name).cloned();
            let Some(barrier) = barrier else {
                return Ok(());
            };
            let mut lease = barrier.lease.lock().await;
            let released = match lease.take() {
                Some(lease) => lease.release().await,
                None => Ok(()),
            };
            // A Failed Release Leaves The Resident In The Original Manager.
            // Retry Its Physical Finalizer Even After The Lease Was Consumed.
            let removed = if runtime.operations.resident_definition(&name).is_some() {
                runtime.operations.unregister_resident(&name).await
            } else {
                Ok(())
            };
            released.and(removed).map_err(Error::from)?;
            Ok(())
        })
        .await
        .map_err(operation_error)?
    }
    pub(crate) async fn stop_execution_log_tails(&self, execution: Option<&TestExecutionId>) -> Result<()> {
        let names = self.execution_storage.tail_names(execution).await?;
        let mut failure = None;
        for name in names.iter() {
            let current = self.stop_execution_log_tail(name).await.err();
            failure = failure.or(current);
        }
        let mut stops = self.execution_storage.tail_stops.lock().await;
        for name in names.iter() {
            remove_stopped_tail(self, name, &mut stops);
        }
        failure.map_or(Ok(()), Err)
    }

    pub(crate) async fn finish_execution_outputs(&self, execution: &TestExecutionId) -> Result<()> {
        // Running Captures Seal Into The Execution While It Still Admits Artifacts
        self.execution_storage.captures.close(execution).await;
        {
            let _starts = self.execution_storage.tail_stops.lock().await;
            self.execution_storage.seal_output_admission(execution).await?;
        }
        self.stop_execution_log_tails(Some(execution)).await?;
        self.execution_storage.finish_outputs(execution).await
    }
}

impl Resident<crate::app::Dokime> for LogTail {
    type Error = TailError;
    async fn run(&mut self, mut context: ServiceContext<crate::app::Dokime>) -> std::result::Result<(), TailError> {
        self.pump(&mut context).await
    }

    async fn cleanup(&mut self) -> std::result::Result<(), TailError> {
        self.finish_opening().await?;
        self.settle_io().await?;
        if self.source_failure.is_none() && !self.drained {
            self.drain().await?;
        }
        drop(self.file.take());
        // Normal EOF Already Closed The Writer; This Also Settles Interrupted Attempts.
        self.storage.close_output(&self.execution, &self.path, self.source_failure.is_some()).await.map_err(TailError::Storage)?;
        match self.source_failure.take() {
            Some(error) => Err(TailError::Source(error)),
            None => Ok(()),
        }
    }
}

impl LogTail {
    async fn drain(&mut self) -> std::result::Result<(), TailError> {
        // Resume From The Acknowledged Prefix After Any Cancelled Append Wait.
        self.offset = self.storage.log_prefix(&self.execution, &self.path).await.map_err(TailError::Storage)?;
        let end = match self.final_end {
            Some(end) => end,
            None => {
                let end = self.end().await?;
                self.final_end = Some(end);
                end
            }
        };
        if end < self.offset {
            return Err(TailError::State("append-only log was truncated"));
        }
        while self.offset < end {
            let bytes = self.read_chunk(end).await?;
            let count = bytes.len();
            self.storage.append_output(&self.execution, &self.path, bytes).await.map_err(TailError::Storage)?;
            self.offset += count as u64;
        }
        self.drained = true;
        Ok(())
    }

    async fn finish_opening(&mut self) -> std::result::Result<(), TailError> {
        let Some(opening) = &mut self.opening else {
            return Ok(());
        };
        let result = opening.await;
        self.opening = None;
        let file = result.map_err(TailError::Opening)?.map_err(TailError::Source)?;
        self.file = Some(file);
        Ok(())
    }

    async fn finish_io(&mut self) -> std::result::Result<FileResult, TailError> {
        let pending = self.pending.as_mut();
        let pending = pending.ok_or(TailError::State("log source I/O is missing"))?;
        let result = pending.await;
        self.pending = None;
        let (file, result) = result.map_err(TailError::IoTask)?;
        self.file = Some(file);
        match result {
            Ok(result) => Ok(result),
            Err(error) => {
                let kind = error.kind();
                self.source_failure = Some(error);
                let source = kind.into();
                Err(TailError::Source(source))
            }
        }
    }

    async fn settle_io(&mut self) -> std::result::Result<(), TailError> {
        if self.pending.is_none() {
            return Ok(());
        }
        // Reap The Original Handle And Buffer Before Any Retry, Seek Or Close.
        match self.finish_io().await {
            Ok(_) => Ok(()),
            Err(TailError::Source(_)) => Ok(()),
            Err(error) => Err(error),
        }
    }

    async fn end(&mut self) -> std::result::Result<u64, TailError> {
        let file = self.file.take();
        let mut file = file.ok_or(TailError::State("log source is missing"))?;
        let seek = move || {
            let result = file.seek(std::io::SeekFrom::End(0)).map(FileResult::End);
            (file, result)
        };
        self.pending = Some(tokio::task::spawn_blocking(seek));
        match self.finish_io().await? {
            FileResult::End(end) => Ok(end),
            FileResult::Bytes(_) => Err(TailError::State("unexpected log source read")),
        }
    }

    async fn read_chunk(&mut self, end: u64) -> std::result::Result<LogBuffer, TailError> {
        let capacity = (end - self.offset).min(self.read_bytes as u64) as usize;
        let mut bytes = self.storage.output_buffer(capacity).map_err(TailError::Storage)?;
        bytes.resize(capacity, 0);

        let file = self.file.take();
        let mut file = file.ok_or(TailError::State("log source is missing"))?;
        let offset = self.offset;
        let read = move || {
            let result = (|| {
                file.seek(std::io::SeekFrom::Start(offset))?;
                file.read_exact(&mut bytes)?;
                Ok(FileResult::Bytes(bytes))
            })();
            (file, result)
        };
        self.pending = Some(tokio::task::spawn_blocking(read));
        match self.finish_io().await? {
            FileResult::Bytes(bytes) => Ok(bytes),
            FileResult::End(_) => Err(TailError::State("unexpected log source seek")),
        }
    }

    async fn pump(&mut self, context: &mut ServiceContext<crate::app::Dokime>) -> std::result::Result<(), TailError> {
        self.settle_io().await?;
        // Resume From The Original Private Acknowledged Prefix.
        self.offset = self.storage.log_prefix(&self.execution, &self.path).await.map_err(TailError::Storage)?;
        self.open().await?;
        context.ready();

        while !self.advance(context).await? {}
        Ok(())
    }

    async fn advance(&mut self, context: &mut ServiceContext<crate::app::Dokime>) -> std::result::Result<bool, TailError> {
        // Observe Stop Before Measuring EOF, Including The Final Drain Barrier.
        let stopping = context.is_stopping();
        let length = self.end().await?;
        self.check_length(length).await?;
        if stopping && self.final_end.is_none() {
            self.final_end = Some(length);
        }
        let end = self.final_end.unwrap_or(length);
        if self.offset == end && self.final_end.is_some() {
            self.drained = true;
            self.storage.close_output(&self.execution, &self.path, false).await.map_err(TailError::Storage)?;
            return Ok(true);
        }
        if self.offset == end {
            context.rest(self.poll_interval).await;
            return Ok(false);
        }

        let bytes = self.read_chunk(end).await?;
        let count = bytes.len();
        self.storage.append_output(&self.execution, &self.path, bytes).await.map_err(TailError::Storage)?;
        self.offset += count as u64;
        Ok(false)
    }

    async fn open(&mut self) -> std::result::Result<(), TailError> {
        if self.file.is_some() {
            return Ok(());
        }
        if self.opening.is_none() {
            let source = self.source.clone();
            let open = move || std::fs::File::open(source);
            self.opening = Some(tokio::task::spawn_blocking(open));
        }
        self.finish_opening().await
    }

    async fn check_length(&self, length: u64) -> std::result::Result<(), TailError> {
        if length >= self.offset {
            return Ok(());
        }
        self.storage.close_output(&self.execution, &self.path, true).await.map_err(TailError::Storage)?;
        Err(TailError::State("append-only log was truncated"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn operation_error(error: impl ToString) -> Error {
    Error::Operation(error.to_string())
}

fn remove_stopped_tail(runtime: &RuntimeHandle, name: &str, stops: &mut BTreeMap<String, Arc<TailStop>>) {
    if runtime.operations.resident_definition(name).is_none() {
        stops.remove(name);
    }
}

/* === Tests === */
