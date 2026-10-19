/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Admit One Execution With A Retained Request Key

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

    #[arg(long, help = "Retain And Reuse This Key And Execution After An Uncertain Reply.")]
    request_key: String,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Emit Execution And Request Identities As JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let id = TestExecutionId::from_uuid(&args.execution_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    dokime_domain::model::execution::command::validate_idempotency_key(&args.request_key)
        .map_err(|error| crate::Error::Input(error.to_string()))?;

    let client = args.connection.connect().await?;
    let result = client.execution(id).start(&args.request_key, ExecutionMode::Automatic).await;
    client.close_and_wait().await;
    let admission = result?;

    if args.json {
        println!("{}", serde_json::json!({"execution_id": args.execution_id, "request_id": admission.command_id}));
    } else {
        println!("Admitted Request {} For Execution {}", admission.command_id, args.execution_id);
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
