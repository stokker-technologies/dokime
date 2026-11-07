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
use super::ExecutionFileStream;
use super::saved::SavedOutputs;
use crate::handle::RuntimeHandle;
use crate::{
    execution_runtime::{
        authoring::ExecutionFileTarget,
        outputs::{ExecutionFileContent, ExecutionFileEntry, validate_relative_path},
    },
    runtime::Result,
};
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn update_plot_settings(
        &self, cycle_id: &TestCycleId, execution_id: &TestExecutionId, document_id: &dokime_domain::model::identity::DocumentId,
        plot_id: &dokime_domain::model::identity::Identifier, display_points: Option<u32>,
        downsample: crate::document::DownsampleTechnique,
    ) -> Result<()> {
        if display_points.is_some_and(|points| u64::from(points) > 100000_u64) {
            return Err(crate::runtime::Error::Operation("plot points exceed their configured maximum".into()));
        }
        let update = crate::transactions::UpdatePlotSettings {
            cycle_id: cycle_id.clone(),
            execution_id: execution_id.clone(),
            document_id: document_id.clone(),
            plot_id: plot_id.clone(),
            display_points,
            downsample,
            occurred_at: crate::model::time::now_utc(),
        };
        update.validate(&self.state_read())?;
        self.commit(update).await.map_err(|error| crate::runtime::Error::conflict("plot_settings_rejected", error.to_string()))?;
        Ok(())
    }

    pub async fn execution_files(&self, test_execution_id: &TestExecutionId) -> Result<Vec<ExecutionFileEntry>> {
        if let Some(saved) = SavedOutputs::open(self.storage(), test_execution_id, None).await? {
            return saved.entries();
        }
        self.execution_storage.native_entries(test_execution_id).await
    }

    pub async fn execution_file_content(
        &self, id: &TestExecutionId, path: String, offset: u64, max_bytes: usize,
    ) -> Result<Option<harmos::storage::ReadBatch<ExecutionFileContent>>> {
        if validate_relative_path(&path).is_err() {
            return Ok(None);
        }
        let maximum = NonZeroUsize::new(max_bytes.max(1)).unwrap();
        if let Some(content) = self.execution_storage.native_read(id, &path, offset, maximum.get()).await? {
            return Ok(Some(content));
        }
        let Some(saved) = SavedOutputs::open(self.storage(), id, None).await? else {
            return Ok(None);
        };
        let Some(file) = saved.file(&path)? else {
            return Ok(None);
        };
        Ok(Some(file.window(offset, maximum).await?))
    }

    pub async fn execution_file_snapshot(
        &self, test_execution_id: &TestExecutionId, relative_path: String,
    ) -> Result<Option<harmos::storage::ReadBatch<ExecutionFileContent>>> {
        if validate_relative_path(&relative_path).is_err() {
            return Ok(None);
        }
        if let Some(content) = self.execution_storage.native_snapshot(test_execution_id, &relative_path).await? {
            return Ok(Some(content));
        }
        let Some(saved) = SavedOutputs::open(self.storage(), test_execution_id, None).await? else {
            return Ok(None);
        };
        let Some(file) = saved.file(&relative_path)? else {
            return Ok(None);
        };
        Ok(Some(file.snapshot().await?))
    }

    pub async fn open_execution_file_stream(
        &self, test_execution_id: &TestExecutionId, relative_path: String,
    ) -> Result<Option<ExecutionFileStream>> {
        if validate_relative_path(&relative_path).is_err() {
            return Ok(None);
        }
        if let Some(saved) = SavedOutputs::open(self.storage(), test_execution_id, None).await? {
            return saved.file(&relative_path)?.map(ExecutionFileStream::saved).transpose();
        }
        self.execution_storage.native_stream(test_execution_id, &relative_path).await?.map(ExecutionFileStream::live).transpose()
    }

    pub async fn execution_file_target(
        &self, test_execution_id: &TestExecutionId, relative_path: String, text_hint: bool, streamed: bool,
    ) -> Result<Option<ExecutionFileTarget>> {
        crate::execution_runtime::outputs::validate_relative_path(&relative_path)?;
        let selected = {
            let state = self.state_read();
            state
                .executions
                .get(test_execution_id)
                .filter(|execution| matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running))
                .and_then(|execution| state.cycle_view(&execution.test_cycle_id).map(|cycle| (cycle, execution.clone())))
        };
        let Some((cycle, execution)) = selected else {
            return Ok(None);
        };
        let storage = self.execution_storage.clone();
        tokio::spawn(async move {
            storage.prepare_execution(&cycle, &execution).await?;
            storage.native_target(execution.test_execution_id, relative_path, text_hint, streamed).await.map(Some)
        })
        .await
        .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?
    }

    pub(crate) async fn abandon_execution_artifacts(&self, test_execution_id: &TestExecutionId) -> Result<()> {
        self.stop_execution_log_tails(Some(test_execution_id)).await?;
        self.execution_storage.retire_outputs(test_execution_id).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
