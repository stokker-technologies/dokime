/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Append And Admit A Case Re-Run

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
    #[arg(help = "Cycle UUID That Owns The Case.")]
    cycle_id: String,
    #[arg(help = "Case UUID Whose Next Attempt Is Appended As A New Execution.")]
    case_id: String,
    #[arg(
        long,
        help = "Caller-Retained Key That Admits The New Attempt Exactly Once."
    )]
    key: String,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(long, help = "Emit The Admission As JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let id = dokime_contract::core::TestCycleId::from_uuid(&args.cycle_id)
        .map_err(|error| crate::Error::Input(error.to_string()))?;
    let client = args.connection.connect().await?;
    let cycle = client.cycle(id);

    // === Append The Attempt, Then Admit It === //

    let result = async {
        cycle.schedule_rerun(&args.case_id).await?;
        cycle.run_case(&args.case_id, &args.key).await
    }
    .await;
    client.close_and_wait().await;
    let admission = result?;
    if args.json {
        println!(
            "{}",
            serde_json::json!({ "cycle_id": args.cycle_id, "case_id": args.case_id, "request_id": admission.command_id })
        );
    } else {
        println!(
            "Re-Run Admitted For Case {} As Request {}",
            args.case_id, admission.command_id
        );
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
