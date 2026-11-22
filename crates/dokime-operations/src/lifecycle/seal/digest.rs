/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Frozen Evidence Digest

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use arrow_array::{Array, StringArray, StructArray};
use harmos_operations::storage::{ListRows, ListRowsRequest, RowOutput};
use harmos_protocol::model::{PageRequest, ReadCut};
use harmos_query::{arrow::ArrowBatch, columnar};
use harmos_runtime::operations::OperationManager;
use harmos_runtime::storage::query::{FileSelection, ObjectSelection, ObjectVersions, RecordSelection, Source};
use harmos_runtime::{Identified, Identifier};
use sha2::{Digest, Sha256};

// Local Imports
use super::super::io;
use crate::{Error, Result};
use dokime_storage::persisted::{
    self, bindings, records,
    shapes::{PreparedReport, Preview, Report},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Sources Come From the Execution's Frozen Catalog and Native Object Inventory.
/// Only One Bounded Page and One Normalized Row Are Held While Hashing.
pub(super) async fn evidence(
    operations: &OperationManager, scope: &Identifier, at: &ReadCut, sources: &[Source], rows: NonZeroUsize, bytes: NonZeroUsize,
) -> Result<Vec<u8>> {
    let mut inventory = Vec::new();
    for definition in persisted::catalog().definitions() {
        let schema = definition.schema().map_err(|source| Error::Invalid(format!("Digest Evidence: {source}")))?;
        if [Preview::ID, PreparedReport::ID, Report::ID].contains(&schema.id.as_str()) {
            continue;
        }
        inventory.push(Source::Objects(ObjectSelection {
            schema: Some(schema),
            key: None,
            versions: ObjectVersions::Latest,
        }));
    }
    for (schema, _) in [
        bindings::schema::<records::ExecutionEvent>()?,
        bindings::schema::<records::EvidenceEvent>()?,
        bindings::schema::<records::HealthEvent>()?,
        bindings::schema::<records::PromptEvent>()?,
        bindings::schema::<records::AnswerEvent>()?,
        bindings::schema::<records::DocumentPart>()?,
    ] {
        inventory.push(Source::Records(RecordSelection {
            schema,
            scope: Some(scope.clone()),
            item: None,
            source_start: None,
            source_end: None,
        }));
    }
    inventory.push(Source::Files(FileSelection { scope: Some(scope.clone()), item: None }));
    // The Caller Supplies Only the Plugin Record/Sample Sources From the Frozen Catalog.
    for source in sources {
        if !matches!(source, Source::Records(_) | Source::Stream(_)) {
            return Err(Error::Invalid("Digest Evidence: Supply Only the Frozen Plugin Record and Sample Sources".into()));
        }
        inventory.push(source.clone());
    }
    let encoded = serde_json::to_vec(&inventory).map_err(|source| Error::Invalid(format!("Digest Evidence: {source}")))?;
    if encoded.len() > bytes.get() {
        return Err(Error::Bound("Digest Evidence: Keep the Frozen Source Inventory Within the Message Bound".into()));
    }
    let mut sources = inventory
        .iter()
        .map(|source| {
            validate(source, scope)?;
            let identity = serde_json::to_vec(source).map_err(|source| Error::Invalid(format!("Digest Evidence: {source}")))?;
            Ok((identity, source))
        })
        .collect::<Result<Vec<_>>>()?;
    sources.sort_by(|left, right| left.0.cmp(&right.0));
    if sources.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(Error::Invalid("Digest Evidence: Enumerate Each Frozen Source Once".into()));
    }
    let mut digest = Sha256::new();
    digest.update(b"dokime.evidence.v1\0");
    for (identity, source) in sources {
        digest.update((identity.len() as u64).to_le_bytes());
        digest.update(identity);
        let mut token = String::new();
        loop {
            let response = io::call::<ListRows>(
                operations,
                ListRowsRequest {
                    source: Some(source.clone()),
                    merge: None,
                    at: Some(at.clone()),
                    transforms: Vec::new(),
                    page: Some(PageRequest {
                        page_size: rows.get() as u32,
                        page_token: token,
                        ..Default::default()
                    }),
                },
            )
            .await?;
            for frame in response.rows {
                let RowOutput::Batch(data) = frame else { continue };
                let batch = ArrowBatch::from_ipc(data)
                    .map_err(|source| Error::Invalid(format!("Digest Evidence: Restore Its Native Rows: {source}")))?;
                let batch = batch.as_record_batch();
                let metadata = batch.column_by_name("_storage").and_then(|column| column.as_any().downcast_ref::<StructArray>());
                let scopes = metadata
                    .and_then(|metadata| metadata.column_by_name("scope"))
                    .or_else(|| batch.column_by_name("scope"))
                    .or_else(|| batch.column_by_name("_storage.scope"))
                    .and_then(|column| column.as_any().downcast_ref::<StringArray>())
                    .ok_or_else(|| Error::Precondition("Digest Evidence: Restore Every Row's Exact Scope".into()))?;
                for index in 0..batch.num_rows() {
                    if scopes.is_null(index) || scopes.value(index) != scope.as_str() {
                        continue;
                    }
                    let row =
                        columnar::take(batch, &[index as u64]).map_err(|source| Error::Invalid(format!("Digest Evidence: {source}")))?;
                    let row = ArrowBatch::try_new(row).map_err(|source| Error::Invalid(format!("Digest Evidence: {source}")))?;
                    digest.update((row.ipc_bytes().len() as u64).to_le_bytes());
                    digest.update(row.ipc_bytes());
                }
            }
            token = response.next_page_token;
            if token.is_empty() {
                break;
            }
        }
    }
    Ok(digest.finalize().to_vec())
}

/* === Private Functions === */

fn validate(source: &Source, scope: &Identifier) -> Result<()> {
    let valid = match source {
        Source::Objects(selection) => {
            selection.schema.as_ref().is_some_and(|schema| ![Preview::ID, PreparedReport::ID, Report::ID].contains(&schema.id.as_str()))
        }
        Source::Records(selection) => selection.scope.as_ref() == Some(scope),
        Source::Stream(selection) => selection.scope.as_ref() == Some(scope) && selection.after == 0,
        Source::Files(selection) => selection.scope.as_ref() == Some(scope),
        Source::Schemas(_) => false,
    };
    if !valid {
        return Err(Error::Invalid("Digest Evidence: Select Exact Execution Evidence and Exclude Previews and Reports".into()));
    }
    Ok(())
}

/* === Tests === */
