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
    let mut client = dokime::v1::executions::progress_client::ProgressClient::with_interceptor(channel, settings.interceptor())
        .max_decoding_message_size(settings.message_bytes.get());
    let request = dokime::v1::executions::ProgressWatchRequest { name: args.execution, at: None };
    let mut stream = client.watch(request).await?.into_inner();
    let mut terminal = false;
    while let Some(frame) = stream.message().await? {
        if terminal {
            return Err("Watch Execution: Progress After Settlement".into());
        }
        terminal = frame.finished;
        crate::output::print(&frame, args.json)?;
        use std::io::Write;
        std::io::stdout().flush()?;
    }
    if !terminal {
        return Err("Watch Execution: Missing Finished Progress".into());
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
