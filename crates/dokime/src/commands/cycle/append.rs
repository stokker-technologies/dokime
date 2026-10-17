/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Append Planned Cases To An Existing Cycle

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_domain::operations::execution::AppendPlannedCasesRequest;

/* === Constants === */

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
    #[command(flatten)]
    input: crate::commands::plan::input::Args,
    #[command(flatten)]
    connection: crate::connection::Args,
    #[arg(long, help = "Emit The Append Reply As JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let mut input = args.input;
    let (cycle_id, planned) = match input.input_file.take() {
        Some(path) => {
            let bytes = dokime_filesystem::input::read(&path, 16 * 1024 * 1024)?;
            let request = serde_json::from_slice::<AppendPlannedCasesRequest>(&bytes)
                .map_err(|error| crate::Error::Input(format!("invalid append input: {error}")))?;
            (
                request.cycle_id,
                crate::commands::plan::input::Planned::canonical(request.plan)?,
            )
        }
        None => (args.cycle_id.unwrap_or_default(), input.plan()?),
    };
    let id = dokime_domain::model::identity::TestCycleId::from_uuid(&cycle_id)
        .map_err(|error| crate::Error::Input(error.to_string()))?;
    let client = args.connection.connect().await?;
    let result = async {
        let plan = planned
            .resolve(&client)
            .await?
            .ok_or_else(|| crate::Error::Input("a selection plan is required".into()))?;
        Ok::<_, crate::Error>(client.cycle(id).append_cases(plan).await?)
    }
    .await;
    client.close_and_wait().await;
    let reply = result?;
    if args.json {
        println!(
            "{}",
            serde_json::json!({ "cycle_id": cycle_id, "observed_cursor": reply.observed_cursor })
        );
    } else {
        println!("Appended Cases To Cycle {cycle_id}");
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
