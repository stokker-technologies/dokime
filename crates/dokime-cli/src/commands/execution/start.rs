/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Admit One Execution By Its Durable Identity

use dokime::domain::model::execution::ExecutionMode;
use dokime::domain::model::identity::TestExecutionId;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Pending Execution UUID To Admit.")]
    execution_id: String,

    #[arg(long, value_parser = ["automatic", "interactive"], default_value = "automatic", help = "Execution Mode; Retries Must Use The Same Mode.")]
    mode: String,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[command(flatten)]
    recovery: super::recovery::Args,

    #[arg(long, help = "Emit The Admitted Execution Identity As JSON.")]
    json: bool,
}

pub(crate) async fn run(mut args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    args.connection = args.connection.with_settings(*settings);
    let id = TestExecutionId::from_uuid(&args.execution_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let mode = if args.mode == "interactive" { ExecutionMode::Interactive } else { ExecutionMode::Automatic };
    let recovery = args.recovery.prepare(&id)?;
    let client = args.connection.connect().await?;
    let execution = client.execution(id);
    let operation = async { execution.start(mode).await.map_err(crate::Error::from) };
    let result = super::recovery::observe(&client, &execution, recovery.as_ref(), "Start Confirmation", operation).await;
    client.close_and_wait().await;
    result?;

    if args.json {
        println!("{}", serde_json::json!({"execution_id": args.execution_id}));
    } else {
        println!("Execution {} Started Or Already Admitted", args.execution_id);
    }
    Ok(())
}
