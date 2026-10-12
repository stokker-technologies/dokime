/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{TestCycleHandle, invalid};
use crate::client::{Error, ResponseStream, Result, validate_observed_cursor, wire};
use crate::core::{CommandId, PromptId, TestCycleId};

/* === Constants === */
const PAGE_SIZE: u32 = 100;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl TestCycleHandle {
    pub async fn prompt(&self, id: &PromptId) -> Result<wire::PromptView> {
        self.client.verify(true).await?;
        let view = self
            .client
            .prompts_get_prompt(wire::GetPromptRequest {
                prompt_id: id.to_string(),
            })
            .await?;
        validate_view(&self.client.backend().epoch, id, &view)?;
        if view
            .prompt
            .as_ref()
            .and_then(|prompt| prompt.scope.as_ref())
            .and_then(|scope| scope.cycle_id.as_deref())
            != Some(self.id.as_str())
        {
            return Err(Error::Rpc(tonic::Status::invalid_argument(
                "prompt belongs to another cycle",
            )));
        }
        Ok(view)
    }

    pub fn pending_prompts(&self) -> Result<ResponseStream<wire::PromptInstance>> {
        let handle = self.clone();
        ResponseStream::spawn(&self.client, move |sender| async move {
            let read = async {
                let mut request = wire::ListPendingPromptsRequest {
                    cycle_id: Some(handle.id.to_string()),
                    limit: PAGE_SIZE,
                    ..Default::default()
                };
                let selection = crate::prompts::pending_stream(Some(&handle.id), None);
                loop {
                    handle.client.verify(true).await?;
                    let page = handle.client.prompts_list_pending(request.clone()).await?;
                    validate_page(
                        &handle.client.backend().epoch,
                        &handle.id,
                        &selection,
                        &request,
                        &page,
                    )?;
                    for identity in page.prompts {
                        let id = PromptId::from_uuid(&identity.prompt_id)
                            .map_err(|_| invalid("pending prompt identity is invalid"))?;
                        let view = handle.prompt(&id).await?;
                        let prompt = view.prompt.as_ref().expect("validated prompt view");
                        validate_listed(&identity, prompt)?;
                        // Another Operator May Have Answered After The Identity Page.
                        if view.response.is_some() {
                            continue;
                        }
                        if sender
                            .send(Ok(view.prompt.expect("validated prompt view")))
                            .await
                            .is_err()
                        {
                            return Ok(());
                        }
                    }
                    match page.next_after_prompt_id {
                        Some(after) => {
                            request.after_prompt_id = after;
                            request.cursor = page.cursor;
                        }
                        None => return Ok(()),
                    }
                }
            }
            .await;
            if let Err(error) = read {
                let _ = sender.send(Err(error)).await;
            }
        })
    }

    pub async fn respond_prompt(
        &self,
        id: &PromptId,
        action: String,
        values: wire::ValueObject,
        accepted: bool,
    ) -> Result<wire::PromptResponseReceipt> {
        self.prompt(id).await?;
        self.client.verify(true).await?;
        let receipt = self
            .client
            .prompts_respond(wire::RespondToPromptRequest {
                prompt_id: id.to_string(),
                action,
                values: Some(values),
                accepted,
                context: None,
            })
            .await?;
        validate_receipt(&self.client.backend().epoch, id, &receipt)?;
        Ok(receipt)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn validate_view(epoch: &str, id: &PromptId, view: &wire::PromptView) -> Result<()> {
    validate_observed_cursor(epoch, view.observed_cursor.as_ref())?;
    let prompt = view
        .prompt
        .as_ref()
        .ok_or_else(|| invalid("prompt view has no admitted prompt"))?;
    if prompt.prompt_id != id.as_str()
        || prompt
            .definition
            .as_ref()
            .is_none_or(|definition| definition.prompt_type.is_empty())
        || view
            .response
            .as_ref()
            .is_some_and(|response| response.prompt_id != id.as_str())
    {
        return Err(invalid("prompt view differs from its admitted identity"));
    }
    Ok(())
}

fn validate_page(
    epoch: &str,
    cycle: &TestCycleId,
    selection: &str,
    request: &wire::ListPendingPromptsRequest,
    page: &wire::ListPendingPromptsReply,
) -> Result<()> {
    validate_observed_cursor(epoch, page.cursor.as_ref())?;
    if page
        .cursor
        .as_ref()
        .is_none_or(|cursor| cursor.stream != selection)
        || (request.cursor.is_some() && request.cursor != page.cursor)
        || page.prompts.len() > request.limit as usize
    {
        return Err(invalid(
            "prompt page changed selection, revision, or page bound",
        ));
    }
    let mut previous = request.after_prompt_id.as_str();
    for prompt in &page.prompts {
        PromptId::from_uuid(&prompt.prompt_id)
            .map_err(|_| invalid("pending prompt identity is invalid"))?;
        if prompt.prompt_id.as_str() <= previous
            || prompt
                .scope
                .as_ref()
                .and_then(|scope| scope.cycle_id.as_deref())
                != Some(cycle.as_str())
        {
            return Err(invalid(
                "pending prompt identity does not advance within the cycle",
            ));
        }
        previous = &prompt.prompt_id;
    }
    if let Some(after) = &page.next_after_prompt_id
        && (after.is_empty() || page.prompts.last().map(|prompt| &prompt.prompt_id) != Some(after))
    {
        return Err(invalid("pending prompt page has an invalid continuation"));
    }
    Ok(())
}

fn validate_listed(
    identity: &wire::PendingPromptIdentity,
    prompt: &wire::PromptInstance,
) -> Result<()> {
    let matches_definition = identity
        .definition
        .as_ref()
        .zip(prompt.definition.as_ref())
        .is_some_and(|(listed, actual)| {
            listed.prompt_type == actual.prompt_type
                && listed.schema_version == actual.schema_version
                && listed.title == actual.title
        });
    if prompt.prompt_id != identity.prompt_id
        || prompt.scope != identity.scope
        || prompt.created_at != identity.created_at
        || !matches_definition
    {
        return Err(invalid("prompt view differs from the listed request"));
    }
    Ok(())
}

fn validate_receipt(
    epoch: &str,
    id: &PromptId,
    receipt: &wire::PromptResponseReceipt,
) -> Result<()> {
    validate_observed_cursor(epoch, receipt.observed_cursor.as_ref())?;
    CommandId::from_uuid(&receipt.command_id)
        .map_err(|_| invalid("prompt receipt has an invalid command identity"))?;
    if receipt.prompt_id != id.as_str()
        || receipt.committed_revision == 0
        || receipt
            .observed_cursor
            .as_ref()
            .is_none_or(|cursor| cursor.position < receipt.committed_revision)
    {
        return Err(invalid(
            "prompt receipt differs from the submitted response",
        ));
    }
    Ok(())
}

/* === Tests === */
