/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Durable Request UUID Returned By Cycle Run.")]
    request_id: String,
    #[arg(long, help = "Require The Receipt To Belong To This Cycle UUID.")]
    cycle_id: Option<String>,
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..=86400), help = "Wait Up To This Many Seconds For A Terminal Receipt.")]
    wait_seconds: Option<u64>,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(long, help = "Emit Request Progress As JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let id = dokime::core::CommandId::from_uuid(&args.request_id)
        .map_err(|error| crate::Error::Input(error.to_string()))?;
    let cycle = args
        .cycle_id
        .map(dokime::core::TestCycleId::from_uuid)
        .transpose()
        .map_err(|error| crate::Error::Input(error.to_string()))?;
    let client = args.connection.connect().await?;
    let request = match cycle {
        Some(cycle) => client.cycle(cycle).request(id),
        None => client.execution_request(id),
    };
    let result = match args.wait_seconds {
        Some(seconds) => request.wait(Some(Duration::from_secs(seconds)), None).await,
        None => request.status().await,
    };
    client.close_and_wait().await;
    let receipt = result?;
    print(&receipt, args.json)
}

pub(super) fn print(
    receipt: &dokime::client::wire::RunRequestReceipt,
    json: bool,
) -> crate::Result<()> {
    let phase = dokime::client::wire::RunRequestPhase::try_from(receipt.phase)
        .map_err(|_| crate::Error::Setup("invalid native request phase".into()))?;
    if json {
        println!(
            "{}",
            serde_json::json!({
                "request_id": receipt.request_id,
                "cycle_id": receipt.cycle_id,
                "phase": phase.as_str_name(),
                "paused": receipt.paused,
                "stop_requested": receipt.stop_requested,
                "planned_count": receipt.planned_count,
                "finished_count": receipt.finished_count,
                "failed_count": receipt.failed_count,
                "unstarted_count": receipt.unstarted_count,
                "interruption": receipt.interruption,
            })
        );
    } else {
        println!(
            "Request {}: {phase:?} ({}/{} Finished, {} Failed)",
            receipt.request_id, receipt.finished_count, receipt.planned_count, receipt.failed_count
        );
        println!("Cycle {}", receipt.cycle_id);
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
