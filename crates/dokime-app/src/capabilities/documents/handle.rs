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
use harmos::{
    MemorySize,
    storage::{QueryOptions, QueryValue},
};

// Local Imports
use super::ExecutionFileStream;
use super::saved::{SavedOutputFile, SavedOutputs};
use crate::document::{DocumentPayload, DocumentSelection};
use crate::execution::ExecutionStatus;
use crate::handle::RuntimeHandle;
use crate::{
    Result,
    execution_runtime::{
        authoring::{ExecutionFileTarget, FileRegexMatch},
        outputs::{
            ExecutionFileContent, ExecutionFileEntry, regex_matches, validate_relative_path,
        },
    },
};
use dokime::core::{TestCycleId, TestExecutionId};

/* === Constants === */

/* === Macros === */
/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
enum ExecutionWindow {
    Live(QueryValue<ExecutionFileContent>),
    Saved {
        file: Box<SavedOutputFile>,
        content: QueryValue<ExecutionFileContent>,
    },
}

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn update_plot_settings(
        &self,
        cycle_id: &TestCycleId,
        execution_id: &TestExecutionId,
        document_id: &dokime::core::DocumentId,
        plot_id: &dokime::core::Identifier,
        display_points: Option<u32>,
        downsample: dokime::document::DownsampleTechnique,
    ) -> Result<()> {
        let update = crate::transactions::UpdatePlotSettings {
            cycle_id: cycle_id.clone(),
            execution_id: execution_id.clone(),
            document_id: document_id.clone(),
            plot_id: plot_id.clone(),
            display_points,
            downsample,
            occurred_at: dokime::core::Timestamp::now_utc(),
        };
        update.validate(&self.state_read())?;
        self.commit(update)
            .await
            .map_err(|error| crate::Error::conflict("plot_settings_rejected", error.to_string()))?;
        Ok(())
    }

    pub async fn resolve_cycle_document(
        &self,
        test_cycle_id: &TestCycleId,
        document_id: &dokime::core::DocumentId,
        mut selection: DocumentSelection,
    ) -> Result<Option<DocumentPayload>> {
        if selection.scope.test_execution_id.is_none() {
            return Err(crate::Error::Operation(
                "document selection requires an execution".into(),
            ));
        }
        if selection.scope.test_cycle_id.is_none() {
            selection.scope.test_cycle_id = Some(test_cycle_id.clone());
        }
        let telemetry = self
            .collect_telemetry(&crate::evidence::TelemetrySelection {
                scope: selection.evidence_scope(),
                ..Default::default()
            })
            .await;
        let snapshot = self.capture_state().await?;
        let records = self.records_at(&snapshot).await?;
        let artifacts = self.artifact_records_now().await?.to_vec();
        let state = snapshot.state();
        let data = crate::documents::RuntimeDocumentData {
            records,
            telemetry,
            artifacts,
        };
        let Some(cycle) = state
            .cycle_view(test_cycle_id)
            .map(crate::definition::TestCycleInstance::new)
        else {
            return Ok(None);
        };

        crate::documents::resolve_cycle_document(
            &state.suites,
            &cycle,
            document_id,
            selection,
            data,
        )
    }

    pub async fn execution_files(
        &self,
        test_execution_id: &TestExecutionId,
    ) -> Result<QueryValue<Vec<ExecutionFileEntry>>> {
        if let Some(mut saved) = SavedOutputs::open(
            self.storage(),
            test_execution_id,
            None,
            QueryOptions::default(),
            self.config().storage_reads.decoded_bytes,
        )
        .await?
        {
            return saved.entries(self.config().storage_reads.decoded_bytes);
        }
        self.execution_storage
            .native_entries(test_execution_id)
            .await
    }

    pub async fn execution_files_projected<U: MemorySize>(
        &self,
        test_execution_id: &TestExecutionId,
        map: impl FnOnce(&Vec<ExecutionFileEntry>, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        let maximum = self.config().storage_reads.decoded_bytes;
        if let Some(mut saved) = SavedOutputs::open(
            self.storage(),
            test_execution_id,
            None,
            QueryOptions::default(),
            maximum,
        )
        .await?
        {
            let entries = saved.entries(maximum)?;
            return saved.project(&entries, map);
        }
        let entries = self
            .execution_storage
            .native_entries(test_execution_id)
            .await?;
        let computation = entries.computation()?;
        Ok(computation.project(&entries, maximum, map)?)
    }

    pub async fn execution_file_content(
        &self,
        test_execution_id: &TestExecutionId,
        relative_path: String,
        offset: u64,
        max_bytes: usize,
    ) -> Result<Option<QueryValue<ExecutionFileContent>>> {
        let window = self
            .execution_window(test_execution_id, &relative_path, offset, max_bytes)
            .await?;
        Ok(window.map(|window| match window {
            ExecutionWindow::Live(content) | ExecutionWindow::Saved { content, .. } => content,
        }))
    }

    pub async fn execution_file_content_projected<U: MemorySize>(
        &self,
        test_execution_id: &TestExecutionId,
        relative_path: String,
        offset: u64,
        max_bytes: usize,
        projection: impl FnOnce(&ExecutionFileContent) -> harmos::Result<NonZeroUsize>,
        map: impl FnOnce(&ExecutionFileContent, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<Option<QueryValue<U>>> {
        let Some(window) = self
            .execution_window(test_execution_id, &relative_path, offset, max_bytes)
            .await?
        else {
            return Ok(None);
        };
        let output = match window {
            ExecutionWindow::Live(content) => {
                let computation = content.computation()?;
                let maximum = projection(&content)?;
                computation.project(&content, maximum, map)?
            }
            ExecutionWindow::Saved { mut file, content } => {
                file.project(&content, projection(&content)?, map)?
            }
        };
        Ok(Some(output))
    }

    async fn execution_window(
        &self,
        id: &TestExecutionId,
        path: &str,
        offset: u64,
        max_bytes: usize,
    ) -> Result<Option<ExecutionWindow>> {
        if validate_relative_path(path).is_err() {
            return Ok(None);
        }
        let maximum = NonZeroUsize::new(max_bytes.clamp(
            1,
            crate::execution_runtime::outputs::MAX_ARTIFACT_READ_BYTES,
        ))
        .unwrap();
        if let Some(content) = self
            .execution_storage
            .native_read(id, path, offset, maximum.get())
            .await?
        {
            return Ok(Some(ExecutionWindow::Live(content)));
        }
        let Some(saved) = SavedOutputs::open(
            self.storage(),
            id,
            None,
            QueryOptions::default(),
            self.config().storage_reads.decoded_bytes,
        )
        .await?
        else {
            return Ok(None);
        };
        let Some(mut file) = saved.file(path).await? else {
            return Ok(None);
        };
        let projection = file.window_memory(offset, maximum)?;
        let content = file.window(offset, maximum, projection).await?;
        Ok(Some(ExecutionWindow::Saved {
            file: Box::new(file),
            content,
        }))
    }

    pub async fn execution_file_snapshot(
        &self,
        test_execution_id: &TestExecutionId,
        relative_path: String,
    ) -> Result<Option<QueryValue<ExecutionFileContent>>> {
        if validate_relative_path(&relative_path).is_err() {
            return Ok(None);
        }
        if let Some(content) = self
            .execution_storage
            .native_snapshot(test_execution_id, &relative_path)
            .await?
        {
            return Ok(Some(content));
        }
        let Some(saved) = SavedOutputs::open(
            self.storage(),
            test_execution_id,
            None,
            QueryOptions::default(),
            self.config().storage_reads.decoded_bytes,
        )
        .await?
        else {
            return Ok(None);
        };
        let Some(mut file) = saved.file(&relative_path).await? else {
            return Ok(None);
        };
        let maximum =
            NonZeroUsize::new(crate::execution_runtime::outputs::MAX_EXECUTION_FILE_SNAPSHOT_BYTES)
                .unwrap();
        Ok(Some(file.snapshot(maximum, true).await?))
    }

    pub async fn open_execution_file_stream(
        &self,
        test_execution_id: &TestExecutionId,
        relative_path: String,
    ) -> Result<Option<ExecutionFileStream>> {
        if validate_relative_path(&relative_path).is_err() {
            return Ok(None);
        }
        if let Some(saved) = SavedOutputs::open(
            self.storage(),
            test_execution_id,
            None,
            QueryOptions::default(),
            self.config().storage_reads.decoded_bytes,
        )
        .await?
        {
            return saved
                .file(&relative_path)
                .await?
                .map(ExecutionFileStream::saved)
                .transpose();
        }
        self.execution_storage
            .native_stream(test_execution_id, &relative_path)
            .await?
            .map(ExecutionFileStream::live)
            .transpose()
    }

    pub async fn execution_file_target(
        &self,
        test_execution_id: &TestExecutionId,
        relative_path: String,
        text_hint: bool,
        streamed: bool,
    ) -> Result<Option<ExecutionFileTarget>> {
        crate::execution_runtime::outputs::validate_relative_path(&relative_path)?;
        let selected = {
            let state = self.state_read();
            state
                .executions
                .get(test_execution_id)
                .filter(|execution| {
                    matches!(
                        execution.status,
                        ExecutionStatus::Pending | ExecutionStatus::Running
                    )
                })
                .and_then(|execution| {
                    state
                        .cycle_view(&execution.test_cycle_id)
                        .map(|cycle| (cycle, execution.clone()))
                })
        };
        let Some((cycle, execution)) = selected else {
            return Ok(None);
        };
        let storage = self.execution_storage.clone();
        tokio::spawn(async move {
            storage.prepare_execution(&cycle, &execution).await?;
            storage
                .native_target(
                    execution.test_execution_id,
                    relative_path,
                    text_hint,
                    streamed,
                )
                .await
                .map(Some)
        })
        .await
        .map_err(|error| crate::Error::Operation(error.to_string()))?
    }

    pub async fn extract_execution_file_regex(
        &self,
        test_execution_id: &TestExecutionId,
        relative_path: String,
        pattern: String,
        first_only: bool,
    ) -> Result<Option<Vec<FileRegexMatch>>> {
        let Some(content) = self
            .execution_file_content_up_to(test_execution_id, relative_path, 2 * 1024 * 1024)
            .await?
        else {
            return Ok(None);
        };
        regex_matches(&content, &pattern, first_only).map(Some)
    }

    async fn execution_file_content_up_to(
        &self,
        test_execution_id: &TestExecutionId,
        relative_path: String,
        limit: usize,
    ) -> Result<Option<QueryValue<ExecutionFileContent>>> {
        if validate_relative_path(&relative_path).is_err() {
            return Ok(None);
        }
        let maximum = NonZeroUsize::new(limit).ok_or_else(|| {
            crate::Error::Operation("execution output prefix must be nonempty".into())
        })?;
        if let Some(content) = self
            .execution_storage
            .native_read_up_to(test_execution_id, &relative_path, maximum.get())
            .await?
        {
            return Ok(Some(content));
        }
        let Some(saved) = SavedOutputs::open(
            self.storage(),
            test_execution_id,
            None,
            QueryOptions::default(),
            self.config().storage_reads.decoded_bytes,
        )
        .await?
        else {
            return Ok(None);
        };
        let Some(mut file) = saved.file(&relative_path).await? else {
            return Ok(None);
        };
        Ok(Some(file.snapshot(maximum, false).await?))
    }

    pub(crate) async fn abandon_execution_artifacts(
        &self,
        test_execution_id: &TestExecutionId,
    ) -> Result<()> {
        self.stop_execution_log_tails(Some(test_execution_id))
            .await?;
        self.execution_storage
            .retire_outputs(test_execution_id)
            .await
    }
}

/* === Public Functions === */

/* === Private Functions === */
/* === Functions === */

/* === Tests === */
