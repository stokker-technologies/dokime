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
use harmos_protocol::model::{Position, ReadCut, Receipt};
use harmos_runtime::Identifier;
use harmos_runtime::builtins::storage::{MutationId, SchemaInput, SetObject, SetObjectRequest, set_object_response::Outcome};
use harmos_runtime::storage::ObjectDefinition;
use prost::Message;
use prost_types::Timestamp;

// Local Imports
use crate::host::{Context, files::transfer, storage::administration};
use crate::{Error, Result};
use dokime_protocol::host::executions::{ExecutionCommandPhase, ExecutionCommandReceipt};
use dokime_storage::persisted::{self, Administration, Scope};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
struct Stored {
    #[prost(bytes = "vec", tag = "1")]
    current: Vec<u8>,

    #[prost(bytes = "vec", tag = "2")]
    original: Vec<u8>,

    #[prost(string, tag = "3")]
    window: String,

    #[prost(string, tag = "4")]
    mutation: String,

    #[prost(uint32, tag = "5")]
    format: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn read(
    context: &Context, execution: &str, command: &str, at: Option<Position>,
) -> Result<(Option<ExecutionCommandReceipt>, Position)> {
    let key = key(execution, command);
    let at = at.map(|position| ReadCut {
        positions: vec![position],
        revisions: Vec::new(),
    });
    let snapshot = administration::read(context, &key, &[], at).await?;
    let position = snapshot
        .resolved_at
        .and_then(|cut| cut.positions.into_iter().next())
        .ok_or_else(|| Error::Precondition("Read Execution Command: Return Its Durable Read Position".into()))?;
    let command = if snapshot.revision.is_empty() {
        None
    } else {
        Some(ExecutionCommandReceipt::decode(stored(&snapshot.payload)?.current.as_slice())?)
    };
    Ok((command, position))
}

pub async fn save(context: &Context, command: &ExecutionCommandReceipt) -> Result<Receipt> {
    let key = key(&command.execution_id, &command.command_id);
    let snapshot = administration::read(context, &key, &[], None).await?;
    let request_id = uuid::Uuid::now_v7().to_string();
    let mutation = transfer::mutation(context, &key, &request_id).await?;
    let payload = command.encode_to_vec();
    let saved = if snapshot.revision.is_empty() {
        Stored {
            current: payload.clone(),
            original: payload,
            window: mutation.window.to_string(),
            mutation: mutation.key.to_string(),
            format: 1,
        }
    } else {
        let mut saved = stored(&snapshot.payload)?;
        saved.current = payload;
        saved
    };
    write(context, &key, &command.execution_id, &saved, snapshot.revision, mutation).await
}

pub async fn receipt(context: &Context, execution: &str, command: &str) -> Result<Receipt> {
    let key = key(execution, command);
    let snapshot = administration::read(context, &key, &[], None).await?;
    if snapshot.revision.is_empty() {
        return Err(Error::NotFound("Read Execution Command Receipt: Select an Accepted Command".into()));
    }
    let mut saved = stored(&snapshot.payload)?;
    let original = ExecutionCommandReceipt::decode(saved.original.as_slice())?;
    saved.current = saved.original.clone();
    let mutation = MutationId {
        window: Identifier::new(saved.window.clone()).map_err(|source| Error::Invalid(format!("Replay Execution Command: {source}")))?,
        key: Identifier::new(saved.mutation.clone()).map_err(|source| Error::Invalid(format!("Replay Execution Command: {source}")))?,
    };
    let mut receipt = write(context, &key, execution, &saved, Vec::new(), mutation).await?;
    receipt.request_id = original.request_id;
    Ok(receipt)
}

pub async fn recover(context: &Context, at: Timestamp) -> Result<()> {
    let (commands, _) = administration::scan(context, "execution-command/", None).await?;
    for command in commands {
        let mut command = ExecutionCommandReceipt::decode(stored(&command.payload)?.current.as_slice())?;
        if !matches!(
            ExecutionCommandPhase::try_from(command.phase),
            Ok(ExecutionCommandPhase::Accepted | ExecutionCommandPhase::Dispatching | ExecutionCommandPhase::Running)
        ) {
            continue;
        }
        command.phase = ExecutionCommandPhase::Interrupted as i32;
        command.updated_at = Some(at);
        command.detail = Some("Host Restarted; Inspect the Sealed Execution Before Starting a New Run".into());
        save(context, &command).await?;
    }
    Ok(())
}

/* === Private Functions === */

fn stored(payload: &[u8]) -> Result<Stored> {
    let incompatible = || Error::Precondition("Open Command History: Select a Fresh Storage Root; Preserve the Existing Root".into());
    let saved = Stored::decode(payload).map_err(|_| incompatible())?;
    if saved.format != 1 || saved.current.is_empty() || saved.original.is_empty() || saved.window.is_empty() || saved.mutation.is_empty() {
        return Err(incompatible());
    }
    Ok(saved)
}

async fn write(context: &Context, key: &str, execution: &str, saved: &Stored, expected: Vec<u8>, mutation: MutationId) -> Result<Receipt> {
    let bound = NonZeroUsize::new(context.settings.message_bytes)
        .ok_or_else(|| Error::Bound("Save Execution Command: Configure a Positive Message Bound".into()))?;
    let value = Administration {
        key: key.into(),
        scope: Scope::Execution(execution).identifier()?,
        payload: saved.encode_to_vec(),
    };
    let value = persisted::catalog().encode(&value, bound).map_err(|source| Error::Invalid(format!("Save Execution Command: {source}")))?;
    let graph = ObjectDefinition::of::<Administration>()
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

#[cfg(test)]
mod tests {
    use super::stored;
    use dokime_protocol::host::executions::ExecutionCommandReceipt;
    use prost::Message;

    #[test]
    fn superseded_command_payloads_are_refused() {
        let command = ExecutionCommandReceipt {
            request_id: "request".into(),
            execution_id: "execution".into(),
            command_id: "command".into(),
            ..Default::default()
        };
        assert!(stored(&command.encode_to_vec()).is_err());
    }
}
