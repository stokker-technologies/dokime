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
use std::{num::NonZeroUsize, ops::Deref};

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{QueryValue, SharedQueryValue},
};
use serde::{Serialize, Serializer};

// Local Imports
use super::{RuntimeExecutionFileFrame, RuntimeExecutionFileRecord};
use crate::{
    execution_runtime::outputs::ExecutionFileEntry,
    queries::{RuntimeResolution, RuntimeWindow},
};
use dokime::core::{RecordScope, TestExecutionId, Timestamp};

/* === Constants === */
static EMPTY: RuntimeExecutionFileFrame = RuntimeExecutionFileFrame {
    files: Vec::new(),
    total_files: 0,
    returned_files: 0,
    truncated: false,
    error: None,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug)]
pub struct RuntimeExecutionFileRead {
    value: Option<SharedQueryValue<RuntimeExecutionFileFrame>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeExecutionFileRead {
    pub(crate) fn unselected() -> Self {
        Self { value: None }
    }

    pub(crate) fn from_query(value: QueryValue<RuntimeExecutionFileFrame>) -> harmos::Result<Self> {
        Ok(Self {
            value: Some(value.try_into_shared()?),
        })
    }
}

impl Deref for RuntimeExecutionFileRead {
    type Target = RuntimeExecutionFileFrame;
    fn deref(&self) -> &Self::Target {
        self.value.as_deref().unwrap_or(&EMPTY)
    }
}

impl PartialEq for RuntimeExecutionFileRead {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl Eq for RuntimeExecutionFileRead {}

impl Serialize for RuntimeExecutionFileRead {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.deref().serialize(serializer)
    }
}

impl MemorySize for RuntimeExecutionFileRecord {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.artifact_id.capacity())
            .saturating_add(self.filename.capacity())
            .saturating_add(self.content_type.as_ref().map_or(0, String::capacity))
            .saturating_add(
                self.scope
                    .memory_size()
                    .saturating_sub(std::mem::size_of_val(&self.scope)),
            )
            .saturating_add(
                self.updated_at
                    .memory_size()
                    .saturating_sub(std::mem::size_of_val(&self.updated_at)),
            )
    }
}

impl MemorySize for RuntimeExecutionFileFrame {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.files
                    .memory_size()
                    .saturating_sub(std::mem::size_of_val(&self.files)),
            )
            .saturating_add(self.error.as_ref().map_or(0, String::capacity))
    }
}

/* === Public Functions === */
pub(crate) fn execution_file_frame(
    execution: &TestExecutionId,
    files: &[ExecutionFileEntry],
    window: &RuntimeWindow,
    resolution: &RuntimeResolution,
    allowance: NonZeroUsize,
) -> harmos::Result<RuntimeExecutionFileFrame> {
    let total = windowed(files, window).count();
    let count = match resolution {
        RuntimeResolution::Raw => total,
        RuntimeResolution::Summary => total.min(1),
        RuntimeResolution::Samples { count } => *count,
        RuntimeResolution::Pixels { width } => *width as usize,
    };
    // Summary Keeps The Latest File; Reductions Use Harmos `decimate`.
    let indices = match resolution {
        RuntimeResolution::Summary => total.checked_sub(1).into_iter().collect(),
        _ if total <= count => (0..total).collect(),
        _ => harmos::signal::decimate(total, count),
    };
    let returned = indices.len();
    let selected = || selected(files, window, &indices);
    let mut bytes = std::mem::size_of::<RuntimeExecutionFileFrame>();
    for file in selected() {
        for field in [
            std::mem::size_of::<RuntimeExecutionFileRecord>(),
            execution.as_str().len(),
            file.artifact_id.len(),
            file.relative_path.len(),
            file.content_type.as_ref().map_or(0, String::len),
            file.updated_at.as_str().len(),
        ] {
            bytes = bytes.checked_add(field).ok_or_else(exhausted)?;
        }
    }
    if bytes > allowance.get() {
        return Err(exhausted());
    }

    let mut records = Vec::new();
    records
        .try_reserve_exact(returned)
        .map_err(|_| exhausted())?;
    if records.capacity() > returned {
        return Err(exhausted());
    }
    for file in selected() {
        let id = TestExecutionId::from_uuid(text(execution.as_str())?).map_err(|_| {
            harmos::Error::InvalidInput {
                field: "execution files frame",
                reason: "invalid original execution identity",
            }
        })?;
        records.push(RuntimeExecutionFileRecord {
            artifact_id: text(&file.artifact_id)?,
            scope: RecordScope::default().with_test_execution(id),
            filename: text(&file.relative_path)?,
            content_type: file.content_type.as_deref().map(text).transpose()?,
            content_kind: file.content_kind,
            stream_state: file.stream_state,
            size_bytes: file.size_bytes,
            updated_at: Timestamp::new(text(file.updated_at.as_str())?),
        });
    }
    Ok(RuntimeExecutionFileFrame {
        files: records,
        total_files: total,
        returned_files: returned,
        truncated: returned < total,
        error: None,
    })
}

/* === Private Functions === */
fn windowed<'a>(
    files: &'a [ExecutionFileEntry],
    window: &'a RuntimeWindow,
) -> impl Iterator<Item = &'a ExecutionFileEntry> {
    files
        .iter()
        .enumerate()
        .filter(move |(index, file)| match window {
            RuntimeWindow::Full | RuntimeWindow::Since { .. } => true,
            RuntimeWindow::LastSamples { count } => *index >= files.len().saturating_sub(*count),
            RuntimeWindow::TimeRange { start, end } => {
                start.millis_until(&file.updated_at).is_some()
                    && file.updated_at.millis_until(end).is_some()
            }
            RuntimeWindow::Tail { duration } => files.last().is_some_and(|last| {
                file.updated_at
                    .millis_until(&last.updated_at)
                    .is_some_and(|elapsed| elapsed.0 <= duration.0)
            }),
        })
        .map(|(_, file)| file)
}

fn selected<'a>(
    files: &'a [ExecutionFileEntry],
    window: &'a RuntimeWindow,
    indices: &'a [usize],
) -> impl Iterator<Item = &'a ExecutionFileEntry> {
    let mut selected = indices.iter().copied().peekable();
    windowed(files, window)
        .enumerate()
        .filter(move |(index, _)| {
            if selected.peek() != Some(index) {
                return false;
            }
            selected.next();
            true
        })
        .map(|(_, file)| file)
}

fn text(source: &str) -> harmos::Result<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(source.len())
        .map_err(|_| exhausted())?;
    if value.capacity() > source.len() {
        return Err(exhausted());
    }
    value.push_str(source);
    Ok(value)
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "execution files frame projection",
    }
}

/* === Tests === */
