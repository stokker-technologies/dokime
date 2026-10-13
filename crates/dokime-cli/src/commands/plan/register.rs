/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Register A Canonical Selection Request

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_contract::operations::planning::RegisterSelectionRequest;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    input: super::input::Args,
    #[arg(
        long,
        required_unless_present = "input_file",
        conflicts_with = "input_file",
        help = "Caller-Retained Registration Key For Recoverable Retries."
    )]
    request_key: Option<String>,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(long, help = "Emit The Canonical Reply As Compact JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let mut input = args.input;
    let request = match input.input_file.take() {
        Some(path) => {
            let bytes = dokime_filesystem::input::read(&path, 16 * 1024 * 1024)?;
            serde_json::from_slice::<RegisterSelectionRequest>(&bytes)
                .map_err(|error| crate::Error::Input(format!("invalid plan input: {error}")))?
        }
        None => RegisterSelectionRequest {
            plan: Some(input.plan()?),
            idempotency_key: args
                .request_key
                .ok_or_else(|| crate::Error::Input("request key is required".into()))?,
        },
    };
    super::input::validate(request.plan.as_ref())?;
    dokime_contract::core::command::validate_idempotency_key(&request.idempotency_key)
        .map_err(|error| crate::Error::Input(error.to_string()))?;
    let client = args.connection.connect().await?;
    let result = client.register_selection(request).await;
    client.close_and_wait().await;
    super::input::print(&result?, args.json)
}

/* === Private Functions === */

/* === Tests === */
