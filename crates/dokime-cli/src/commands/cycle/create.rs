/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Start a Cycle From Frozen Cases

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports
use prost::Message;

// Local Imports
use dokime::Metadata;
use dokime::client::CyclesCreateOptions;
use dokime_protocol::host::CasesCreateResponse;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, help = "Reuse This Identity When Retrying the Same Mutation")]
    request_id: Option<String>,

    #[arg(long)]
    cases: PathBuf,

    #[arg(long, default_value = "Cycle")]
    label: String,

    #[arg(long)]
    json: bool,

    #[command(flatten)]
    id_file: crate::commands::id_file::Args,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    if std::fs::metadata(&args.cases)?.len() > settings.retained_bytes.get() as u64 {
        return Err(crate::error::Error::Capacity("Start Cycle: Select a Smaller Case File".into()));
    }
    let cases = CasesCreateResponse::decode(std::fs::read(args.cases)?.as_slice())?;
    let origins =
        cases.cases.into_iter().map(|case| case.origin.ok_or("Start Cycle: Case Has No Frozen Origin")).collect::<Result<Vec<_>, _>>()?;
    let id_file = args.id_file.open()?;
    let client = settings.sdk(endpoint).await?;
    let options = CyclesCreateOptions {
        context: settings.context()?,
        request_id: Some(settings.request_id(args.request_id)?),
    };
    let metadata = Metadata { label: args.label, ..Default::default() };
    let value = client.cycles.create_with(origins, metadata, true, "", options).await?;
    if let Some(file) = id_file {
        file.write(value.cycle.iter().map(|cycle| &cycle.cycle_id))?;
    }
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
