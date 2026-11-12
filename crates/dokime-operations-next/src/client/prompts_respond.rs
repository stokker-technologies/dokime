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

pub const ID: &str = "dokime.prompts.respond";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PromptsRespondRequest, v1::PromptResponseReceipt>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::PromptsRespondRequest) -> Result<v1::PromptResponseReceipt> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if request.context.is_some() {
            return Err(Error::Invalid("Respond MutationContext Is Not Yet Served by the New Host".into()));
        }
        let (execution, question) = super::projection::pending(context, &request.prompt_id)?;
        context.host.answer(
            &execution,
            v1::OperatorAnswer {
                prompt: question.prompt,
                action: request.action,
                accepted: request.accepted,
                values: request.values.unwrap_or_default().values,
                responder: "local".into(),
                submitted_at: Some(super::projection::timestamp(context.host.now_ns())),
            },
        )?;
        Ok(v1::PromptResponseReceipt {
            prompt_id: request.prompt_id,
            accepted_at: Some(super::projection::timestamp(context.host.now_ns())),
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
