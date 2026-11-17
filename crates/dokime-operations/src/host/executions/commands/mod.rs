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
use harmos_protocol::model::{Position, ReadCut};
use harmos_runtime::Identifier;
use harmos_runtime::builtins::storage::{SchemaInput, SetObject, SetObjectRequest, set_object_response::Outcome};
use harmos_runtime::storage::ObjectDefinition;
use prost::Message;
use prost_types::Timestamp;

// Local Imports
use crate::host::{Context, files::transfer, storage::administration};
use crate::{Error, Result};
use dokime_protocol::host::executions::{ExecutionCommandPhase, ExecutionCommandReceipt};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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
    let command = if snapshot.revision.is_empty() { None } else { Some(ExecutionCommandReceipt::decode(snapshot.payload.as_slice())?) };
    Ok((command, position))
}

pub async fn save(context: &Context, command: &ExecutionCommandReceipt) -> Result<()> {
    let key = key(&command.execution_id, &command.command_id);
    let snapshot = administration::read(context, &key, &[], None).await?;
    let bound = NonZeroUsize::new(context.settings.message_bytes)
        .ok_or_else(|| Error::Bound("Save Execution Command: Configure a Positive Message Bound".into()))?;
    let value = administration::Administration {
        key: key.clone(),
        payload: command.encode_to_vec(),
    };
    let mut value =
        administration::catalog().encode(&value, bound).map_err(|source| Error::Invalid(format!("Save Execution Command: {source}")))?;
    value.scope =
        Some(Identifier::new(command.execution_id.clone()).map_err(|source| Error::Invalid(format!("Save Execution Command: {source}")))?);
    let graph = ObjectDefinition::of::<administration::Administration>()
        .graph(&Default::default())
        .map_err(|source| Error::Invalid(format!("Save Execution Command: {source}")))?;
    let request_id = uuid::Uuid::now_v7().to_string();
    let mutation = transfer::mutation(context, &key, &request_id).await?;
    let response = context
        .storage::<SetObject>(SetObjectRequest {
            mutation: Some(mutation),
            create_if_absent: snapshot.revision.is_empty(),
            expected_revision: snapshot.revision,
            schema: Some(SchemaInput {
                key: Some(value.schema.clone()),
                graph: Some(graph),
            }),
            value: Some(value),
        })
        .await?;
    match response.outcome {
        Some(Outcome::Applied(_)) => Ok(()),
        Some(Outcome::Conflict(_)) => Err(Error::Conflict("Save Execution Command: Read Its Current Revision Before Updating It".into())),
        None => Err(Error::Precondition("Save Execution Command: Return Its Durable Mutation Outcome".into())),
    }
}

pub async fn recover(context: &Context, at: Timestamp) -> Result<()> {
    let (commands, _) = administration::scan(context, "execution-command/", None).await?;
    for command in commands {
        let mut command = ExecutionCommandReceipt::decode(command.payload.as_slice())?;
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

fn key(execution: &str, command: &str) -> String {
    format!("execution-command/{}/{execution}{command}", execution.len())
}

/* === Tests === */
