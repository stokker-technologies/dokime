/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompt Catalog

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::PromptInstance;
use dokime_protocol::host::PromptKind;
use dokime_protocol::host::PromptResponse;
use dokime_protocol::host::PromptView;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::model::MapValue;
use dokime_protocol::model::Prompt;
use dokime_protocol::model::PromptArchetype;
use dokime_protocol::model::PromptDefinitions;
use std::result::Result as StandardResult;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn definitions(context: &Context) -> Result<Vec<Prompt>> {
    let mut definitions = BTreeMap::new();
    for plugin in context.host.plugins().await? {
        let Some(catalog) = plugin.description.and_then(|description| description.declarations) else {
            continue;
        };
        for section in catalog.sections {
            if section.r#type != "dokime.model.v1.PromptDefinitions" {
                continue;
            }
            for definition in PromptDefinitions::decode(section.payload.as_slice())?.prompts {
                let id = definition
                    .metadata
                    .as_ref()
                    .ok_or_else(|| Error::Invalid("Reconnect a Plugin with Named Prompt Definitions".into()))?
                    .identifier
                    .clone();
                if let Some(existing) = definitions.insert(id.clone(), definition.clone())
                    && existing != definition
                {
                    return Err(Error::Precondition(format!("Read Prompt {id}: Remove the Ambiguous Plugin Definition")));
                }
            }
        }
    }
    Ok(definitions.into_values().collect())
}

pub(crate) fn view(context: &Context, identifier: &str) -> Result<PromptView> {
    let (execution, prompt) =
        identifier.split_once('/').ok_or_else(|| Error::Invalid("Read Prompt: Select an Identifier Returned by ListPending".into()))?;
    if !context.host.store().identifiers()?.iter().any(|id| id == execution) {
        return Err(Error::NotFound("Read Prompt: Select an Existing Execution".into()));
    }
    let (prompt, position) =
        prompt.rsplit_once('/').ok_or_else(|| Error::Invalid("Read Prompt: Use the Complete Pending Prompt Identifier".into()))?;
    let position = position.parse::<u64>().map_err(|_| Error::Invalid("Read Prompt: Use the Returned Question Position".into()))?;
    let snapshot = crate::host::execution_state::evidence(context, execution)?;
    let entry = snapshot
        .evidence
        .iter()
        .find(|entry| entry.position == position && entry.kind == "prompt")
        .ok_or_else(|| Error::NotFound("Read Prompt: Select a Question Asked by This Execution".into()))?;
    let question = PromptsCreateRequest::decode(entry.payload.as_slice())?;
    if question.prompt != prompt {
        return Err(Error::NotFound("Read Prompt: Use the Identifier Returned by ListPending".into()));
    }
    let next =
        snapshot.evidence.iter().find(|entry| entry.position > position && entry.kind == "prompt").map_or(u64::MAX, |entry| entry.position);
    let definition = crate::host::projection::prompt_definition(&snapshot, &question)?;
    let answer = snapshot
        .evidence
        .iter()
        .rev()
        .filter(|entry| entry.position > position && entry.position < next && (entry.kind == "client:answer" || entry.kind == "answer"))
        .map(|entry| PromptsCreateResponse::decode(entry.payload.as_slice()))
        .collect::<StandardResult<Vec<_>, _>>()?
        .into_iter()
        .find(|answer| answer.prompt == prompt);
    let response = answer.map(|answer| PromptResponse {
        prompt_id: identifier.into(),
        action: answer.action,
        accepted: answer.accepted,
        responder: answer.responder,
        submitted_at: answer.submitted_at,
        values: Some(MapValue { values: answer.values }),
    });
    let kind = match PromptArchetype::try_from(definition.archetype).unwrap_or_default() {
        PromptArchetype::Confirmation => PromptKind::Confirmation,
        PromptArchetype::Form => PromptKind::Form,
        PromptArchetype::Choice => PromptKind::Choice,
        PromptArchetype::Status => PromptKind::Status,
        _ => PromptKind::Unspecified,
    };
    Ok(PromptView {
        prompt: Some(PromptInstance {
            prompt_id: identifier.into(),
            definition: Some(definition),
            kind: kind as i32,
            values: Some(MapValue { values: question.values }),
            scope: Some(crate::host::projection::entry_scope(&snapshot, entry)?),
            created_at: Some(crate::host::projection::timestamp(entry.receipt_ns)),
        }),
        response,
        ..Default::default()
    })
}

/* === Private Functions === */

/* === Tests === */
