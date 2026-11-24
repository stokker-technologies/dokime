/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Samples List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_protocol::model::ReadCut;
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::evidence::SampleRow;
use dokime_protocol::host::evidence::SamplesListRequest;
use dokime_protocol::host::evidence::SamplesListResponse;
use dokime_protocol::host::evidence::sample_row;
use dokime_protocol::model::origin;
use dokime_storage::{ExecutionSnapshot, SampleWindow};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Samples/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: SamplesListRequest) -> Result<SamplesListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async { read_page(context, request, None) }).await
}

/// One Page of the Selection at Its Cut; After a Cut a Watch Completed, Only the Samples Beyond It.
pub(crate) fn read_page(context: &Context, request: SamplesListRequest, after: Option<&ReadCut>) -> Result<SamplesListResponse> {
    let identifier = crate::host::projection::execution_scope(request.scope.as_ref())?;
    let store = context.host.store();
    let cut = store.metadata(identifier)?;
    let page = request.page.clone().unwrap_or_default();
    let supplied = request
        .at
        .as_ref()
        .map(|cut| {
            if cut.positions.len() != 1 || !cut.revisions.is_empty() {
                return Err(Error::Invalid("List Samples: Supply Its Returned Execution Cut".into()));
            }
            Ok(&cut.positions[0])
        })
        .transpose()?;
    let position = store.resource_value(identifier, "progress", supplied)?.resolved_at;
    let floor = match after.map(|after| after.positions.as_slice()) {
        None => 0,
        Some([completed]) if completed.history == position.history && completed.sequence <= position.sequence => completed.sequence,
        Some(_) => return Err(Error::Precondition("Watch Samples: A Retained History Was Removed; Resynchronize".into())),
    };
    let mut cut = cut;
    cut.frontier = position.sequence;
    let resolved_at = ReadCut {
        positions: vec![position],
        revisions: Vec::new(),
    };
    let mut selected = request.clone();
    selected.page = None;
    selected.at = None;
    let binding = format!(
        "{}{}",
        crate::host::views::hex(&crate::host::paging::bytes(&selected)),
        crate::host::views::hex(&Sha256::digest(resolved_at.encode_to_vec()))
    );
    let mut cursor = BTreeMap::<String, u64>::new();
    if !page.page_token.is_empty() {
        let (key, token) =
            page.page_token.split_once('/').ok_or_else(|| Error::Invalid("List Samples: Supply the Returned Page Token".into()))?;
        if key != binding {
            return Err(Error::Invalid("List Samples: Keep the Same Cut and Selection".into()));
        }
        cursor = serde_json::from_str(token).map_err(|_| Error::Invalid("List Samples: Supply the Returned Stream Positions".into()))?;
    }
    let start_ns = request
        .between
        .as_ref()
        .and_then(|window| window.start.as_ref())
        .map(crate::resources::timestamp_ns)
        .transpose()?
        .unwrap_or(i64::MIN);
    let end_ns = request
        .between
        .as_ref()
        .and_then(|window| window.end.as_ref())
        .map(crate::resources::timestamp_ns)
        .transpose()?
        .unwrap_or(i64::MAX);
    if start_ns > end_ns {
        return Err(Error::Invalid("List Samples: Choose Increasing Source-Time Bounds".into()));
    }
    let names = store.stream_names(identifier)?;
    let streams = names
        .iter()
        .filter(|stream| {
            request.streams.is_empty() || request.streams.iter().any(|pattern| crate::host::evidence::selection::matches(pattern, stream))
        })
        .cloned()
        .collect::<Vec<_>>();
    if cursor.iter().any(|(key, value)| !names.contains(key) || *value > cut.frontier) {
        return Err(Error::Invalid("List Samples: Continue Within the Selected Stream Histories".into()));
    }
    let limit = crate::host::projection::limit(page.page_size, context.settings.page_items);
    let (step_after, through) = store.step_range(
        identifier,
        request.scope.as_ref().and_then(|scope| scope.step_id.as_deref()).unwrap_or_default(),
        cut.frontier,
        NonZeroUsize::new(context.settings.page_items).unwrap(),
    )?;
    let step_after = step_after.max(floor);
    for stream in &streams {
        cursor.entry(stream.clone()).and_modify(|value| *value = (*value).max(step_after)).or_insert(step_after);
    }
    let source = match cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
        Some(origin::Kind::PluginRun(run)) => run.plugin.clone(),
        _ => "import".into(),
    };
    let source_matches =
        request.sources.is_empty() || request.sources.iter().any(|pattern| crate::host::evidence::selection::matches(pattern, &source));
    let version_matches = request.versions.is_empty() || request.versions.contains(&1);
    let scope_matches =
        request.scope.as_ref().is_none_or(|scope| scope.cycle_id.as_ref().is_none_or(|cycle| cycle == &cut.execution.cycle));
    let entries = cursor.iter().map(|(key, value)| (key.clone(), *value)).collect::<Vec<_>>();
    let mut samples = if streams.is_empty() || !source_matches || !version_matches || !scope_matches {
        Vec::new()
    } else {
        store.sample_window(SampleWindow {
            execution: identifier,
            streams: &streams,
            fields: &request.fields,
            after: &entries,
            through,
            start_ns,
            end_ns,
            latest: request.latest,
            limit: NonZeroUsize::new(limit + 1).unwrap(),
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
        cursor.insert(sample.stream.clone(), sample.position);
        selected.push(sample);
    }
    if bounded {
        for stream in &streams {
            cursor.insert(stream.clone(), through);
        }
    }
    let catalog = store.frozen_catalog(&cut.execution.catalog)?;
    let snapshot = ExecutionSnapshot {
        execution: cut.execution,
        state: cut.state,
        seal: cut.seal,
        samples: selected,
        evidence: Vec::new(),
    };
    let mut samples = crate::host::projection::samples(&snapshot, &catalog, &[], None)?;
    for sample in &mut samples {
        sample.source = source.clone();
        if let Some(scope) = &mut sample.scope {
            scope.step_id = request.scope.as_ref().and_then(|selected| selected.step_id.clone());
        }
        // Rows Read Only the Selected Fields, So Values Drop the Others' Defaults.
        if let Some(values) = &mut sample.values {
            values.values.retain(|key, _| {
                let field = request.fields.is_empty() || request.fields.iter().any(|field| field.split('.').next() == Some(key.as_str()));
                let signal = request.signals.is_empty()
                    || request.signals.iter().any(|pattern| crate::host::evidence::selection::matches(pattern, key));
                field && signal
            });
        }
    }
    let next_page_token = if bounded {
        String::new()
    } else {
        format!(
            "{binding}/{}",
            serde_json::to_string(&cursor).map_err(|error| Error::Invalid(format!("List Samples: Encode Continuation: {error}")))?
        )
    };
    Ok(SamplesListResponse {
        samples: samples
            .into_iter()
            .map(|sample| SampleRow {
                kind: Some(sample_row::Kind::Raw(sample)),
            })
            .collect(),
        schema: None,
        resolved_at: Some(resolved_at),
        next_page_token,
    })
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SamplesListRequest, SamplesListResponse, context, "dokime.host.v1.evidence", "Samples", "List");
