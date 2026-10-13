/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Preview A Canonical Selection Request

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_contract::operations::planning::PreviewSelectionRequest;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    input: super::input::Args,
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
            serde_json::from_slice::<PreviewSelectionRequest>(&bytes)
                .map_err(|error| crate::Error::Input(format!("invalid plan input: {error}")))?
        }
        None => PreviewSelectionRequest {
            plan: Some(input.plan()?),
        },
    };
    super::input::validate(request.plan.as_ref())?;
    let client = args.connection.connect().await?;
    let result = client.preview_selection(request).await;
    client.close_and_wait().await;
    super::input::print(&result?, args.json)
}

/* === Private Functions === */

/* === Tests === */
