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
use std::{ops::Deref, sync::Arc};

// Third Party Imports
use serde::{Serialize, Serializer};

// Local Imports
use super::{RuntimeExecutionFileFrame, RuntimeExecutionFileRecord};
use crate::{
    execution_runtime::outputs::ExecutionFileEntry,
    queries::{RuntimeResolution, RuntimeWindow},
};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

static EMPTY: RuntimeExecutionFileFrame =
    RuntimeExecutionFileFrame { files: Vec::new(), total_files: 0, returned_files: 0, truncated: false, error: None };

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct RuntimeExecutionFileRead {
    value: Option<Arc<RuntimeExecutionFileFrame>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeExecutionFileRead {
    pub(crate) fn unselected() -> Self {
        Self { value: None }
    }

    pub(crate) fn from_frame(value: RuntimeExecutionFileFrame) -> Self {
        Self { value: Some(Arc::new(value)) }
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

/* === Public Functions === */

pub(crate) fn execution_file_frame(
    execution: &TestExecutionId, files: &[ExecutionFileEntry], window: &RuntimeWindow, resolution: &RuntimeResolution,
) -> RuntimeExecutionFileFrame {
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
    let mut records = Vec::with_capacity(returned);
    for file in selected(files, window, &indices) {
        records.push(RuntimeExecutionFileRecord {
            artifact_id: file.artifact_id.clone(),
            scope: RecordScope::default().with_test_execution(execution.clone()),
            filename: file.relative_path.clone(),
            content_type: file.content_type.clone(),
            content_kind: file.content_kind,
            stream_state: file.stream_state,
            size_bytes: file.size_bytes,
            updated_at: file.updated_at.clone(),
        });
    }
    RuntimeExecutionFileFrame { files: records, total_files: total, returned_files: returned, truncated: returned < total, error: None }
}

/* === Private Functions === */

fn windowed<'a>(files: &'a [ExecutionFileEntry], window: &'a RuntimeWindow) -> impl Iterator<Item = &'a ExecutionFileEntry> {
    files
        .iter()
        .enumerate()
        .filter(move |(index, file)| match window {
            RuntimeWindow::Full | RuntimeWindow::Since { .. } => true,
            RuntimeWindow::LastSamples { count } => *index >= files.len().saturating_sub(*count),
            RuntimeWindow::TimeRange { start, end } => {
                start.millis_until(&file.updated_at).is_some() && file.updated_at.millis_until(end).is_some()
            }
            RuntimeWindow::Tail { duration } => files
                .last()
                .is_some_and(|last| file.updated_at.millis_until(&last.updated_at).is_some_and(|elapsed| elapsed.0 <= duration.0)),
        })
        .map(|(_, file)| file)
}

fn selected<'a>(
    files: &'a [ExecutionFileEntry], window: &'a RuntimeWindow, indices: &'a [usize],
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

/* === Tests === */
