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
        if request.cursor.is_some() || !request.after_prompt_id.is_empty() {
            return Err(Error::Invalid("ListPending Cursors Are Not Yet Served by the New Host".into()));
        }
        let mut prompts = Vec::new();
        for (execution, question) in context.host.pending()? {
            if request.execution_id.as_ref().is_some_and(|identifier| identifier != &execution) {
                continue;
            }
            let snapshot = context.host.store().read(&execution)?;
            if request.cycle_id.as_ref().is_some_and(|identifier| identifier != &snapshot.execution.cycle) {
                continue;
            }
            let definition = super::projection::prompt_definition(&snapshot, &question)?;
            let title = definition.metadata.as_ref().map(|metadata| metadata.label.clone()).unwrap_or_default();
            prompts.push(v1::PendingPromptIdentity {
                definition: Some(v1::PromptDefinitionIdentity {
                    prompt_type: question.prompt_type.clone(),
                    title,
                }),
                prompt_id: super::projection::prompt_id(&execution, &question.prompt),
                scope: Some(super::projection::scope(&snapshot)),
                ..Default::default()
            });
        }
        prompts.sort_by(|left, right| left.prompt_id.cmp(&right.prompt_id));
        super::projection::bounded(prompts.len(), super::projection::limit(request.limit, context.settings.page_items))?;
        Ok(v1::ListPendingPromptsReply { prompts, ..Default::default() })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
