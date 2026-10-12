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
use dokime::client::operations::{MAX_INPUT_BYTES, Request};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Operation Identifier From `operation list`.")]
    operation: String,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(
        long,
        help = "Read And Validate Request JSON Locally; Defaults To JSON Null."
    )]
    input_file: Option<PathBuf>,
    #[arg(long, help = "Emit Compact JSON Instead Of Indented JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let input = match args.input_file {
        Some(path) => crate::input::read(&path, MAX_INPUT_BYTES)?,
        None => b"{}".to_vec(),
    };
    let request = Request::from_json(&args.operation, &input)?;
    drop(input);
    let client = args.connection.connect().await?;
    let result = client.invoke(request).await;
    client.close_and_wait().await;
    let response = result?;
    let output = if args.json {
        serde_json::to_string(&response)?
    } else {
        serde_json::to_string_pretty(&response)?
    };
    println!("{output}");
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
