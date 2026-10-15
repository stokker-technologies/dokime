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
use std::{io::Write, path::PathBuf, time::Duration};

// Third Party Imports

// Local Imports
use dokime_contract::core::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(long, conflicts_with_all = ["case_ids", "identifier", "label", "continue_on_failure"], help = "Read The Canonical CreateCycleRequest JSON Without Flag Overrides.")]
    input_file: Option<PathBuf>,
    #[arg(
        long = "case-id",
        required_unless_present = "input_file",
        help = "Registered Case ID; Repeat To Preserve Order And Repeated Attempts."
    )]
    case_ids: Vec<String>,
    #[arg(long, required_unless_present = "input_file")]
    identifier: Option<String>,
    #[arg(long, required_unless_present = "input_file")]
    label: Option<String>,
    #[arg(long)]
    continue_on_failure: bool,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(
        long,
        help = "Submit The Created Cycle Once Using This Caller-Retained Key."
    )]
    request_key: Option<String>,
    #[arg(long, requires = "request_key", value_parser = clap::value_parser!(u64).range(1..=86400), help = "Wait For A Terminal Receipt After Submission.")]
    wait_seconds: Option<u64>,
    #[arg(
        long,
        help = "Emit Creation, Admission, And Optional Receipt As JSON Lines."
    )]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let mut request = match args.input_file {
        Some(path) => {
            let bytes = dokime_filesystem::input::read(&path, 16 * 1024 * 1024)?;
            serde_json::from_slice::<dokime_contract::operations::execution::CreateCycleRequest>(
                &bytes,
            )
            .map_err(|error| crate::Error::Input(format!("invalid cycle input: {error}")))?
        }
        None => dokime_contract::operations::execution::CreateCycleRequest {
            plan: Some(dokime_contract::operations::planning::SelectionPlan {
                selections: Vec::new(),
                max_cases: 0,
                case_ids: args.case_ids,
                cycle_id: String::new(),
            }),
            metadata: Some(dokime_contract::operations::common::Metadata {
                identifier: args
                    .identifier
                    .ok_or_else(|| crate::Error::Input("identifier is required".into()))?,
                label: args
                    .label
                    .ok_or_else(|| crate::Error::Input("label is required".into()))?,
                description: None,
                tags: Vec::new(),
                attributes: Vec::new(),
                created_at: None,
                updated_at: None,
            }),
            continue_on_failure: args.continue_on_failure,
            context: None,
        },
    };
    // Selections name templates by their readable ID, as `plan` does.
    let planned = crate::commands::plan::input::Planned::canonical(request.plan.take())?;
    if let Some(metadata) = &request.metadata {
        Identifier::new(metadata.identifier.clone())
            .map_err(|error| crate::Error::Input(error.to_string()))?;
    }
    if let Some(key) = &args.request_key {
        dokime_contract::core::command::validate_idempotency_key(key)
            .map_err(|error| crate::Error::Input(error.to_string()))?;
    }
    let client = args.connection.connect().await?;
    let result = async {
        request.plan = planned.resolve(&client).await?;
        let cycle = client.create_cycle(request).await?;
        // Expose Recovery Identity Before Starting Work.
        created(
            &mut std::io::stdout(),
            cycle.id().as_str(),
            args.request_key.as_deref(),
            args.json,
        )?;
        if let Some(key) = args.request_key {
            let admission = cycle.run(&key).await?;
            {
                let mut output = std::io::stdout();
                if args.json {
                    writeln!(
                        output,
                        "{}",
                        serde_json::json!({
                            "cycle_id": cycle.id().as_str(), "request_id": admission.command_id,
                        })
                    )?;
                } else {
                    writeln!(
                        output,
                        "Admitted Request {} For Cycle {}",
                        admission.command_id,
                        cycle.id()
                    )?;
                }
                output.flush()?;
            }
            if let Some(seconds) = args.wait_seconds {
                let id = dokime_contract::core::CommandId::from_uuid(&admission.command_id)
                    .map_err(|error| crate::Error::Setup(error.to_string()))?;
                let receipt = cycle
                    .request(id)
                    .wait(Some(Duration::from_secs(seconds)), None)
                    .await?;
                super::request::print(&receipt, args.json)?;
            }
        }
        Ok(())
    }
    .await;
    client.close_and_wait().await;
    result
}

/* === Private Functions === */
fn created(
    output: &mut impl Write,
    cycle: &str,
    key: Option<&str>,
    json: bool,
) -> crate::Result<()> {
    if json {
        let mut value = serde_json::json!({"cycle_id": cycle});
        if let Some(key) = key {
            value["request_key"] = key.into();
        }
        writeln!(output, "{value}")?;
    } else {
        writeln!(output, "Created Cycle {cycle}")?;
        if let Some(key) = key {
            writeln!(output, "Run Request Key {key}")?;
        }
    }
    output.flush()?;
    Ok(())
}

/* === Tests === */
