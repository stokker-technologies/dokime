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
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn definitions(context: &Context) -> Result<Vec<v1::Prompt>> {
    let mut definitions = BTreeMap::new();
    for plugin in context.host.plugins()? {
        let Some(catalog) = plugin.description.and_then(|description| description.catalog) else {
            continue;
        };
        for section in catalog.sections {
            if section.r#type != "dokime.v1.PromptCatalog" {
                continue;
            }
            for definition in v1::PromptCatalog::decode(section.payload.as_slice())?.prompts {
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

pub(super) fn view(context: &Context, identifier: &str) -> Result<v1::PromptView> {
    let (execution, prompt) =
        identifier.split_once('/').ok_or_else(|| Error::Invalid("Read Prompt: Select an Identifier Returned by ListPending".into()))?;
    if !context.host.store().identifiers()?.iter().any(|id| id == execution) {
        return Err(Error::NotFound("Read Prompt: Select an Existing Execution".into()));
    }
    let (prompt, position) =
        prompt.rsplit_once('/').ok_or_else(|| Error::Invalid("Read Prompt: Use the Complete Pending Prompt Identifier".into()))?;
    let position = position.parse::<u64>().map_err(|_| Error::Invalid("Read Prompt: Use the Returned Question Position".into()))?;
    let snapshot = super::execution_state::evidence(context, execution)?;
    let entry = snapshot
        .evidence
        .iter()
        .find(|entry| entry.position == position && entry.kind == "prompt")
        .ok_or_else(|| Error::NotFound("Read Prompt: Select a Question Asked by This Execution".into()))?;
    let question = v1::OperatorQuestion::decode(entry.payload.as_slice())?;
    if question.prompt != prompt {
        return Err(Error::NotFound("Read Prompt: Use the Identifier Returned by ListPending".into()));
    }
    let next =
        snapshot.evidence.iter().find(|entry| entry.position > position && entry.kind == "prompt").map_or(u64::MAX, |entry| entry.position);
    let definition = super::projection::prompt_definition(&snapshot, &question)?;
    let answer = snapshot
        .evidence
        .iter()
        .rev()
        .filter(|entry| entry.position > position && entry.position < next && (entry.kind == "client:answer" || entry.kind == "answer"))
        .map(|entry| v1::OperatorAnswer::decode(entry.payload.as_slice()))
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .find(|answer| answer.prompt == prompt);
    let response = answer.map(|answer| v1::PromptResponse {
        prompt_id: identifier.into(),
        action: answer.action,
        accepted: answer.accepted,
        responder: answer.responder,
        submitted_at: answer.submitted_at,
        values: Some(v1::MapValue { values: answer.values }),
    });
    let kind = match v1::PromptArchetype::try_from(definition.archetype).unwrap_or_default() {
        v1::PromptArchetype::Confirmation => v1::PromptKind::Confirmation,
        v1::PromptArchetype::Form => v1::PromptKind::Form,
        v1::PromptArchetype::Choice => v1::PromptKind::Choice,
        v1::PromptArchetype::Status => v1::PromptKind::Status,
        _ => v1::PromptKind::Unspecified,
    };
    Ok(v1::PromptView {
        prompt: Some(v1::PromptInstance {
            prompt_id: identifier.into(),
            definition: Some(definition),
            kind: kind as i32,
            values: Some(v1::MapValue { values: question.values }),
            scope: Some(super::projection::entry_scope(&snapshot, entry)?),
            created_at: Some(super::projection::timestamp(entry.receipt_ns)),
        }),
        response,
        ..Default::default()
    })
}

/* === Private Functions === */

/* === Tests === */
