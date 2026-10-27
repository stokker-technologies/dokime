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

// Third Party Imports

// Local Imports
use crate::RuntimeHandle;
use crate::queries::{
    RuntimeCursor, RuntimeListFrame, RuntimeResolution, RuntimeStateSnapshot, RuntimeViewFrame, RuntimeViewPayload, RuntimeViewQuery,
    RuntimeViewTopic,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn query_runtime_view(&self, query: RuntimeViewQuery) -> crate::runtime::Result<RuntimeViewFrame> {
        let snapshot = self.capture_state().await?;

        self.view_frame(&query, &snapshot).await
    }

    pub(crate) async fn view_frame(
        &self, query: &RuntimeViewQuery, snapshot: &RuntimeStateSnapshot,
    ) -> crate::runtime::Result<RuntimeViewFrame> {
        let scope = query.scope.as_ref();
        let window = &query.window;
        let resolution = &query.resolution;
        let session = self.storage().clone();
        let payload = if let Some(mut payload) = state_payload(query, snapshot)? {
            if let RuntimeViewPayload::Overview { frame } = &mut payload {
                frame.artifacts = self.execution_storage.artifact_count().await?;
                if matches!(resolution, RuntimeResolution::Raw) {
                    frame.shell = Some(self.runtime_overview().await?);
                }
                frame.installed_plugins = self.state_read().installed_plugins.len();
                frame.telemetry_samples = self.telemetry_sample_count().await;
            }
            payload
        } else {
            match &query.topic {
                RuntimeViewTopic::ExecutionFiles => {
                    let frame = match scope.and_then(|scope| scope.test_execution_id.as_ref()) {
                        Some(id) => {
                            let files = self.execution_files(id).await?;
                            crate::queries::RuntimeExecutionFileRead::from_frame(crate::queries::artifacts::execution_file_frame(
                                id, &files, window, resolution,
                            ))
                        }
                        None => crate::queries::RuntimeExecutionFileRead::unselected(),
                    };
                    RuntimeViewPayload::ExecutionFiles { frame }
                }
                RuntimeViewTopic::Records => {
                    let captured = crate::queries::records::execution_capture(
                        snapshot.state().executions.values(),
                        scope.and_then(|scope| scope.test_execution_id.as_ref()),
                    );
                    let filter = crate::queries::records::view_filter(snapshot, scope, captured);
                    let records = self.record_source.saved_filtered_records(&self.state_manager, snapshot.revision(), filter).await?;
                    let records = records.iter().flat_map(|page| page.iter());
                    let frame = crate::queries::records::record_frame(records, window, resolution, |record| {
                        captured.and_then(|captures| captures.captured_scope(record))
                    });
                    RuntimeViewPayload::Records { frame: Box::new(frame) }
                }
                RuntimeViewTopic::Telemetry { stream } => {
                    // Live views use the bounded Harmos window. Historical execution
                    // inspection reads captured evidence through the native lake.
                    // Display reduction runs outside Harmos's writer task.
                    let samples = self
                        .collect_telemetry(&crate::evidence_runtime::TelemetrySelection {
                            streams: stream.clone().into_iter().map(Into::into).collect(),
                            ..crate::evidence_runtime::TelemetrySelection::default()
                        })
                        .await?;
                    let scope = query.scope.clone();
                    let window = window.clone();
                    let resolution = resolution.clone();
                    let request_cursor = query.cursor;

                    return tokio::task::spawn_blocking(move || {
                        crate::queries::telemetry::telemetry_view_frame(samples, None, scope.as_ref(), &window, &resolution, request_cursor)
                    })
                    .await
                    .map_err(|error| crate::runtime::Error::Operation(format!("telemetry frame task failed: {error}")))?;
                }
                RuntimeViewTopic::Document { document_id } => {
                    // Documents embed telemetry, which lives on async durable streams —
                    // resolve it here so the live Document view stays byte-identical with
                    // the on-demand `resolve_cycle_document` route (same data seam).
                    let session = self.storage().clone();
                    let at = self.storage().cursor_saved();
                    let telemetry = self
                        .collect_telemetry_at_in(
                            &session,
                            &crate::evidence_runtime::TelemetrySelection {
                                scope: crate::document::DocumentSelection { scope: query.scope.clone().unwrap_or_default() }
                                    .evidence_scope(),
                                ..crate::evidence_runtime::TelemetrySelection::default()
                            },
                            None,
                            None,
                            &at,
                        )
                        .await?;
                    let records = self.record_pages_at_in(&session, snapshot, &at).await?;
                    let artifacts = self.artifacts_at_in(&session, &at).await?;
                    let state = snapshot.state();
                    let data = crate::documents::RuntimeDocumentData { records, telemetry, artifacts };
                    let selection = crate::document::DocumentSelection { scope: query.scope.clone().unwrap_or_default() };

                    RuntimeViewPayload::Document {
                        frame: crate::queries::documents::resolved_document_frame(
                            state,
                            document_id,
                            selection,
                            data,
                            crate::document::bounded::DocumentLimits::default(),
                        )?,
                    }
                }
                RuntimeViewTopic::Artifacts => {
                    let artifacts = self.artifacts().await?.to_vec();

                    RuntimeViewPayload::Artifacts {
                        frame: Box::new(crate::queries::artifacts::artifact_frame(artifacts.iter(), scope, window, resolution)),
                    }
                }
                RuntimeViewTopic::Traceability => {
                    let at = self.storage().cursor_saved();
                    let snapshots = self.retained_traceability_at_in(&session, snapshot.state(), scope, &at, true).await?;
                    RuntimeViewPayload::Traceability {
                        frame: crate::queries::projections::traceability_frame(snapshots, scope, window, resolution)?,
                    }
                }
                _ => unreachable!("state-backed topics were projected above"),
            }
        };

        Ok(RuntimeViewFrame { cursor: RuntimeCursor(snapshot.revision()), window: query.window.clone(), payload })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn state_payload(query: &RuntimeViewQuery, snapshot: &RuntimeStateSnapshot) -> crate::runtime::Result<Option<RuntimeViewPayload>> {
    let scope = query.scope.as_ref();
    let window = &query.window;
    let resolution = &query.resolution;
    Ok(Some(match &query.topic {
        RuntimeViewTopic::Overview => RuntimeViewPayload::Overview {
            frame: Box::new(crate::runtime_state::overview_entity_counts(
                snapshot.state(),
                0,
                crate::traceability::count(snapshot.state())?,
            )),
        },
        RuntimeViewTopic::InstalledPlugins => RuntimeViewPayload::InstalledPlugins {
            frame: Box::new(crate::queries::installed_plugins::installed_plugin_frame(snapshot.state())),
        },
        RuntimeViewTopic::Suites => RuntimeViewPayload::Suites {
            frame: list_frame(snapshot.state().suites.suites().map(|snapshot| snapshot.suite.metadata.identifier.to_string())),
        },
        RuntimeViewTopic::Cycle => {
            RuntimeViewPayload::Cycles { frame: list_frame(snapshot.state().cycles.keys().map(|cycle_id| cycle_id.to_string())) }
        }
        RuntimeViewTopic::Execution => RuntimeViewPayload::Execution {
            frame: list_frame(
                snapshot
                    .state()
                    .executions
                    .values()
                    .filter(|execution| scope.is_none_or(|scope| execution.scope.is_visible_in(scope)))
                    .map(|execution| execution.test_execution_id.to_string()),
            ),
        },
        RuntimeViewTopic::RecordDefinitions => RuntimeViewPayload::RecordDefinitions {
            frame: list_frame(snapshot.state().record_definitions.values().map(|definition| definition.record_type.to_string())),
        },
        RuntimeViewTopic::TelemetryDefinitions { stream } => RuntimeViewPayload::TelemetryDefinitions {
            frame: list_frame(
                snapshot
                    .state()
                    .telemetry_definitions
                    .values()
                    .filter(|definition| stream.as_ref().is_none_or(|stream| &definition.stream == stream))
                    .map(|definition| definition.stream.to_string()),
            ),
        },
        RuntimeViewTopic::Documents => {
            let cycles = snapshot.state().cycle_views().map(crate::definition::TestCycleInstance::new).collect::<Vec<_>>();

            RuntimeViewPayload::Documents { frame: Box::new(crate::queries::documents::document_frame(cycles.iter(), scope)) }
        }
        RuntimeViewTopic::Resources => RuntimeViewPayload::Resources {
            frame: Box::new(crate::queries::projections::resource_frame(
                snapshot.state().resource_records().iter(),
                scope,
                window,
                resolution,
            )),
        },
        RuntimeViewTopic::Prompts => RuntimeViewPayload::Prompts {
            frame: Box::new(crate::queries::projections::prompt_frame(
                snapshot.state().pending_prompt_projection().iter(),
                scope,
                window,
                resolution,
            )),
        },
        RuntimeViewTopic::Health => RuntimeViewPayload::Health {
            // Health items carry no sequence, so `Since` never filters them —
            // the cursor is pure change detection, stamped from the journal
            // like every other topic.
            frame: Box::new(crate::queries::projections::health_frame(snapshot.state().health_items().iter(), scope, window, resolution)),
        },
        RuntimeViewTopic::Notes => RuntimeViewPayload::Notes {
            frame: Box::new(crate::queries::projections::note_frame(snapshot.state().notes().iter(), scope, window, resolution)),
        },
        RuntimeViewTopic::ExecutionFiles
        | RuntimeViewTopic::Records
        | RuntimeViewTopic::Telemetry { .. }
        | RuntimeViewTopic::Document { .. }
        | RuntimeViewTopic::Artifacts
        | RuntimeViewTopic::Traceability => return Ok(None),
    }))
}

fn list_frame(items: impl Iterator<Item = String>) -> RuntimeListFrame {
    let items = items.collect::<Vec<_>>();

    RuntimeListFrame { count: items.len(), items }
}

/* === Tests === */
