/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Admit One Execution By Its Durable Identity

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_domain::model::execution::ExecutionMode;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
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

/* === Private Functions === */

/* === Tests === */
