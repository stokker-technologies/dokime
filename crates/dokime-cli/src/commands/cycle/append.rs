/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Register Explicit Execution Identities In An Existing Cycle

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::domain::model::identity::{CaseId, TestCycleId, TestExecutionId};
use dokime::operations::execution::{AppendExecutions, ExecutionBinding};

/* === Constants === */

const INPUT_BYTES: u64 = 16_777_216;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(
        required_unless_present = "input_file",
        conflicts_with = "input_file",
        help = "Existing Cycle UUID. This Command Never Creates A Cycle."
    )]
    cycle_id: Option<String>,

    #[arg(
        long,
        conflicts_with = "case_ids",
        help = "Canonical AppendExecutions JSON With Caller-Retained Execution UUIDs For Safe Retries."
    )]
    input_file: Option<std::path::PathBuf>,

    #[arg(
        long = "case-id",
        required_unless_present = "input_file",
        help = "Materialized Case UUID; Repeat To Allocate Ordered Unstarted Execution UUIDs."
    )]
    case_ids: Vec<String>,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[command(flatten)]
    recovery: crate::commands::execution::recovery::Args,

    #[arg(long, help = "Emit The Append Reply As JSON.")]
    json: bool,

    #[arg(long, conflicts_with = "json", help = "Print Each Execution UUID Created By This Append On Its Own Line.")]
    id_only: bool,

    #[command(flatten)]
    id_file: crate::commands::id_file::Args,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(mut args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    args.connection = args.connection.with_settings(*settings);
    let request = match args.input_file {
        Some(path) => {
            let bytes = dokime::harmos::filesystem::input::read(&path, INPUT_BYTES as usize)?;
            serde_json::from_slice::<AppendExecutions>(&bytes)
                .map_err(|error| crate::Error::Input(format!("invalid append input: {error}")))?
        }
        None => AppendExecutions {
            cycle_id: args.cycle_id.unwrap_or_default(),
            executions: args
                .case_ids
                .into_iter()
                .map(|case_id| ExecutionBinding { execution_id: uuid::Uuid::now_v7().to_string(), case_id })
                .collect(),
        },
    };
    let id = TestCycleId::from_uuid(&request.cycle_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let mut execution_ids = Vec::with_capacity(request.executions.len());
    for binding in &request.executions {
        CaseId::from_wire(&binding.case_id).map_err(|error| crate::Error::Input(error.to_string()))?;
        let execution = TestExecutionId::from_uuid(&binding.execution_id).map_err(|error| crate::Error::Input(error.to_string()))?;
        execution_ids.push(execution);
    }
    let recovery = args.recovery.prepare_append(&id, &execution_ids)?;
    let id_file = args.id_file.open()?;
    let cycle_id = request.cycle_id;
    // Retain Exact Retry Identities If The Admission Reply Is Lost.
    eprintln!(
        "Append Registration: {}",
        serde_json::to_string(&AppendExecutions { cycle_id: cycle_id.clone(), executions: request.executions.clone() })?
    );
    let client = args.connection.connect().await?;
    let cycle = client.cycle(id);
    let result = async { tokio::select! {
        biased;
        interrupted = crate::commands::execution::recovery::interrupted(&cycle_id, "Append Confirmation", recovery.is_some()) => {
            interrupted?;
            eprintln!("Cancellation Requested For Append Confirmation In Owned Cycle {cycle_id}.");
            if let Some(plan) = &recovery {
                plan.recover_append(&client).await?;
                return Err(crate::Error::Setup(format!("Interrupted append to cycle {cycle_id}. Collected evidence was saved; append was not retried.")));
            }
            Err(crate::Error::Setup(format!("Append confirmation for cycle {cycle_id} was interrupted; registration may have completed. Retain the exact requested execution identities.")))
        }
        appended = cycle.append(request.executions) => appended.map_err(crate::Error::from),
    }}.await;
    client.close_and_wait().await;
    let reply = result?;
    if let Some(file) = id_file {
        file.write(&reply.execution_ids)?;
    }
    if args.id_only {
        for id in reply.execution_ids {
            println!("{id}");
        }
    } else if args.json {
        println!(
            "{}",
            serde_json::json!({ "cycle_id": cycle_id, "observed_cursor": reply.observed_cursor, "execution_ids": reply.execution_ids })
        );
    } else {
        println!("Appended Executions To Cycle {cycle_id}");
        for id in reply.execution_ids {
            println!("Execution {id}");
        }
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
