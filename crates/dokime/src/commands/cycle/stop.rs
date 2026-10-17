/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Request Backend Cycle Stop

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
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Cycle UUID Whose Backend Work Should Stop.")]
    cycle_id: String,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(
        long,
        help = "Emit The Stop Reply As JSON; This Does Not Wait For Sealing."
    )]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let id = dokime_domain::model::identity::TestCycleId::from_uuid(&args.cycle_id)
        .map_err(|error| crate::Error::Input(error.to_string()))?;
    let client = args.connection.connect().await?;
    let result = client.cycle(id).stop().await;
    client.close_and_wait().await;
    let reply = result?;
    if args.json {
        println!(
            "{}",
            serde_json::json!({ "cycle_id": args.cycle_id, "observed_cursor": reply.observed_cursor })
        );
    } else {
        println!("Stop Requested For Cycle {}", args.cycle_id);
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
