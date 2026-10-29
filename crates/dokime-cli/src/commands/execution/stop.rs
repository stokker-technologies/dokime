/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Request Backend Execution Stop

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Execution UUID To Stop; Other Executions Of Its Cycle Keep Running.")]
    execution_id: String,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Emit The Stop Reply As JSON; This Does Not Wait For Sealing.")]
    json: bool,
}

pub(crate) async fn run(mut args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    args.connection = args.connection.with_settings(*settings);
    let id = dokime::domain::model::identity::TestExecutionId::from_uuid(&args.execution_id)
        .map_err(|error| crate::Error::Input(error.to_string()))?;
    let client = args.connection.connect().await?;
    let result = client.execution(id).stop().await;
    client.close_and_wait().await;
    let reply = result?;
    if args.json {
        println!("{}", serde_json::json!({ "execution_id": args.execution_id, "observed_cursor": reply.observed_cursor }));
    } else {
        println!("Stop Requested For Execution {}", args.execution_id);
    }
    Ok(())
}
