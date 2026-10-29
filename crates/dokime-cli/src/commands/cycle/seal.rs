/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Seal A Remote Cycle And Wait For Durability

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Cycle UUID To Seal; Running Work Stops And Unrun Executions Are Cancelled.")]
    cycle_id: String,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Emit The Durable Seal Reply As JSON.")]
    json: bool,
}

pub(crate) async fn run(mut args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    args.connection = args.connection.with_settings(*settings);
    let id =
        dokime::domain::model::identity::TestCycleId::from_uuid(&args.cycle_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let client = args.connection.connect().await?;
    let result = client.cycle(id).seal().await;
    client.close_and_wait().await;
    let reply = result?;
    if args.json {
        println!("{}", serde_json::json!({"cycle_id": args.cycle_id, "sealed": true, "observed_cursor": reply.observed_cursor}));
    } else {
        println!("Cycle {} Sealed", args.cycle_id);
    }
    Ok(())
}
