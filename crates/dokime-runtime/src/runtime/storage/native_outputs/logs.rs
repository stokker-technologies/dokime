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
    fn write(self, mut file: std::fs::File, acknowledged: u64) -> Result<()> {
        if file.metadata().map_err(error)?.len() < acknowledged {
            return Err(error("native log lost its acknowledged prefix"));
        }
        // Only A Failed Private Append Is Removed; Immutable Files Are Untouched.
        file.set_len(acknowledged).map_err(error)?;
        file.seek(std::io::SeekFrom::Start(acknowledged)).map_err(error)?;
        file.write_all(&self.bytes).map_err(error)?;
        file.flush().map_err(error)?;

        drop(self.execution);
        drop(self.path);
        drop(self.bytes);
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
        let ingress = Append { execution: execution.clone(), path: path.to_owned(), bytes };
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
            let write = move || ingress.write(file, acknowledged);
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
        let close = Close { execution: execution.clone(), path: path.to_owned(), interrupted };
        let storage = self.clone();
        tokio::spawn(async move {
            let owner = storage.acquisition(&close.execution).await;
            let Some(owner) = owner else {
                return Err(error("log acquisition is missing"));
            };
            let mut acquisition = owner.lock().await;
            let output = acquisition.native_outputs.get_mut(&close.path);
            let Some(output) = output else {
                return Err(error("log output is missing"));
            };
            let log = output.log.as_mut();
            let Some(log) = log else {
                return Err(error("output is not a log"));
            };
            log.closed.get_or_insert(if close.interrupted { SavedStreamState::Interrupted } else { SavedStreamState::Finalized });
            Ok(())
        })
        .await
        .map_err(error)?
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn active_log(output: &NativeOutput) -> Result<()> {
    match &output.log {
        Some(LogPrefix { closed: None, .. }) => Ok(()),
        _ => Err(error("log path belongs to a different or closed producer")),
    }
}

/* === Tests === */
