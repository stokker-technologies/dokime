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
use dokime_domain::operations::planning::RegisterSelection;

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
    // === Read Request === //

    let mut input = args.input;
    let (planned, idempotency_key) = match input.input_file.take() {
        Some(path) => {
            let bytes = harmos::filesystem::input::read(&path, 16 * 1024 * 1024)?;
            let request = serde_json::from_slice::<RegisterSelection>(&bytes)
                .map_err(|error| crate::Error::Input(format!("invalid plan input: {error}")))?;
            (super::input::Planned::canonical(request.plan)?, request.idempotency_key)
        }
        None => (input.plan()?, args.request_key.ok_or_else(|| crate::Error::Input("request key is required".into()))?),
    };
    dokime_domain::model::execution::command::validate_idempotency_key(&idempotency_key)
        .map_err(|error| crate::Error::Input(error.to_string()))?;

    // === Resolve And Register === //

    let client = args.connection.connect().await?;
    let result = async {
        let plan = planned.resolve(&client).await?;
        let reply = client.register_selection(RegisterSelection { plan, idempotency_key }).await?;
        super::input::print(&client, &reply, args.json).await
    }
    .await;
    client.close_and_wait().await;
    result
}

/* === Private Functions === */

/* === Tests === */
