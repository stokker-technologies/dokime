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
use dokime_client_internal::CyclesCreateOptions;
use dokime::Metadata;

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

    #[command(flatten)]
    id_file: crate::commands::id_file::Args,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let id_file = args.id_file.open()?;
    let client = settings.sdk(endpoint).await?;
    let options = CyclesCreateOptions {
        context: settings.context()?,
        request_id: Some(settings.request_id(args.request_id)?),
    };
    let source = client.cycles.get(args.cycle.clone()).await?.cycle.ok_or("Rerun Cycle: Read Its Source")?;
    let metadata = source.metadata.unwrap_or_else(|| Metadata {
        label: args.cycle.clone(),
        ..Default::default()
    });
    let value = client.cycles.create_with(Vec::new(), metadata, true, args.cycle, options).await?;
    if let Some(file) = id_file {
        file.write(value.cycle.iter().map(|cycle| &cycle.cycle_id))?;
    }
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
