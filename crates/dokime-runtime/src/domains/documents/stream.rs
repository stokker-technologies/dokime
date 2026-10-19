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
use std::num::NonZeroUsize;

// Third Party Imports

// Local Imports
use super::saved::SavedOutputFile;
use crate::{ExecutionFileContent, runtime::Result, runtime::storage::native_outputs::reader::Reader};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct ExecutionFileStream {
    source: Source,
    metadata: ExecutionFileContent,
    offset: u64,
}

/* === Enums === */
enum Source {
    Live(Reader),
    Saved(Box<SavedOutputFile>),
}

/* === Traits === */

/* === Implementations === */
impl ExecutionFileStream {
    pub(crate) fn live(reader: Reader) -> Result<Self> {
        let metadata = ExecutionFileContent { offset: 0, bytes: Vec::new(), ..reader.metadata().clone() };
        Ok(Self { source: Source::Live(reader), metadata, offset: 0 })
    }

    pub(crate) fn saved(file: SavedOutputFile) -> Result<Self> {
        let metadata = file.metadata();
        Ok(Self { source: Source::Saved(Box::new(file)), metadata, offset: 0 })
    }

    pub fn metadata(&self) -> &ExecutionFileContent {
        &self.metadata
    }

    pub async fn next_chunk(&mut self, maximum: NonZeroUsize) -> Result<Option<ExecutionFileContent>> {
        let content = match &mut self.source {
            Source::Live(reader) => reader.next_chunk(maximum).await?,
            Source::Saved(file) => {
                let content = file.window(self.offset, maximum).await?;
                (!content.bytes.is_empty()).then_some(content)
            }
        };
        let Some(content) = content else {
            if self.offset != self.metadata.size_bytes {
                return Err(invalid("source ended before observed prefix").into());
            }
            return Ok(None);
        };
        if content.offset != self.offset
            || content.bytes.is_empty()
            || content.size_bytes != self.metadata.size_bytes
            || end(&content)? > self.metadata.size_bytes
        {
            return Err(invalid("original chunk differs from selected prefix").into());
        }
        self.offset = end(&content)?;
        Ok(Some(content))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn end(content: &ExecutionFileContent) -> harmos::Result<u64> {
    content.offset.checked_add(content.bytes.len() as u64).ok_or_else(|| invalid("chunk offset overflows"))
}

fn invalid(reason: &'static str) -> harmos::Error {
    harmos::Error::InvalidInput { field: "execution stream", reason }
}

/* === Tests === */
