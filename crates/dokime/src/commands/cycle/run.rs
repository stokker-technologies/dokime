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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(
        required_unless_present = "input_file",
        conflicts_with = "input_file",
        help = "Existing Cycle UUID. This Command Never Creates A Cycle."
    )]
    cycle_id: Option<String>,
    #[arg(
        long,
        required_unless_present = "input_file",
        conflicts_with = "input_file",
        help = "Retain And Reuse This Exact Key And Cycle After An Uncertain Reply."
    )]
    request_key: Option<String>,
    #[arg(long, help = "Read Local JSON Containing cycle_id And request_key.")]
    input_file: Option<PathBuf>,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(long, help = "Emit Admission Identity As JSON.")]
    json: bool,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    cycle_id: String,
    request_key: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let input = match args.input_file {
        Some(path) => {
            let bytes = dokime_filesystem::input::read(&path, 16 * 1024)?;
            serde_json::from_slice::<Input>(&bytes)
                .map_err(|error| crate::Error::Input(format!("invalid cycle input: {error}")))?
        }
        None => Input {
            cycle_id: args
                .cycle_id
                .ok_or_else(|| crate::Error::Input("cycle ID is required".into()))?,
            request_key: args
                .request_key
                .ok_or_else(|| crate::Error::Input("request key is required".into()))?,
        },
    };
    let id = dokime_domain::model::identity::TestCycleId::from_uuid(&input.cycle_id)
        .map_err(|error| crate::Error::Input(error.to_string()))?;
    dokime_domain::model::execution::command::validate_idempotency_key(&input.request_key)
        .map_err(|error| crate::Error::Input(error.to_string()))?;
    let client = args.connection.connect().await?;
    let result = client.cycle(id).run(&input.request_key).await;
    client.close_and_wait().await;
    let admission = result?;
    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "cycle_id": input.cycle_id,
                "request_id": admission.command_id,
            })
        );
    } else {
        println!(
            "Admitted Request {} For Cycle {}",
            admission.command_id, input.cycle_id
        );
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
