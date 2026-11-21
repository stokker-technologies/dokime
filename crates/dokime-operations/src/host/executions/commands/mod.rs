/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Execution Commands

/* === Modules === */

pub mod create;
pub mod get;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use arrow_array::{Array, BinaryArray};
use harmos_operations::storage::{
    ColumnKind, ColumnPredicate, ColumnSelection, ColumnSource, GetObject, GetObjectRequest, ListColumnRows, ListColumnRowsRequest,
    MutationId, SchemaInput, SetObject, SetObjectRequest, set_object_response::Outcome,
};
use harmos_protocol::model::PageRequest;
use harmos_protocol::model::{Position, ReadCut, Receipt};
use harmos_query::arrow::ArrowBatch;
use harmos_runtime::storage::{ObjectDefinition, ReadLimits};
use prost::Message;
use prost_types::Timestamp;
use uuid::Uuid;

// Local Imports
use crate::host::{Context, files::transfer};
use crate::{Error, Result};
use dokime_protocol::host::executions::{ExecutionCommandPhase, ExecutionCommandReceipt};
use dokime_storage::persisted::{self, Administration, Command, Scope};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Snapshot {
    value: Option<Command>,
    revision: Vec<u8>,
    position: Position,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn read(
    context: &Context, execution: &str, command: &str, at: Option<Position>,
) -> Result<(Option<ExecutionCommandReceipt>, Position)> {
    let at = at.map(|position| ReadCut {
        positions: vec![position],
        revisions: Vec::new(),
    });
    let snapshot = snapshot(context, execution, command, at).await?;
    let command = snapshot.value.map(|saved| decode(&saved.current)).transpose()?;
    Ok((command, snapshot.position))
}

pub async fn save(context: &Context, command: &ExecutionCommandReceipt) -> Result<Receipt> {
    let key = key(&command.execution_id, &command.command_id);
    let snapshot = snapshot(context, &command.execution_id, &command.command_id, None).await?;
    let request_id = Uuid::now_v7().to_string();
    let mutation = transfer::mutation(context, &key, &request_id).await?;
    let payload = command.encode_to_vec();
    let saved = match snapshot.value {
        None => Command {
            key,
            scope: Scope::Execution(&command.execution_id).identifier()?,
            current: payload.clone(),
            original: payload,
            window: mutation.window.clone(),
            mutation: mutation.key.clone(),
        },
        Some(mut saved) => {
            saved.current = payload;
            saved
        }
    };
    write(context, &saved, snapshot.revision, mutation).await
}

pub async fn receipt(context: &Context, execution: &str, command: &str) -> Result<Receipt> {
    let snapshot = snapshot(context, execution, command, None).await?;
    let mut saved = snapshot.value.ok_or_else(|| Error::NotFound("Read Execution Command Receipt: Select an Accepted Command".into()))?;
    let original = decode(&saved.original)?;
    saved.current = saved.original.clone();
    let mutation = MutationId {
        window: saved.window.clone(),
        key: saved.mutation.clone(),
    };
    let mut receipt = write(context, &saved, Vec::new(), mutation).await?;
    receipt.request_id = original.request_id;
    Ok(receipt)
}

pub async fn recover(context: &Context, at: Timestamp) -> Result<()> {
    // Refuse Superseded Runtime State Without Reading or Rewriting Its Payload.
    let schema = ObjectDefinition::of::<Administration>()
        .schema()
        .map_err(|source| Error::Invalid(format!("Recover Execution Commands: Restore Their Schema: {source}")))?;
    let mut cut = None;
    let mut token = String::new();
    loop {
        let response = context
            .storage::<ListColumnRows>(ListColumnRowsRequest {
                selection: Some(ColumnSelection {
                    source: Some(ColumnSource {
                        kind: ColumnKind::Objects,
                        schema: schema.clone(),
                    }),
                    columns: vec!["payload.key".into()],
                    predicate: Some(ColumnPredicate {
                        column: "_storage.item".into(),
                        lower: None,
                        upper: None,
                        prefix: Some("execution-command/".into()),
                    }),
                    time: None,
                    start: 0,
                    work: Some(limits(context)?),
                }),
                at: cut,
                page: Some(PageRequest {
                    page_size: 1,
                    page_token: token,
                    ..Default::default()
                }),
            })
            .await?;
        cut = Some(response.resolved_at.ok_or_else(|| Error::Precondition("Recover Execution Commands: Return Their Issued Cut".into()))?);
        if let Some(page) = response.page {
            let batch = ArrowBatch::from_ipc(page.batch.data)
                .map_err(|source| Error::Invalid(format!("Recover Execution Commands: Read Their Durable Keys: {source}")))?;
            if batch.as_record_batch().num_rows() != 0 {
                return Err(Error::Precondition("Open Command History: Select a Fresh Storage Root; Preserve the Existing Root".into()));
            }
        }
        token = response.next_page_token;
        if token.is_empty() {
            break;
        }
    }

    // Recover One Bounded Page at a Fixed Image Before Reading the Next Page.
    let schema = ObjectDefinition::of::<Command>()
        .schema()
        .map_err(|source| Error::Invalid(format!("Recover Execution Commands: Restore Their Schema: {source}")))?;
    let mut cut = None;
    let mut token = String::new();
    loop {
        let response = context
            .storage::<ListColumnRows>(ListColumnRowsRequest {
                selection: Some(ColumnSelection {
                    source: Some(ColumnSource {
                        kind: ColumnKind::Objects,
                        schema: schema.clone(),
                    }),
                    columns: vec!["payload.current".into()],
                    predicate: None,
                    time: None,
                    start: 0,
                    work: Some(limits(context)?),
                }),
                at: cut,
                page: Some(PageRequest {
                    page_size: context.settings.page_items as u32,
                    page_token: token,
                    ..Default::default()
                }),
            })
            .await?;
        cut = Some(response.resolved_at.ok_or_else(|| Error::Precondition("Recover Execution Commands: Return Their Issued Cut".into()))?);
        if let Some(page) = response.page {
            let batch = ArrowBatch::from_ipc(page.batch.data)
                .map_err(|source| Error::Invalid(format!("Recover Execution Commands: Read Their Durable State: {source}")))?;
            let batch = batch.as_record_batch();
            let commands = batch
                .column_by_name("payload.current")
                .and_then(|column| column.as_any().downcast_ref::<BinaryArray>())
                .ok_or_else(|| Error::Precondition("Recover Execution Commands: Restore Their Current Payload Column".into()))?;
            for index in 0..commands.len() {
                if commands.is_null(index) {
                    return Err(Error::Precondition("Recover Execution Commands: Restore Every Command Payload".into()));
                }
                let mut command = decode(commands.value(index))?;
                if !matches!(
                    ExecutionCommandPhase::try_from(command.phase),
                    Ok(ExecutionCommandPhase::Accepted | ExecutionCommandPhase::Dispatching | ExecutionCommandPhase::Running)
                ) {
                    continue;
                }
                command.phase = ExecutionCommandPhase::Interrupted as i32;
                command.updated_at = Some(at);
                command.detail = Some("Host Restarted; Inspect the Sealed Execution Before Starting a New Run".into());
                // Resolve the Opaque Revision at the Opening Image, Preserving Its Continuations.
                let snapshot = snapshot(context, &command.execution_id, &command.command_id, cut.clone()).await?;
                let mut saved =
                    snapshot.value.ok_or_else(|| Error::Precondition("Recover Execution Command: Restore Its Durable Value".into()))?;
                saved.current = command.encode_to_vec();
                let request_id = Uuid::now_v7().to_string();
                let mutation = transfer::mutation(context, &saved.key, &request_id).await?;
                write(context, &saved, snapshot.revision, mutation).await?;
            }
        }
        token = response.next_page_token;
        if token.is_empty() {
            return Ok(());
        }
    }
}

/* === Private Functions === */

fn decode(payload: &[u8]) -> Result<ExecutionCommandReceipt> {
    if payload.is_empty() {
        return Err(Error::Precondition("Open Command History: Select a Fresh Storage Root; Preserve the Existing Root".into()));
    }
    ExecutionCommandReceipt::decode(payload).map_err(|source| {
        Error::Precondition(format!("Open Command History: Select a Fresh Storage Root; Preserve the Existing Root: {source}"))
    })
}

fn limits(context: &Context) -> Result<ReadLimits> {
    Ok(ReadLimits {
        rows: NonZeroUsize::new(context.settings.page_items)
            .ok_or_else(|| Error::Bound("Access Execution Commands: Configure a Positive Page Bound".into()))?,
        bytes: NonZeroUsize::new(context.settings.message_bytes)
            .ok_or_else(|| Error::Bound("Access Execution Commands: Configure a Positive Message Bound".into()))?,
    })
}

async fn snapshot(context: &Context, execution: &str, command: &str, at: Option<ReadCut>) -> Result<Snapshot> {
    let schema = ObjectDefinition::of::<Command>()
        .schema()
        .map_err(|source| Error::Invalid(format!("Read Execution Command: Restore Its Schema: {source}")))?;
    let response = context
        .storage::<GetObject>(GetObjectRequest {
            schema: Some(schema),
            item: key(execution, command),
            at,
        })
        .await?;
    let value = response
        .value
        .map(|value| {
            persisted::catalog()
                .decode::<Command>(&value, limits(context)?.bytes)
                .map_err(|source| Error::Invalid(format!("Read Execution Command: Restore Its Frozen Payload: {source}")))
        })
        .transpose()?;
    let incomplete = value.as_ref().is_some_and(|saved| saved.current.is_empty() || saved.original.is_empty());
    if incomplete {
        return Err(Error::Precondition("Open Command History: Select a Fresh Storage Root; Preserve the Existing Root".into()));
    }
    let position = response
        .resolved_at
        .and_then(|cut| cut.positions.into_iter().next())
        .ok_or_else(|| Error::Precondition("Read Execution Command: Return Its Durable Read Position".into()))?;
    Ok(Snapshot {
        value,
        revision: response.revision,
        position,
    })
}

async fn write(context: &Context, saved: &Command, expected: Vec<u8>, mutation: MutationId) -> Result<Receipt> {
    let value = persisted::catalog()
        .encode(saved, limits(context)?.bytes)
        .map_err(|source| Error::Invalid(format!("Save Execution Command: {source}")))?;
    let graph = ObjectDefinition::of::<Command>()
        .graph(&Default::default())
        .map_err(|source| Error::Invalid(format!("Save Execution Command: {source}")))?;
    let response = context
        .storage::<SetObject>(SetObjectRequest {
            mutation: Some(mutation),
            create_if_absent: expected.is_empty(),
            expected_revision: expected,
            schema: Some(SchemaInput {
                key: Some(value.schema.clone()),
                graph: Some(graph),
            }),
            value: Some(value),
        })
        .await?;
    match response.outcome {
        Some(Outcome::Applied(applied)) => {
            applied.receipt.ok_or_else(|| Error::Precondition("Save Execution Command: Return Its Durable Receipt".into()))
        }
        Some(Outcome::Conflict(_)) => Err(Error::Conflict("Save Execution Command: Read Its Current Revision Before Updating It".into())),
        None => Err(Error::Precondition("Save Execution Command: Return Its Durable Mutation Outcome".into())),
    }
}

fn key(execution: &str, command: &str) -> String {
    format!("execution-command/{}/{execution}{command}", execution.len())
}

/* === Tests === */
