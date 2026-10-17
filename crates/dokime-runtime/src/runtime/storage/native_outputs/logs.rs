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
use std::sync::Arc;

// Third Party Imports
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

// Local Imports
use super::{
    LogPrefix, MAX_EXECUTION_FILE_SNAPSHOT_BYTES, NativeOutput, error, open_file,
    validate_producer_path,
};
use crate::{runtime::Result, runtime::storage::executions::ExecutionStorage};
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::execution::saved::SavedStreamState;

/* === Constants === */
const CHUNK_BYTES: usize = 64 * 1024;

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
impl ExecutionStorage {
    pub(crate) async fn reserve_output(
        self: &Arc<Self>,
        execution: &TestExecutionId,
        path: &str,
    ) -> Result<()> {
        validate_producer_path(path)?;
        let storage = self.clone();
        let execution = execution.clone();
        let path = path.to_owned();
        tokio::spawn(async move {
            let owner = storage
                .acquisition(&execution)
                .await
                .ok_or_else(|| error("log execution has no admitted producer owner"))?;
            let mut acquisition = owner.lock().await;
            if acquisition.finish_requested || acquisition.outputs_sealed {
                return Err(error("log producer admission is closed"));
            }
            if let Some(output) = acquisition.native_outputs.get(&path) {
                return if output.log.as_ref().is_some_and(|log| log.closed.is_none()) {
                    Ok(())
                } else {
                    Err(error("log path belongs to a different or closed producer"))
                };
            }
            let mut output = NativeOutput::new(&path)?;
            output.log = Some(LogPrefix {
                acknowledged: 0,
                closed: None,
            });
            acquisition.native_outputs.insert(path, output);
            Ok(())
        })
        .await
        .map_err(error)?
    }

    pub(crate) async fn log_prefix(&self, execution: &TestExecutionId, path: &str) -> Result<u64> {
        let owner = self
            .acquisition(execution)
            .await
            .ok_or_else(|| error("log acquisition is missing"))?;
        let acquisition = owner.lock().await;
        let output = acquisition
            .native_outputs
            .get(path)
            .ok_or_else(|| error("log output is missing"))?;
        let log = output
            .log
            .as_ref()
            .ok_or_else(|| error("output is not a log"))?;
        if log.closed.is_some() {
            return Err(error("log output is closed"));
        }
        Ok(log.acknowledged)
    }

    pub(crate) async fn append_output(
        self: &Arc<Self>,
        execution: &TestExecutionId,
        path: &str,
        bytes: &[u8],
    ) -> Result<()> {
        validate_producer_path(path)?;
        if bytes.len() > CHUNK_BYTES {
            return Err(error("native log append exceeds 64 KiB"));
        }
        let ingress = Append {
            execution: execution.clone(),
            path: path.to_owned(),
            bytes: bytes.to_vec(),
        };
        let storage = self.clone();
        tokio::spawn(async move {
            let owner = storage
                .acquisition(&ingress.execution)
                .await
                .ok_or_else(|| error("log acquisition is missing"))?;
            let mut acquisition = owner.lock().await;
            if acquisition.outputs_quiescent {
                return Err(error("log producers have already quiesced"));
            }
            let output = acquisition
                .native_outputs
                .get_mut(&ingress.path)
                .ok_or_else(|| error("log output is missing"))?;
            let log = output
                .log
                .as_ref()
                .ok_or_else(|| error("output is not a log"))?;
            if log.closed.is_some() {
                return Err(error("log output is closed"));
            }
            let acknowledged = log.acknowledged;
            let next = acknowledged
                .checked_add(ingress.bytes.len() as u64)
                .ok_or_else(|| error("native log prefix length overflows"))?;
            if next > MAX_EXECUTION_FILE_SNAPSHOT_BYTES as u64 {
                return Err(crate::runtime::Error::ExecutionFileTooLarge);
            }
            let mut file = open_file(output).await?;
            // Remove Only A Failed Private Append; Native Immutable Files Are Untouched.
            if file.metadata().await.map_err(error)?.len() < acknowledged {
                return Err(error("native log lost its acknowledged prefix"));
            }
            file.set_len(acknowledged).await.map_err(error)?;
            file.seek(std::io::SeekFrom::Start(acknowledged))
                .await
                .map_err(error)?;
            file.write_all(&ingress.bytes).await.map_err(error)?;
            // Complete Tokio's Pending Write Before Acknowledging The Prefix.
            file.flush().await.map_err(error)?;
            output
                .log
                .as_mut()
                .expect("log producer checked")
                .acknowledged = next;
            Ok(())
        })
        .await
        .map_err(error)?
    }

    pub(crate) async fn close_output(
        self: &Arc<Self>,
        execution: &TestExecutionId,
        path: &str,
        interrupted: bool,
    ) -> Result<()> {
        validate_producer_path(path)?;
        let close = Close {
            execution: execution.clone(),
            path: path.to_owned(),
            interrupted,
        };
        let storage = self.clone();
        tokio::spawn(async move {
            let owner = storage
                .acquisition(&close.execution)
                .await
                .ok_or_else(|| error("log acquisition is missing"))?;
            let mut acquisition = owner.lock().await;
            let output = acquisition
                .native_outputs
                .get_mut(&close.path)
                .ok_or_else(|| error("log output is missing"))?;
            let log = output
                .log
                .as_mut()
                .ok_or_else(|| error("output is not a log"))?;
            log.closed.get_or_insert(if close.interrupted {
                SavedStreamState::Interrupted
            } else {
                SavedStreamState::Finalized
            });
            Ok(())
        })
        .await
        .map_err(error)?
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
