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
use std::{future::poll_fn, path::PathBuf, pin::Pin, sync::Arc};
// Third Party Imports
use harmos::{services::Fault, services::Resident, services::Service};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncSeek, AsyncSeekExt};
// Local Imports
use crate::runtime::storage::executions::ExecutionStorage;
use crate::{runtime::Error, runtime::Result, RuntimeHandle};
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */
const LOG_BUFFER_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct LogTail {
    storage: Arc<ExecutionStorage>,
    source: PathBuf,
    execution: TestExecutionId,
    path: String,
    offset: u64,
    opening: Option<tokio::task::JoinHandle<std::io::Result<std::fs::File>>>,
    file: Option<tokio::fs::File>,
    buffer: Vec<u8>,
    source_failure: Option<std::io::Error>,
    final_end: Option<u64>,
    drained: bool,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
enum TailError {
    #[error("log source failed: {0}")]
    Source(#[source] std::io::Error),
    #[error("log source acquisition failed: {0}")]
    Opening(#[source] tokio::task::JoinError),
    #[error("log storage failed: {0}")]
    Storage(#[source] Error),
    #[error("log state is invalid: {0}")]
    State(&'static str),
}

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn start_execution_log_tail(
        &self,
        execution: TestExecutionId,
        relative_path: impl Into<String>,
        source: impl Into<PathBuf>,
    ) -> Result<String> {
        let runtime = self.clone();
        let relative_path = relative_path.into();
        let source = source.into();
        tokio::spawn(async move {
            let path = relative_path;
            super::validate_relative_path(&path)?;
            let name = format!(
                "execution-log:{execution}:{}",
                Sha256::digest(path.as_bytes())
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            );
            let preparation = runtime.execution_storage.prepare_tail_stop(&name)?;
            let mut stops = runtime.execution_storage.tail_stops.lock().await;
            runtime
                .execution_storage
                .reserve_output(&execution, &path)
                .await?;
            let storage = runtime.execution_storage.clone();
            runtime
                .operations
                .register_service(
                    harmos::services::ServiceDefinition {
                        id: name.clone(),
                        policy: harmos::services::POLICY,
                    },
                    move |_: Arc<()>| LogTail {
                        storage: Arc::clone(&storage),
                        source: source.clone(),
                        execution: execution.clone(),
                        path: path.clone(),
                        offset: 0,
                        opening: None,
                        file: None,
                        buffer: vec![0; LOG_BUFFER_BYTES],
                        source_failure: None,
                        final_end: None,
                        drained: false,
                    },
                )
                .map_err(|error| Error::Operation(error.to_string()))?;
            let lease = match runtime.operations.submit_resident(&name).await {
                Ok(lease) => lease,
                Err(error) => {
                    runtime
                        .operations
                        .unregister_resident(&name)
                        .await
                        .map_err(|error| Error::Operation(error.to_string()))?;
                    return Err(Error::Operation(error.to_string()));
                }
            };
            let (key, barrier) = preparation.into_entry(lease);
            stops.insert(key, barrier);
            Ok(name)
        })
        .await
        .map_err(|error| Error::Operation(error.to_string()))?
    }

    pub async fn stop_execution_log_tail(&self, name: &str) -> Result<()> {
        let runtime = self.clone();
        let name = name.to_owned();
        tokio::spawn(async move {
            let barrier = runtime
                .execution_storage
                .tail_stops
                .lock()
                .await
                .get(&name)
                .cloned();
            if let Some(barrier) = barrier {
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
            }
            Ok(())
        })
        .await
        .map_err(|error| Error::Operation(error.to_string()))?
    }
    pub(crate) async fn stop_execution_log_tails(
        &self,
        execution: Option<&TestExecutionId>,
    ) -> Result<()> {
        let names = self.execution_storage.tail_names(execution).await?;
        let mut failure = None;
        for name in names.iter() {
            if let Err(error) = self.stop_execution_log_tail(name).await {
                failure.get_or_insert(error);
            }
        }
        let mut stops = self.execution_storage.tail_stops.lock().await;
        for name in names.iter() {
            if self.operations.resident_definition(name).is_none() {
                stops.remove(name);
            }
        }
        failure.map_or(Ok(()), Err)
    }

    pub(crate) async fn finish_execution_outputs(&self, execution: &TestExecutionId) -> Result<()> {
        // Running Captures Seal Into The Execution While It Still Admits Artifacts
        self.execution_storage.captures.close(execution).await;
        {
            let _starts = self.execution_storage.tail_stops.lock().await;
            self.execution_storage
                .seal_output_admission(execution)
                .await?;
        }
        self.stop_execution_log_tails(Some(execution)).await?;
        self.execution_storage.finish_outputs(execution).await
    }
}

impl Resident<crate::app::Dokime> for LogTail {
    type Error = TailError;
    async fn run(
        &mut self,
        mut context: Service<crate::app::Dokime>,
    ) -> std::result::Result<(), Fault<TailError>> {
        self.pump(&mut context).await
    }

    async fn cleanup(&mut self) -> std::result::Result<(), TailError> {
        self.finish_opening().await?;
        if self.source_failure.is_none()
            && let Some(file) = &mut self.file
        {
            // Drain The Original Tokio Read Or Seek Before Releasing Its File.
            // A Failed Blocking Join Must Not Be Polled Again On Cleanup Retry.
            if let Err(error) = poll_fn(|context| Pin::new(&mut *file).poll_complete(context)).await
            {
                self.source_failure = Some(error);
            }
        }
        if self.source_failure.is_none() && !self.drained {
            // Resume From The Acknowledged Prefix After Any Cancelled Append Wait.
            self.offset = self
                .storage
                .log_prefix(&self.execution, &self.path)
                .await
                .map_err(TailError::Storage)?;
            let file = self
                .file
                .as_mut()
                .ok_or(TailError::State("log source is missing"))?;
            let end = match self.final_end {
                Some(end) => end,
                None => {
                    let end = file
                        .seek(std::io::SeekFrom::End(0))
                        .await
                        .map_err(TailError::Source)?;
                    self.final_end = Some(end);
                    end
                }
            };
            if end < self.offset {
                return Err(TailError::State("append-only log was truncated"));
            }
            while self.offset < end {
                let count = (end - self.offset).min(self.buffer.len() as u64) as usize;
                file.seek(std::io::SeekFrom::Start(self.offset))
                    .await
                    .map_err(TailError::Source)?;
                file.read_exact(&mut self.buffer[..count])
                    .await
                    .map_err(TailError::Source)?;
                self.storage
                    .append_output(&self.execution, &self.path, &self.buffer[..count])
                    .await
                    .map_err(TailError::Storage)?;
                self.offset += count as u64;
            }
            self.drained = true;
        }
        if let Some(file) = self.file.take() {
            match file.try_into_std() {
                Ok(file) => drop(file),
                Err(file) => {
                    self.file = Some(file);
                    return Err(TailError::State("log source I/O is still retained"));
                }
            }
        }
        // Normal EOF Already Closed The Writer; This Also Settles Interrupted Attempts.
        self.storage
            .close_output(&self.execution, &self.path, self.source_failure.is_some())
            .await
            .map_err(TailError::Storage)?;
        self.source_failure
            .take()
            .map_or(Ok(()), |error| Err(TailError::Source(error)))
    }
}

impl LogTail {
    async fn finish_opening(&mut self) -> std::result::Result<(), TailError> {
        let Some(opening) = &mut self.opening else {
            return Ok(());
        };
        let result = opening.await;
        self.opening = None;
        let file = result
            .map_err(TailError::Opening)?
            .map_err(TailError::Source)?;
        let mut file = tokio::fs::File::from_std(file);
        file.set_max_buf_size(LOG_BUFFER_BYTES);
        self.file = Some(file);
        Ok(())
    }

    async fn pump(
        &mut self,
        context: &mut Service<crate::app::Dokime>,
    ) -> std::result::Result<(), Fault<TailError>> {
        // Resume From The Original Private Acknowledged Prefix.
        self.offset = self
            .storage
            .log_prefix(&self.execution, &self.path)
            .await
            .map_err(storage_error)?;
        if self.file.is_none() {
            if self.opening.is_none() {
                let source = self.source.clone();
                self.opening = Some(tokio::task::spawn_blocking(move || {
                    std::fs::File::open(source)
                }));
            }
            self.finish_opening().await.map_err(Fault::Environmental)?;
        }
        let file = self.file.as_mut().ok_or_else(|| {
            Fault::Application(TailError::State("log source acquisition is missing"))
        })?;
        context.ready();

        loop {
            // Observe stop before measuring EOF: the producer may append after
            // an earlier metadata read and then request this drain barrier.
            let stopping = context.is_stopping();
            let length = file
                .seek(std::io::SeekFrom::End(0))
                .await
                .map_err(|error| Fault::Environmental(TailError::Source(error)))?;
            if length < self.offset {
                self.storage
                    .close_output(&self.execution, &self.path, true)
                    .await
                    .map_err(storage_error)?;
                return Err(Fault::Application(TailError::State(
                    "append-only log was truncated",
                )));
            }
            if stopping && self.final_end.is_none() {
                self.final_end = Some(length);
            }
            let end = self.final_end.unwrap_or(length);
            if self.offset == end {
                if self.final_end.is_some() {
                    self.drained = true;
                    return self
                        .storage
                        .close_output(&self.execution, &self.path, false)
                        .await
                        .map_err(storage_error);
                }
                context.rest(std::time::Duration::from_millis(25)).await;
                continue;
            }
            let count = (end - self.offset).min(self.buffer.len() as u64) as usize;
            file.seek(std::io::SeekFrom::Start(self.offset))
                .await
                .map_err(|error| Fault::Environmental(TailError::Source(error)))?;
            file.read_exact(&mut self.buffer[..count])
                .await
                .map_err(|error| Fault::Environmental(TailError::Source(error)))?;
            self.storage
                .append_output(&self.execution, &self.path, &self.buffer[..count])
                .await
                .map_err(storage_error)?;
            self.offset += count as u64;
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn storage_error(error: Error) -> Fault<TailError> {
    Fault::Application(TailError::Storage(error))
}

/* === Tests === */
