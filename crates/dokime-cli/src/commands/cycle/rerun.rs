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
use std::io::Write;

// Third Party Imports

// Local Imports
use dokime::domain::model::execution::wire::RerunCycleReply;
use dokime::operations::execution::RerunCycle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Cycle UUID To Rerun; Active, Sealed And Archived Cycles Are Accepted.")]
    source_cycle_id: String,

    #[arg(long, help = "Title Of The New Cycle; Omitted Keeps The Source Title.")]
    label: Option<String>,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Emit The New Cycle Identity And Skipped Cases As JSON.")]
    json: bool,

    #[arg(long, conflicts_with = "json", help = "Print Only The New Cycle UUID For Scripts.")]
    id_only: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(mut args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    args.connection = args.connection.with_settings(*settings);
    dokime::domain::model::identity::TestCycleId::from_uuid(&args.source_cycle_id)
        .map_err(|error| crate::Error::Input(error.to_string()))?;
    let request = RerunCycle {
        source_cycle_id: args.source_cycle_id,
        label: args.label,
        context: None,
    };
    let client = args.connection.connect().await?;
    let result = client.rerun_cycle(request).await;
    client.close_and_wait().await;
    let (_, reply) = result?;
    if args.id_only {
        println!("{}", reply.cycle_id);
    } else {
        rerun(&mut std::io::stdout(), &reply, args.json)?;
    }
    std::io::stdout().flush()?;
    Ok(())
}

/* === Private Functions === */

fn rerun(output: &mut impl Write, reply: &RerunCycleReply, json: bool) -> crate::Result<()> {
    let notices = |notices: &[dokime::domain::model::execution::wire::RerunCaseNotice]| {
        notices
            .iter()
            .map(|notice| serde_json::json!({"case_id": notice.case_id, "suite_id": notice.suite_id, "reason": notice.reason}))
            .collect::<Vec<_>>()
    };
    if json {
        writeln!(
            output,
            "{}",
            serde_json::json!({
                "cycle_id": reply.cycle_id,
                "skipped": notices(&reply.skipped),
                "dropped_parameters": notices(&reply.dropped_parameters),
            })
        )?;
        return Ok(());
    }
    writeln!(output, "Created Cycle {}", reply.cycle_id)?;
    for notice in &reply.skipped {
        writeln!(output, "Skipped {} ({}): {}", notice.case_id, notice.suite_id, notice.reason)?;
    }
    for notice in &reply.dropped_parameters {
        writeln!(output, "Dropped Parameters of {} ({}): {}", notice.case_id, notice.suite_id, notice.reason)?;
    }
    Ok(())
}

/* === Tests === */
