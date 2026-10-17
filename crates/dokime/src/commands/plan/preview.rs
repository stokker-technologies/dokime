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
use dokime_domain::operations::planning::PreviewSelection;

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
    // === Read Request === //

    let mut input = args.input;
    let planned = match input.input_file.take() {
        Some(path) => {
            let bytes = harmos::filesystem::input::read(&path, 16 * 1024 * 1024)?;
            let request = serde_json::from_slice::<PreviewSelection>(&bytes)
                .map_err(|error| crate::Error::Input(format!("invalid plan input: {error}")))?;
            super::input::Planned::canonical(request.plan)?
        }
        None => input.plan()?,
    };

    // === Resolve And Preview === //

    let client = args.connection.connect().await?;
    let result = async {
        let plan = planned.resolve(&client).await?;
        let reply = client.preview_selection(PreviewSelection { plan }).await?;
        super::input::print(&client, &reply, args.json).await
    }
    .await;
    client.close_and_wait().await;
    result
}

/* === Private Functions === */

/* === Tests === */
