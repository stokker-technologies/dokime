/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Start An Execution And Observe Its Durable Completion

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::domain::model::execution::ExecutionMode;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    waiting: super::wait::Args,

    #[arg(long, value_parser = ["automatic", "interactive"], default_value = "automatic", help = "Execution Mode; Retries Must Use The Same Mode.")]
    mode: String,

    #[arg(long, help = "Print All Captured Records; --record-type Restricts This Selection.")]
    records: bool,

    #[arg(long, help = "Print All Captured Samples; --stream Restricts This Selection.")]
    samples: bool,

    #[arg(long = "record-type", help = "Case-Sensitive Whole-Identifier Glob; Repeat For OR Selection.")]
    record_types: Vec<String>,

    #[arg(long = "stream", help = "Case-Sensitive Whole-Identifier Glob; Repeat For OR Selection.")]
    streams: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let mode = if args.mode == "interactive" { ExecutionMode::Interactive } else { ExecutionMode::Automatic };
    let record_types = (args.records || !args.record_types.is_empty()).then_some(args.record_types);
    let streams = (args.samples || !args.streams.is_empty()).then_some(args.streams);
    let options = dokime::client::RunOptions { mode, record_types, streams, ..Default::default() };
    options.validate().map_err(|error| crate::Error::Input(error.to_string()))?;
    super::wait::execute(args.waiting, Some(options)).await
}

/* === Private Functions === */

/* === Tests === */
