/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.telemetry.list_samples";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::TelemetryListSamplesRequest, v1::TelemetrySamplesReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::TelemetryListSamplesRequest) -> Result<v1::TelemetrySamplesReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let identifier = super::projection::execution_scope(request.scope.as_ref())?;
        let store = context.host.store();
        let cut = store.metadata(identifier)?;
        if request.latest && !request.after_stream_cursors.is_empty() {
            return Err(Error::Invalid("List Samples: Choose a Latest Tail or Forward Cursors".into()));
        }
        let start_ns = request.start.as_ref().map(crate::resources::timestamp_ns).transpose()?.unwrap_or(i64::MIN);
        let end_ns = request.end.as_ref().map(crate::resources::timestamp_ns).transpose()?.unwrap_or(i64::MAX);
        if start_ns > end_ns {
            return Err(Error::Invalid("List Samples: Choose Increasing Source-Time Bounds".into()));
        }
        let names = store.stream_names(identifier)?;
        let streams = names
            .iter()
            .filter(|stream| {
                request.streams.is_empty() || request.streams.iter().any(|pattern| super::evidence_selection::matches(pattern, stream))
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut after = std::collections::BTreeMap::new();
        for cursor in &request.after_stream_cursors {
            if !names.contains(&cursor.key) || cursor.value > cut.frontier || after.insert(cursor.key.clone(), cursor.value).is_some() {
                return Err(Error::Invalid("List Samples: Resynchronize with Unique Cursors from the Selected Execution".into()));
            }
        }
        let limit = super::projection::limit(request.limit, context.settings.page_items);
        let (step_after, through) = store.step_range(
            identifier,
            request.scope.as_ref().and_then(|scope| scope.step_id.as_deref()).unwrap_or_default(),
            cut.frontier,
            std::num::NonZeroUsize::new(context.settings.page_items).unwrap(),
        )?;
        for stream in &streams {
            after.entry(stream.clone()).and_modify(|value| *value = (*value).max(step_after)).or_insert(step_after);
        }
        let source = match cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
            Some(v1::origin::Kind::PluginRun(run)) => run.plugin.clone(),
            _ => "import".into(),
        };
        let source_matches =
            request.sources.is_empty() || request.sources.iter().any(|pattern| super::evidence_selection::matches(pattern, &source));
        let version_matches = request.schema_versions.is_empty() || request.schema_versions.contains(&1);
        let scope_matches =
            request.scope.as_ref().is_none_or(|scope| scope.cycle_id.as_ref().is_none_or(|cycle| cycle == &cut.execution.cycle));
        let entries = after.iter().map(|(key, value)| (key.clone(), *value)).collect::<Vec<_>>();
        let mut samples = if streams.is_empty() || !source_matches || !version_matches || !scope_matches {
            Vec::new()
        } else {
            store.sample_window(dokime_storage::SampleWindow {
                execution: identifier,
                streams: &streams,
                after: &entries,
                through,
                start_ns,
                end_ns,
                latest: request.latest,
                limit: std::num::NonZeroUsize::new(limit + 1).unwrap(),
            })?
        };
        let complete = request.latest || samples.len() <= limit;
        if samples.len() > limit {
            if request.latest {
                samples.remove(0);
            } else {
                samples.truncate(limit);
            }
        }
        let mut bytes = 128;
        let mut selected = Vec::new();
        let mut bounded = complete;
        for sample in samples {
            let length = sample.row.len() * 2 + 512;
            if bytes + length > context.settings.message_bytes {
                if selected.is_empty() {
                    return Err(Error::Invalid("List Samples: Increase the Message Budget for This Row".into()));
                }
                if request.latest {
                    return Err(Error::Invalid("List Samples: Lower the Latest Limit to Fit the Message Budget".into()));
                }
                bounded = false;
                break;
            }
            bytes += length;
            after.insert(sample.stream.clone(), sample.position);
            selected.push(sample);
        }
        if bounded {
            for stream in &streams {
                after.insert(stream.clone(), through);
            }
        }
        let snapshot = dokime_storage::ExecutionSnapshot {
            execution: cut.execution,
            state: cut.state,
            seal: cut.seal,
            samples: selected,
            evidence: Vec::new(),
        };
        let mut samples = super::projection::samples(&snapshot, &[], None)?;
        for sample in &mut samples {
            sample.source = source.clone();
            if let Some(scope) = &mut sample.scope {
                scope.step_id = request.scope.as_ref().and_then(|selected| selected.step_id.clone());
            }
            if !request.signals.is_empty()
                && let Some(values) = &mut sample.values
            {
                values.values.retain(|key, _| request.signals.iter().any(|pattern| super::evidence_selection::matches(pattern, key)));
            }
        }
        Ok(v1::TelemetrySamplesReply {
            samples,
            complete: bounded,
            through_stream_cursors: after
                .into_iter()
                .map(|(key, value)| v1::TelemetrySamplesReplyThroughStreamCursorsEntry { key, value })
                .collect(),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
