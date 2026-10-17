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
use dokime_domain::model::identity::Identifier;
use dokime_domain::operations::common::Metadata;
use dokime_domain::operations::execution::CreateCycleRequest;
use dokime_domain::operations::planning::SelectionPlan;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(long, conflicts_with_all = ["identifier", "label", "description", "continue_on_failure"], help = "Read The Canonical CreateCycleRequest JSON Without Flag Overrides.")]
    input_file: Option<PathBuf>,
    #[arg(long, required_unless_present = "input_file", help = "Cycle Title.")]
    label: Option<String>,
    #[arg(long, help = "Cycle Identifier; Omitted Lets The Backend Derive One.")]
    identifier: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    continue_on_failure: bool,
    #[arg(
        long = "case-id",
        help = "Registered Case ID To Append After Creation; Repeat To Preserve Order."
    )]
    case_ids: Vec<String>,
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
    let request = match args.input_file {
        Some(path) => {
            let bytes = dokime_filesystem::input::read(&path, 16 * 1024 * 1024)?;
            serde_json::from_slice::<CreateCycleRequest>(&bytes)
                .map_err(|error| crate::Error::Input(format!("invalid cycle input: {error}")))?
        }
        None => CreateCycleRequest {
            metadata: Some(Metadata {
                identifier: args.identifier.unwrap_or_default(),
                label: args
                    .label
                    .ok_or_else(|| crate::Error::Input("label is required".into()))?,
                description: args.description,
                tags: Vec::new(),
                attributes: Vec::new(),
                created_at: None,
                updated_at: None,
            }),
            continue_on_failure: args.continue_on_failure,
            context: None,
        },
    };
    if let Some(metadata) = &request.metadata
        && !metadata.identifier.is_empty()
    {
        Identifier::new(metadata.identifier.clone())
            .map_err(|error| crate::Error::Input(error.to_string()))?;
    }
    for id in &args.case_ids {
        dokime_domain::model::identity::CaseId::from_wire(id)
            .map_err(|error| crate::Error::Input(error.to_string()))?;
    }
    if let Some(key) = &args.request_key {
        dokime_domain::model::execution::command::validate_idempotency_key(key)
            .map_err(|error| crate::Error::Input(error.to_string()))?;
    }
    let client = args.connection.connect().await?;
    let result = async {
        let cycle = client.create_cycle(request).await?;
        // Expose Recovery Identity Before Starting Work.
        created(
            &mut std::io::stdout(),
            cycle.id().as_str(),
            args.request_key.as_deref(),
            args.json,
        )?;
        if !args.case_ids.is_empty() {
            cycle
                .append_cases(SelectionPlan {
                    selections: Vec::new(),
                    max_cases: 0,
                    case_ids: args.case_ids,
                    cycle_id: String::new(),
                })
                .await?;
        }
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
                let id = dokime_domain::model::identity::CommandId::from_uuid(&admission.command_id)
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
