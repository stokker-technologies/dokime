/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Watch Execution Events to Terminal Settlement

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
    let mut client = dokime::v1::observation_client::ObservationClient::with_interceptor(channel, settings.interceptor())
        .max_decoding_message_size(settings.message_bytes.get());
    let request = dokime::v1::ObservationWatchExecutionProgressRequest { test_execution_id: args.execution };
    let mut stream = client.watch_execution_progress(request).await?.into_inner();
    let mut terminal = false;
    while let Some(frame) = stream.message().await? {
        if terminal {
            return Err("Watch Execution: Received Data After Terminal Response".into());
        }
        match frame.value {
            Some(dokime::v1::observation_watch_execution_progress_output::Value::Item(frame)) => {
                let progress = frame.progress.ok_or("Watch Execution: Missing Progress")?;
                crate::output::print(&progress, args.json)?;
                use std::io::Write;
                std::io::stdout().flush()?;
            }
            Some(dokime::v1::observation_watch_execution_progress_output::Value::Response(_)) => terminal = true,
            None => return Err("Watch Execution: Missing Envelope Value".into()),
        }
    }
    if !terminal {
        return Err("Watch Execution: Missing Terminal Response".into());
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
