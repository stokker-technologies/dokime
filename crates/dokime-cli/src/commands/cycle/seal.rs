/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Submit an Execution Control Through the SDK

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_client_internal::{ArchiveScope, CycleFilter, CyclePhase, CycleSelection, CyclesSealOptions, CyclesWatchOptions};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, help = "Reuse This Identity When Retrying the Same Mutation")]
    request_id: Option<String>,

    cycle: String,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let client = settings.sdk(endpoint).await?;
    let options = CyclesSealOptions {
        context: settings.context()?,
        request_id: Some(settings.request_id(args.request_id)?),
    };
    let name = args.cycle;
    let admission = client.cycles.seal_with(name.clone(), options).await?;
    crate::output::print(&admission, args.json)?;
    if admission.cycle.as_ref().is_some_and(|cycle| cycle.sealed) {
        return Ok(());
    }
    let selection = CycleSelection {
        cycle_ids: vec![name.clone()],
        ..Default::default()
    };
    let options = CyclesWatchOptions {
        context: settings.context()?,
        ..Default::default()
    };
    let mut completion = client
        .cycles
        .watch_with(
            selection,
            CycleFilter {
                archive: Some(ArchiveScope::All as i32),
                ..Default::default()
            },
            Default::default(),
            options,
        )
        .await?;
    while let Some(frame) = completion.next().await? {
        if let Some(failure) = frame.resync_required {
            return Err(failure.message.into());
        }
        let cycle = frame.cycles.iter().find(|cycle| cycle.cycle_id == name).ok_or("Seal Cycle: Restore Its Completion State")?;
        if cycle.phase == CyclePhase::SealFailed as i32 {
            return Err(cycle
                .seal_failure
                .clone()
                .unwrap_or_else(|| "Seal Cycle: Resolve the Materialization Failure and Retry Seal".into())
                .into());
        }
        if cycle.sealed && cycle.phase == CyclePhase::Sealed as i32 {
            crate::output::print(cycle, args.json)?;
            return Ok(());
        }
    }
    Err("Seal Cycle: Reconnect and Inspect Its Completion".into())
}

/* === Private Functions === */

/* === Tests === */
