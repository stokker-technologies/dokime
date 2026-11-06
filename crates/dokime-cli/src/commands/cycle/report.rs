/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Download Reports Through the Shared Operation Output Path

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Cycle UUID, Running Or Sealed.")]
    id: String,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Destination HTML File.")]
    output: PathBuf,

    #[arg(long, help = "Replace An Existing File After Successful Verification.")]
    overwrite: bool,

    #[arg(long, default_value = "cycle.summary", help = "Report Identifier From The Cycle's Report Catalog.")]
    identifier: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    let request = serde_json::json!({"cycle_id": args.id, "identifier": args.identifier});
    let call = super::super::operation::call::Args {
        operation: "dokime.reports.download_cycle_report".into(),
        owner: None,
        connection: args.connection,
        input: None,
        json: Some(request.to_string()),
        output: Some(args.output),
        overwrite: args.overwrite,
        input_stream_file: None,
    };
    super::super::operation::call::run(call, settings).await
}

/* === Private Functions === */

/* === Tests === */
