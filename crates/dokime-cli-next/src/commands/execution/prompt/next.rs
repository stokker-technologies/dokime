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
    let client = dokime::client::HostClient::new(channel.clone(), settings.client())?;
    let pending = client.prompts.pending(args.execution.clone()).await?;
    if let Some(prompt) = pending.prompts.first() {
        return crate::output::print(prompt, args.json);
    }
    let mut observation = dokime::v1::observation_client::ObservationClient::new(channel);
    let request = dokime::v1::ObservationWatchExecutionProgressRequest {
        test_execution_id: args.execution.clone(),
    };
    let mut events = observation.watch_execution_progress(request).await?.into_inner();
    while let Some(frame) = events.message().await? {
        if let Some(dokime::v1::observation_watch_execution_progress_output::Value::Item(frame)) = frame.value
            && frame.progress.is_some_and(|progress| progress.phase == "prompt")
        {
            let pending = client.prompts.pending(args.execution).await?;
            let prompt = pending.prompts.first().ok_or("Await Prompt: Prompt Event Has No Pending Prompt")?;
            return crate::output::print(prompt, args.json);
        }
    }
    let execution = client.executions.get(args.execution).await?;
    let reason = match execution.outcome.and_then(|outcome| outcome.attribution) {
        Some(dokime::v1::execution_outcome::Attribution::Evaluated(outcome)) => outcome.reason,
        Some(dokime::v1::execution_outcome::Attribution::NotEvaluated(outcome)) => outcome.reason,
        _ => String::new(),
    };
    Err(format!("Await Prompt: Execution Finished Without a Pending Prompt; {reason}").into())
}

/* === Private Functions === */

/* === Tests === */
