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
use dokime::client::{CyclePhase, CyclesSealOptions};

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
    let mut completion = client.cycles.seal_with(args.cycle, options).await?;
    let mut sealed = false;
    while let Some(frame) = completion.next().await? {
        let cycle = frame.cycle.as_ref().ok_or("Seal Cycle: Restore Its Completion State")?;
        if cycle.phase == CyclePhase::SealFailed as i32 {
            return Err(format!(
                "Seal Cycle {}: {}; Resolve the Cause and Retry Seal",
                cycle.cycle_id,
                cycle.seal_failure.as_deref().unwrap_or("Evidence Materialization Failed")
            )
            .into());
        }
        if sealed {
            return Err("Seal Cycle: Received Progress After Its Completion".into());
        }
        sealed = cycle.sealed && cycle.phase == CyclePhase::Sealed as i32 && !frame.seal.is_empty();
        crate::output::print(&frame, args.json)?;
    }
    if !sealed {
        return Err("Seal Cycle: Reconnect and Inspect Its Completion".into());
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
