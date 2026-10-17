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
    io::{Read, Seek},
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

// Third Party Imports
use tokio::task::JoinHandle;

// Local Imports
use super::error;
use crate::{
    execution_runtime::outputs::{ExecutionFileContent, ExecutionFileContentKind, ExecutionFileStreamState},
    runtime::Error,
    runtime::Result,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Reader {
    source: Arc<Source>,
    offset: u64,
    pending: Option<Pending>,
}

struct Source {
    file: Mutex<std::fs::File>,
    metadata: ExecutionFileContent,
}

// A Cancelled Caller Retries The Same Accepted Read Rather Than Starting Another.
struct Pending {
    maximum: NonZeroUsize,
    task: JoinHandle<Result<ExecutionFileContent>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Reader {
    pub(super) fn new(file: std::fs::File, path: &str, observed_size: u64, stream_state: ExecutionFileStreamState) -> Self {
        let content_type = super::media_type_for_path(path).map(str::to_owned);
        let metadata = ExecutionFileContent {
            relative_path: path.to_owned(),
            content_kind: ExecutionFileContentKind::from_metadata(path, content_type.as_deref()),
            content_type,
            stream_state,
            offset: 0,
            size_bytes: observed_size,
            bytes: Vec::new(),
        };
        Self { source: Arc::new(Source { file: Mutex::new(file), metadata }), offset: 0, pending: None }
    }

    pub(crate) fn metadata(&self) -> &ExecutionFileContent {
        &self.source.metadata
    }

    pub(crate) async fn next_chunk(&mut self, maximum: NonZeroUsize) -> Result<Option<ExecutionFileContent>> {
        if let Some(pending) = &self.pending {
            if pending.maximum != maximum {
                return Err(harmos::Error::InvalidInput { field: "native stream chunk", reason: "pending read has another maximum" }.into());
            }
        } else {
            if self.offset == self.source.metadata.size_bytes {
                return Ok(None);
            }
            let source = self.source.clone();
            let offset = self.offset;
            let length = (source.metadata.size_bytes - offset).min(maximum.get() as u64) as usize;
            let task = tokio::task::spawn_blocking(move || {
                let mut bytes = vec![0; length];
                let mut file = source.file.lock().map_err(|_| error("native stream file lock poisoned"))?;
                file.seek(std::io::SeekFrom::Start(offset)).map_err(error)?;
                file.read_exact(&mut bytes).map_err(error)?;
                drop(file);
                Ok(ExecutionFileContent { offset, bytes, ..source.metadata.clone() })
            });
            self.pending = Some(Pending { maximum, task });
        }
        let pending = self.pending.as_mut().expect("accepted native read retained");
        let result = (&mut pending.task).await;
        self.pending = None;
        let content = result.map_err(|source| Error::Join { source })??;
        self.offset = content.offset.checked_add(content.bytes.len() as u64).ok_or_else(|| error("native stream offset overflows"))?;
        Ok(Some(content))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
