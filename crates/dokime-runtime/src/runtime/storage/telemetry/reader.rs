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
use harmos::{
    Identifier,
    storage::{ReadBatch, query::StreamSelection},
};

// Local Imports
use super::{PositionedTelemetry, invalid};
use crate::runtime::{Error, Result, storage::execution::ExecutionView};
use crate::storage::telemetry::{TelemetryEntry, TypedCodec};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct TelemetryReader {
    view: ExecutionView,
    local: crate::storage::sqlite::State,
    codecs: ReadBatch<Vec<TypedCodec>>,
    readers: Vec<TypedRead>,
    page_rows: usize,
}

#[derive(Clone)]
struct TypedRead {
    codec: usize,
    head: Option<ReadBatch<Vec<PositionedTelemetry>>>,
    offset: usize,
    after: u64,
    finished: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl TelemetryReader {
    pub(in crate::runtime::storage) async fn open_page(
        view: &ExecutionView, local: &crate::storage::sqlite::State, stream: &Identifier, after: u64,
        settings: &harmos::config::SchemaSettings,
    ) -> Result<Self> {
        Self::selected(view, local, Some(stream), None, after, settings).await
    }

    pub(in crate::runtime::storage) async fn selected(
        view: &ExecutionView, local: &crate::storage::sqlite::State, stream: Option<&Identifier>, version: Option<u32>, after: u64,
        settings: &harmos::config::SchemaSettings,
    ) -> Result<Self> {
        let session = view.session().clone();
        let captured = view.captured_source().ok_or_else(|| invalid("typed telemetry read requires captured producer schemas"))?;
        let selected = |schema: &str, candidate: u32| {
            let Some(schema) = schema.strip_prefix("dokime.telemetry.") else {
                return false;
            };
            captured.telemetry_contracts.iter().any(|(id, versions)| {
                id.as_str() == schema
                    && stream.is_none_or(|stream| stream == id)
                    && version.is_none_or(|version| candidate == version)
                    && versions.contains_key(&candidate)
            })
        };
        let at = view.evidence_through()?;
        let codecs = super::saved_codecs(&session, at, None, settings, selected).await?;
        for (id, versions) in &captured.telemetry_contracts {
            if stream.is_some_and(|stream| stream != id) {
                continue;
            }
            for contract in versions.values() {
                if version.is_some_and(|version| contract.schema_version != version) {
                    continue;
                }
                if !codecs.iter().any(|codec| {
                    codec.schema().id.as_str().strip_prefix("dokime.telemetry.") == Some(id.as_str())
                        && codec.schema().version.get() == contract.schema_version
                }) {
                    return Err(invalid("captured telemetry schema is absent at its saved boundary"));
                }
            }
        }
        let mut readers = Vec::new();
        readers.try_reserve_exact(codecs.len()).map_err(|_| invalid("telemetry reader allocation failed"))?;
        readers.extend((0..codecs.len()).map(|codec| TypedRead {
            codec,
            head: None,
            offset: 0,
            after,
            finished: false,
        }));
        Ok(Self {
            view: view.clone(),
            local: local.clone(),
            codecs,
            readers,
            page_rows: view.storage().settings().response.batch_rows.get(),
        })
    }

    pub async fn next(&mut self) -> Result<Option<ReadBatch<Vec<TelemetryEntry>>>> {
        let Some(rows) = self.next_positioned().await? else {
            return Ok(None);
        };
        Ok(Some(crate::storage::projection::page(rows, |row| Ok(row.captured))?))
    }

    pub(in crate::runtime::storage) async fn next_positioned(&mut self) -> Result<Option<ReadBatch<Vec<PositionedTelemetry>>>> {
        if self.readers.iter().all(|reader| reader.finished && reader.head.is_none()) {
            return Ok(None);
        }
        let mut readers = Vec::new();
        readers.try_reserve_exact(self.readers.len()).map_err(|_| invalid("telemetry reader allocation failed"))?;
        readers.extend(self.readers.iter().cloned());
        let mut selected = harmos::storage::ReadBatch::new(()).try_map(|_| {
            let mut rows = Vec::<PositionedTelemetry>::new();
            rows.try_reserve_exact(self.page_rows).map_err(|_| invalid("telemetry page allocation failed"))?;
            Ok::<_, Error>(rows)
        })?;

        while selected.len() < self.page_rows {
            for source in &mut readers {
                while !source.finished && source.head.is_none() {
                    let rows = read_head(&self.local, &self.view, &self.codecs[source.codec], source.after, self.page_rows).await?;
                    source.finished = rows.len() < self.page_rows;
                    if let Some(last) = rows.last() {
                        source.after = last.position;
                    }
                    source.offset = 0;
                    source.head = (!rows.is_empty()).then_some(rows);
                }
            }
            let next = readers
                .iter()
                .enumerate()
                .filter_map(|(index, source)| source.head.as_ref().map(|rows| (index, rows[source.offset].position)))
                .min_by_key(|(_, position)| *position);
            let Some((index, _)) = next else {
                break;
            };
            let source = &mut readers[index];
            let rows = source.head.as_ref().expect("selected telemetry head exists");
            let row = &rows[source.offset];
            selected = selected.try_map(|mut selected| {
                selected.push(PositionedTelemetry {
                    position: row.position,
                    captured: row.captured.clone(),
                });
                Ok::<_, Error>(selected)
            })?;
            source.offset += 1;
            if source.offset == rows.len() {
                source.head = None;
            }
        }
        // Commit Reader Progress Only After Every Projection Succeeds.
        self.readers = readers;

        Ok((!selected.is_empty()).then_some(selected))
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn read_head(
    local: &crate::storage::sqlite::State, view: &ExecutionView, codec: &TypedCodec, after: u64, page_rows: usize,
) -> Result<ReadBatch<Vec<PositionedTelemetry>>> {
    let source = StreamSelection {
        schema: codec.schema().clone(),
        scope: Some(Identifier::new(view.execution().test_execution_id.as_str()).map_err(crate::storage::Error::from)?),
        item: None,
        stream: None,
        source_start: None,
        source_end: None,
        after,
    };
    let page = harmos::storage::ScanPage {
        after,
        rows: std::num::NonZeroUsize::new(page_rows).ok_or_else(|| invalid("telemetry page must be nonempty"))?,
        bytes: view.storage().settings().response.batch_bytes,
    };
    let rows = local.telemetry_read(view.evidence_through()?.clone(), source, page).await?;
    let rows = rows.into_iter().map(|(position, captured)| PositionedTelemetry { position, captured }).collect();
    Ok(ReadBatch::new(rows))
}

/* === Tests === */
