/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompts Answers Create

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::model;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.prompts.Answers/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::prompts::AnswersCreateRequest, dokime_protocol::host::prompts::AnswersCreateResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::prompts::AnswersCreateRequest,
) -> Result<dokime_protocol::host::prompts::AnswersCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let mut normalized = request.clone();
        normalized.action = normalized.action.trim().into();
        let fingerprint = crate::host::paging::bytes(&normalized);
        let execution = request
            .parent
            .split_once('/')
            .ok_or_else(|| Error::Invalid("Create Prompt Answer: Select Its Returned Prompt Identity".into()))?
            .0;
        if let Some((saved, receipt)) = context.host.store().requested_evidence(execution, ID, &request.request_id, &fingerprint)? {
            let answer = dokime_protocol::host::PromptsCreateResponse::decode(saved.payload.as_slice())?;
            return Ok(dokime_protocol::host::prompts::AnswersCreateResponse {
                answer: Some(dokime_protocol::host::PromptResponse {
                    prompt_id: request.parent,
                    action: answer.action,
                    responder: answer.responder,
                    submitted_at: answer.submitted_at,
                    accepted: answer.accepted,
                    values: Some(dokime_protocol::model::MapValue { values: answer.values }),
                }),
                receipt: Some(receipt),
            });
        }
        let view = crate::host::prompts::catalog::view(context, &request.parent)?;
        if view.response.is_some() {
            return Err(Error::Precondition("Answer Prompt: Inspect the Already Recorded Answer with GetPrompt".into()));
        }
        let (execution, question) = crate::host::projection::pending(context, &request.parent)?;
        let mut snapshot = crate::host::execution_state::evidence(context, &execution)?;
        let cut = context.host.store().cut(&execution)?;
        for stream in crate::host::projection::schemas(&snapshot)?.streams {
            let position = context.host.store().sample_frontier(&execution, &stream.stream, 0, cut.frontier)?;
            if position > 0 {
                snapshot.samples.extend(context.host.store().sample_page(
                    &execution,
                    &stream.stream,
                    position - 1,
                    position,
                    std::num::NonZeroUsize::new(1).unwrap(),
                )?);
            }
        }
        let definition = crate::host::projection::prompt_definition(&snapshot, &question)?;
        let values = request.values.unwrap_or_default().values;
        crate::host::prompts::validation::validate(&definition, request.action.trim(), request.accepted, &values, &snapshot)?;
        let receipt = context.host.record_answer(
            &execution,
            &request.request_id,
            &fingerprint,
            dokime_protocol::host::PromptsCreateResponse {
                prompt: question.prompt,
                action: request.action.trim().into(),
                accepted: request.accepted,
                values,
                responder: "local".into(),
                submitted_at: Some(crate::host::projection::timestamp(context.host.now_ns())),
            },
        )?;
        let answer = crate::host::prompts::catalog::view(context, &request.parent)?.response;
        Ok(dokime_protocol::host::prompts::AnswersCreateResponse { answer, receipt: Some(receipt) })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
