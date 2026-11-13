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
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.prompts.list_pending";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PromptsListPendingRequest, v1::ListPendingPromptsReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::PromptsListPendingRequest) -> Result<v1::ListPendingPromptsReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let mut prompts = Vec::new();
        for (execution, question) in context.host.pending()? {
            if request.execution_id.as_ref().is_some_and(|identifier| identifier != &execution) {
                continue;
            }
            let snapshot = super::execution_state::evidence(context, &execution)?;
            if request.cycle_id.as_ref().is_some_and(|identifier| identifier != &snapshot.execution.cycle) {
                continue;
            }
            let definition = super::projection::prompt_definition(&snapshot, &question)?;
            let title = definition.metadata.as_ref().map(|metadata| metadata.label.clone()).unwrap_or_default();
            let question_entry = snapshot
                .evidence
                .iter()
                .rev()
                .find(|entry| {
                    entry.kind == "prompt"
                        && v1::OperatorQuestion::decode(entry.payload.as_slice()).is_ok_and(|stored| stored.prompt == question.prompt)
                })
                .ok_or_else(|| Error::Precondition("List Pending Prompts: Resynchronize After the Question Changed".into()))?;
            prompts.push(v1::PendingPromptIdentity {
                definition: Some(v1::PromptDefinitionIdentity {
                    prompt_type: question.prompt_type.clone(),
                    title,
                }),
                prompt_id: super::projection::prompt_id(&execution, &question.prompt, question_entry.position),
                scope: Some(super::projection::entry_scope(&snapshot, question_entry)?),
                created_at: Some(super::projection::timestamp(question_entry.receipt_ns)),
            });
        }
        prompts.sort_by(|left, right| left.prompt_id.cmp(&right.prompt_id));
        Ok(v1::ListPendingPromptsReply { prompts })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
