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
use std::path::PathBuf;

// Third Party Imports

// Local Imports
use dokime_client::{dynamic::Event, operations::MAX_INPUT_BYTES};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Operation Identifier From `operation list`.")]
    operation: String,
    #[arg(long, help = "Select The Discovered Provider Owner When More Than One Provider Exists.")]
    owner: Option<String>,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(long, help = "Read Request JSON; Defaults To An Empty Object.")]
    input_file: Option<PathBuf>,
    #[arg(long, help = "Read Input Stream Items As One JSON Value Per Line; Bounded To 16 MiB In Total.")]
    input_stream_file: Option<PathBuf>,
    #[arg(long, help = "Emit Compact JSON. Streamed Output Uses One Item Or Completion Per Line.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let input = match args.input_file {
        Some(path) => harmos::filesystem::input::read(&path, MAX_INPUT_BYTES)?,
        None => b"{}".to_vec(),
    };
    // Reject Malformed JSON Before Selecting Or Opening A Connection.
    let _: serde_json::Value = serde_json::from_slice(&input).map_err(|error| crate::Error::Input(error.to_string()))?;
    let items = match args.input_stream_file {
        None => Vec::new(),
        Some(path) => {
            let bytes = harmos::filesystem::input::read(&path, MAX_INPUT_BYTES.saturating_sub(input.len()))?;
            let text = std::str::from_utf8(&bytes).map_err(|_| crate::Error::Input("Input stream must be UTF-8 JSON lines".into()))?;
            let mut items = Vec::new();
            for line in text.lines().filter(|line| !line.trim().is_empty()) {
                let _: serde_json::Value = serde_json::from_str(line).map_err(|error| crate::Error::Input(error.to_string()))?;
                items.push(line.as_bytes().to_vec());
            }
            items
        }
    };
    let client = args.connection.connect().await?;
    let result = async {
        let stream = client.call_operation(&args.operation, args.owner.as_deref(), &input, items).await?;
        let result = async {
            let interrupt = tokio::signal::ctrl_c();
            tokio::pin!(interrupt);
            loop {
                let event = tokio::select! {
                    biased;
                    interrupted = &mut interrupt => {
                        interrupted?;
                        break;
                    }
                    event = stream.message() => event?,
                };
                let Some(event) = event else { break };
                let value = match event {
                    Event::Item(value) => serde_json::json!({"event": "item", "value": value}),
                    Event::Completion(value) => {
                        serde_json::json!({"event": "completion", "value": value})
                    }
                };
                if args.json {
                    println!("{}", serde_json::to_string(&value)?);
                } else {
                    println!("{}", serde_json::to_string_pretty(&value)?);
                }
            }
            Ok::<_, crate::Error>(())
        }
        .await;
        stream.close().await;
        result
    }
    .await;
    client.close_and_wait().await;
    result
}

/* === Private Functions === */

/* === Tests === */
