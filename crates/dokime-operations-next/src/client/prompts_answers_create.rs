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
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.prompts.Answers/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::prompts::AnswersCreateRequest, v1::prompts::AnswersCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::prompts::AnswersCreateRequest) -> Result<v1::prompts::AnswersCreateResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let mut normalized = request.clone();
        normalized.action = normalized.action.trim().into();
        let fingerprint = super::paging::bytes(&normalized);
        let execution = request
            .parent
            .split_once('/')
            .ok_or_else(|| Error::Invalid("Create Prompt Answer: Select Its Returned Prompt Identity".into()))?
            .0;
        if let Some((saved, receipt)) = context.host.store().requested_evidence(execution, ID, &request.request_id, &fingerprint)? {
            let answer = dokime_protocol::context::PromptsCreateResponse::decode(saved.payload.as_slice())?;
            return Ok(v1::prompts::AnswersCreateResponse {
                answer: Some(v1::PromptResponse {
                    prompt_id: request.parent,
                    action: answer.action,
                    responder: answer.responder,
                    submitted_at: answer.submitted_at,
                    accepted: answer.accepted,
                    values: Some(v1::MapValue { values: answer.values }),
                }),
                receipt: Some(receipt),
            });
        }
        let view = super::prompt_catalog::view(context, &request.parent)?;
        if view.response.is_some() {
            return Err(Error::Precondition("Answer Prompt: Inspect the Already Recorded Answer with GetPrompt".into()));
        }
        let (execution, question) = super::projection::pending(context, &request.parent)?;
        let mut snapshot = super::execution_state::evidence(context, &execution)?;
        let cut = context.host.store().cut(&execution)?;
        for stream in super::projection::schemas(&snapshot)?.streams {
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
        let definition = super::projection::prompt_definition(&snapshot, &question)?;
        let values = request.values.unwrap_or_default().values;
        super::prompt_validation::validate(&definition, request.action.trim(), request.accepted, &values, &snapshot)?;
        let receipt = context.host.record_answer(
            &execution,
            &request.request_id,
            &fingerprint,
            dokime_protocol::context::PromptsCreateResponse {
                prompt: question.prompt,
                action: request.action.trim().into(),
                accepted: request.accepted,
                values,
                responder: "local".into(),
                submitted_at: Some(super::projection::timestamp(context.host.now_ns())),
            },
        )?;
        let answer = super::prompt_catalog::view(context, &request.parent)?.response;
        Ok(v1::prompts::AnswersCreateResponse { answer, receipt: Some(receipt) })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
