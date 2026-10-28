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
    io::{Seek, Write},
    sync::Arc,
};

// Third Party Imports

// Local Imports
use super::{LogPrefix, NativeOutput, error, open_file, validate_producer_path};
use crate::{runtime::Result, runtime::storage::executions::ExecutionStorage};
use dokime_domain::model::execution::saved::SavedStreamState;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Close {
    execution: TestExecutionId,
    path: String,
    interrupted: bool,
}

struct Append {
    execution: TestExecutionId,
    path: String,
    bytes: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Append {
    fn write(&self, mut file: std::fs::File, acknowledged: u64) -> std::io::Result<()> {
        if file.metadata()?.len() < acknowledged {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "native log lost its acknowledged prefix"));
        }
        // Only A Failed Private Append Is Removed; Immutable Files Are Untouched.
        file.set_len(acknowledged)?;
        file.seek(std::io::SeekFrom::Start(acknowledged))?;
        file.write_all(&self.bytes)?;
        file.flush()?;

        Ok(())
    }
}

impl ExecutionStorage {
    pub(crate) async fn reserve_output(self: &Arc<Self>, execution: &TestExecutionId, path: &str) -> Result<()> {
        validate_producer_path(path)?;
        let storage = self.clone();
        let execution = execution.clone();
        let path = path.to_owned();
        tokio::spawn(async move {
            let owner = storage.acquisition(&execution).await;
            let Some(owner) = owner else {
                return Err(error("log execution has no admitted producer owner"));
            };
            let mut acquisition = owner.lock().await;
            if acquisition.finish_requested || acquisition.outputs_sealed {
                return Err(error("log producer admission is closed"));
            }
            if let Some(output) = acquisition.native_outputs.get(&path) {
                return active_log(output);
            }
            let mut output = NativeOutput::new(&path)?;
            output.log = Some(LogPrefix { acknowledged: 0, closed: None });
            acquisition.native_outputs.insert(path, output);
            Ok(())
        })
        .await
        .map_err(error)?
    }

    pub(crate) async fn log_prefix(&self, execution: &TestExecutionId, path: &str) -> Result<u64> {
        let owner = self.acquisition(execution).await;
        let Some(owner) = owner else {
            return Err(error("log acquisition is missing"));
        };
        let acquisition = owner.lock().await;

        let output = acquisition.native_outputs.get(path);
        let Some(output) = output else {
            return Err(error("log output is missing"));
        };
        let log = output.log.as_ref();
        let Some(log) = log else {
            return Err(error("output is not a log"));
        };
        if log.closed.is_some() {
            return Err(error("log output is closed"));
        }
        Ok(log.acknowledged)
    }

    pub(crate) fn output_buffer(&self, capacity: usize) -> Result<Vec<u8>> {
        if capacity == 0 {
            return Err(error("log buffer capacity must be positive"));
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(capacity).map_err(error)?;
        Ok(bytes)
    }

    pub(crate) async fn append_output(self: &Arc<Self>, execution: &TestExecutionId, path: &str, bytes: Vec<u8>) -> Result<()> {
        validate_producer_path(path)?;
        let ingress = Append {
            execution: execution.clone(),
            path: path.to_owned(),
            bytes,
        };
        let storage = self.clone();
        tokio::spawn(async move {
            let owner = storage.acquisition(&ingress.execution).await;
            let Some(owner) = owner else {
                return Err(error("log acquisition is missing"));
            };
            let mut acquisition = owner.lock().await;
            if acquisition.outputs_quiescent {
                return Err(error("log producers have already quiesced"));
            }
            let total = acquisition
                .native_outputs
                .values()
                .filter_map(|output| output.log.as_ref())
                .try_fold(0_u64, |total, log| total.checked_add(log.acknowledged))
                .and_then(|total| total.checked_add(ingress.bytes.len() as u64));
            if total.is_none_or(|total| total > 1024 * 1024 * 1024) {
                return Err(error("execution log reached its 1 GiB capture limit"));
            }
            let output = acquisition.native_outputs.get_mut(&ingress.path);
            let Some(output) = output else {
                return Err(error("log output is missing"));
            };
            let log = output.log.as_ref();
            let Some(log) = log else {
                return Err(error("output is not a log"));
            };
            if log.closed.is_some() {
                return Err(error("log output is closed"));
            }
            let acknowledged = log.acknowledged;
            let next = acknowledged.checked_add(ingress.bytes.len() as u64);
            let Some(next) = next else {
                return Err(error("native log prefix length overflows"));
            };
            let file = open_file(output).await?.into_std().await;
            let write = move || -> Result<()> {
                for attempt in 0..3 {
                    let opened = file.try_clone().map_err(error)?;
                    #[cfg(any(test, feature = "fault-injection"))]
                    let injected = {
                        let mut faults = storage.log_faults.lock().unwrap();
                        let injected = faults.get_mut(&ingress.execution).and_then(|(remaining, kind)| {
                            if *remaining == 0 {
                                return None;
                            }
                            *remaining -= 1;
                            Some(std::io::Error::new(*kind, "injected execution log write failure"))
                        });
                        if faults.get(&ingress.execution).is_some_and(|(remaining, _)| *remaining == 0) {
                            faults.remove(&ingress.execution);
                        }
                        injected
                    };
                    #[cfg(not(any(test, feature = "fault-injection")))]
                    let injected = None::<std::io::Error>;
                    let result = match injected {
                        Some(error) => Err(error),
                        None => ingress.write(opened, acknowledged),
                    };
                    match result {
                        Ok(()) => return Ok(()),
                        Err(source) if attempt < 2 && retryable(&source) => {
                            std::thread::sleep(std::time::Duration::from_millis(25 << attempt));
                        }
                        Err(source) => return Err(error(source)),
                    }
                }
                unreachable!("bounded log retry returns on its last attempt")
            };
            let write = tokio::task::spawn_blocking(write);
            write.await.map_err(error)??;
            // The Detached Append Owner Reaps Its Write Before Acknowledgement.
            output.log.as_mut().expect("log producer checked").acknowledged = next;
            Ok(())
        })
        .await
        .map_err(error)?
    }

    pub(crate) async fn close_output(self: &Arc<Self>, execution: &TestExecutionId, path: &str, interrupted: bool) -> Result<()> {
        validate_producer_path(path)?;
        let close = Close {
            execution: execution.clone(),
            path: path.to_owned(),
            interrupted,
        };
        let storage = self.clone();
        tokio::spawn(async move {
            let owner = storage.acquisition(&close.execution).await;
            let Some(owner) = owner else {
                return Err(error("log acquisition is missing"));
            };
            let mut acquisition = owner.lock().await;
            let session = acquisition.storage.clone();
            let output = acquisition.native_outputs.get_mut(&close.path);
            let Some(output) = output else {
                return Err(error("log output is missing"));
            };
            let log = output.log.as_mut();
            let Some(log) = log else {
                return Err(error("output is not a log"));
            };
            let state =
                *log.closed.get_or_insert(if close.interrupted { SavedStreamState::Interrupted } else { SavedStreamState::Finalized });
            if close.path.rsplit('/').next().is_some_and(|name| name.starts_with("stream.") && name.ends_with(".log")) {
                if output.production.is_some() {
                    super::follow::capture(&close.path, output, Some(state)).await?;
                } else {
                    if output.pending.is_none() {
                        let frozen = super::freeze(&session, output).await?;
                        let metadata = super::file_metadata(&close.execution, &close.path, Some(frozen.length()), Some(*frozen.sha256()))?;
                        output.pending = Some(super::FrozenFile::with_state(super::FrozenSource::Snapshot(frozen), metadata, Some(state)));
                    }
                    super::finish_snapshot(&session, &close.path, output).await?;
                }
                super::store_descriptor(&close.path, output, Some(state))?;
                output.captured = true;
                let descriptor = output.descriptor.as_ref().ok_or_else(|| error("closed log segment has no saved descriptor"))?;
                session.flush_until(&descriptor.file.position).await?;
            }
            Ok(())
        })
        .await
        .map_err(error)?
    }
}

#[cfg(any(test, feature = "fault-injection"))]
impl crate::RuntimeHandle {
    pub fn inject_log_write_faults(&self, execution: &TestExecutionId, count: usize, kind: std::io::ErrorKind) {
        let mut faults = self.execution_storage.log_faults.lock().unwrap();
        if count == 0 {
            faults.remove(execution);
        } else {
            faults.insert(execution.clone(), (count, kind));
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn retryable(source: &std::io::Error) -> bool {
    matches!(source.kind(), std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)
}

fn active_log(output: &NativeOutput) -> Result<()> {
    match &output.log {
        Some(LogPrefix { closed: None, .. }) => Ok(()),
        _ => Err(error("log path belongs to a different or closed producer")),
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn failed_append_retries_only_unacknowledged_bytes() {
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(b"saved.partial").unwrap();
        let append = Append {
            execution: TestExecutionId::from_seed("log.retry"),
            path: "workflow/stream.000000.log".into(),
            bytes: b"complete".to_vec(),
        };
        append.write(file.try_clone().unwrap(), 6).unwrap();
        append.write(file.try_clone().unwrap(), 6).unwrap();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"saved.complete");
        assert!(append.write(file, 1024).is_err());
    }

    #[test]
    fn permanent_log_failures_are_not_retried() {
        assert!(retryable(&std::io::Error::from(std::io::ErrorKind::Interrupted)));
        assert!(retryable(&std::io::Error::from(std::io::ErrorKind::TimedOut)));
        assert!(!retryable(&std::io::Error::from(std::io::ErrorKind::PermissionDenied)));
        assert!(!retryable(&std::io::Error::from(std::io::ErrorKind::InvalidData)));
        assert!(!retryable(&std::io::Error::from(std::io::ErrorKind::StorageFull)));
    }
}
