/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Await an Operator Prompt Without Polling

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    execution: String,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let channel = settings.channel(endpoint, true).await?;
    let mut client = dokime::v1::prompts_client::PromptsClient::with_interceptor(channel.clone(), settings.interceptor());
    let pending = client
        .list_pending(dokime::v1::PromptsListPendingRequest {
            execution_id: Some(args.execution.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    if let Some(prompt) = pending.prompts.first() {
        return crate::output::print(prompt, args.json);
    }
    let mut observation = dokime::v1::observation_client::ObservationClient::with_interceptor(channel.clone(), settings.interceptor());
    let request = dokime::v1::ObservationWatchExecutionProgressRequest {
        test_execution_id: args.execution.clone(),
    };
    let mut events = observation.watch_execution_progress(request).await?.into_inner();
    while let Some(frame) = events.message().await? {
        if let Some(dokime::v1::observation_watch_execution_progress_output::Value::Item(frame)) = frame.value
            && frame.progress.is_some_and(|progress| progress.phase == "prompt")
        {
            let pending = client
                .list_pending(dokime::v1::PromptsListPendingRequest {
                    execution_id: Some(args.execution),
                    ..Default::default()
                })
                .await?
                .into_inner();
            let prompt = pending.prompts.first().ok_or("Await Prompt: Prompt Event Has No Pending Prompt")?;
            return crate::output::print(prompt, args.json);
        }
    }
    let mut client = dokime::v1::execution_client::ExecutionClient::with_interceptor(channel, settings.interceptor());
    let execution = client.get_execution(dokime::v1::ExecutionGetExecutionRequest { execution_id: args.execution }).await?.into_inner();
    let reason = match execution.outcome.and_then(|outcome| outcome.attribution) {
        Some(dokime::v1::execution_outcome::Attribution::Evaluated(outcome)) => outcome.reason,
        Some(dokime::v1::execution_outcome::Attribution::NotEvaluated(outcome)) => outcome.reason,
        _ => String::new(),
    };
    Err(format!("Await Prompt: Execution Finished Without a Pending Prompt; {reason}").into())
}

/* === Private Functions === */

/* === Tests === */
