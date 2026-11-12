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
        let view = super::prompt_catalog::view(context, &request.prompt_id)?;
        if view.response.is_some() { return Err(Error::Conflict("Answer Prompt: Inspect the Already Recorded Answer with GetPrompt".into())); }
        let (execution, question) = super::projection::pending(context, &request.prompt_id)?;
        let snapshot = context.host.store().read(&execution)?;
        let definition = super::projection::prompt_definition(&snapshot, &question)?;
        let values = request.values.unwrap_or_default().values;
        super::prompt_validation::validate(&definition, request.action.trim(), request.accepted, &values, &snapshot)?;
        context.host.record_answer(
            &execution,
            v1::OperatorAnswer {
                prompt: question.prompt,
                action: request.action.trim().into(),
                accepted: request.accepted,
                values,
                responder: "local".into(),
                submitted_at: Some(super::projection::timestamp(context.host.now_ns())),
            },
        )?;
        Ok(v1::PromptResponseReceipt {
            command_id: request.prompt_id.clone(),
            prompt_id: request.prompt_id,
            accepted_at: Some(super::projection::timestamp(context.host.now_ns())),
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
