/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompts List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Prompts/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::PromptsListRequest, dokime_protocol::host::PromptsListResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::PromptsListRequest,
) -> Result<dokime_protocol::host::PromptsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        crate::host::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        if request.pending_only && request.page.is_some() {
            return Err(Error::Invalid("List Prompts: Pending Prompts Return Whole Without Page Controls".into()));
        }
        let key = format!("prompts/{}/{:?}/{:?}/{}", request.parent, request.cycle_id, request.execution_id, request.pending_only);
        let (saved, cut) = crate::host::views::read(context, &key, request.at.as_ref(), async {
            let mut prompts = Vec::new();
            for id in context.host.store().identifiers()? {
                if request.execution_id.as_ref().is_some_and(|selected| selected != &id)
                    || !request.parent.is_empty() && request.parent != id
                {
                    continue;
                }
                let snapshot = crate::host::execution_state::evidence(context, &id)?;
                if request.cycle_id.as_ref().is_some_and(|selected| selected != &snapshot.execution.cycle) {
                    continue;
                }
                for entry in &snapshot.evidence {
                    if entry.kind != "prompt" {
                        continue;
                    }
                    let question = dokime_protocol::host::PromptsCreateRequest::decode(entry.payload.as_slice())?;
                    let name = crate::host::projection::prompt_id(&id, &question.prompt, entry.position);
                    let view = crate::host::prompts::catalog::view(context, &name)?;
                    if request.pending_only && view.response.is_some() {
                        continue;
                    }
                    if request.pending_only
                        && !context.host.pending()?.iter().any(|(execution, pending)| execution == &id && pending.prompt == question.prompt)
                    {
                        continue;
                    }
                    prompts.push(view);
                }
            }
            Ok(dokime_protocol::host::PromptsListResponse { prompts, ..Default::default() })
        })
        .await?;
        let (prompts, next_page_token) = if request.pending_only {
            (saved.prompts, String::new())
        } else {
            crate::host::views::page(
                saved.prompts,
                &cut,
                request.page.as_ref(),
                context.settings.page_items,
                context.settings.message_bytes,
            )?
        };
        crate::host::bounded_reply(
            context,
            dokime_protocol::host::PromptsListResponse {
                prompts,
                resolved_at: Some(cut),
                next_page_token,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
