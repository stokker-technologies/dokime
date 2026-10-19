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
use std::{io::Write, path::PathBuf};

// Third Party Imports

// Local Imports
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::wire::Metadata;
use dokime_domain::operations::execution::CreateCycle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(long, conflicts_with_all = ["identifier", "label", "description", "continue_automatically", "continue_on_failure"], help = "Read The Canonical CreateCycle JSON Without Flag Overrides.")]
    input_file: Option<PathBuf>,

    #[arg(long, required_unless_present = "input_file", help = "Cycle Title.")]
    label: Option<String>,

    #[arg(long, help = "Cycle Identifier; Omitted Lets The Backend Derive One.")]
    identifier: Option<String>,

    #[arg(long)]
    description: Option<String>,

    #[arg(long)]
    continue_on_failure: bool,

    #[arg(long, help = "Let The Server Continue With The Next Unstarted Member After A Durable Seal.")]
    continue_automatically: bool,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Emit The Created Cycle Identity As JSON.")]
    json: bool,

    #[arg(long, conflicts_with = "json", help = "Print Only The Created Cycle UUID For Scripts.")]
    id_only: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let request = match args.input_file {
        Some(path) => {
            let bytes = harmos::filesystem::input::read(&path, 16 * 1024 * 1024)?;
            serde_json::from_slice::<CreateCycle>(&bytes).map_err(|error| crate::Error::Input(format!("invalid cycle input: {error}")))?
        }
        None => CreateCycle {
            metadata: Some(Metadata {
                identifier: args.identifier.unwrap_or_default(),
                label: args.label.ok_or_else(|| crate::Error::Input("label is required".into()))?,
                description: args.description,
                tags: Vec::new(),
                attributes: Vec::new(),
                created_at: None,
                updated_at: None,
            }),
            continue_automatically: args.continue_automatically,
            continue_on_failure: args.continue_on_failure,
            context: None,
        },
    };
    if let Some(metadata) = &request.metadata
        && !metadata.identifier.is_empty()
    {
        Identifier::new(metadata.identifier.clone()).map_err(|error| crate::Error::Input(error.to_string()))?;
    }
    let client = args.connection.connect().await?;
    let result = async {
        let cycle = client.create_cycle(request).await?;
        // Expose Recovery Identity Before Starting Work.
        if args.id_only {
            println!("{}", cycle.id());
            std::io::stdout().flush()?;
        } else {
            created(&mut std::io::stdout(), cycle.id().as_str(), args.json)?;
        }
        Ok(())
    }
    .await;
    client.close_and_wait().await;
    result
}

/* === Private Functions === */

fn created(output: &mut impl Write, cycle: &str, json: bool) -> crate::Result<()> {
    if json {
        writeln!(output, "{}", serde_json::json!({"cycle_id": cycle}))?;
    } else {
        writeln!(output, "Created Cycle {cycle}")?;
    }
    output.flush()?;
    Ok(())
}

/* === Tests === */
