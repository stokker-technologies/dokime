/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Fetch and Verify a Report Before Publishing the Output File

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports

// Local Imports
use dokime_client_internal::report_subject::Kind;
use dokime_client_internal::{ReportSubject, ReportsCreateOptions};

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
    out: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    crate::commands::download::refuse_existing(&args.out, "Fetch Report")?;
    let client = settings.sdk(endpoint).await?;
    let subject = ReportSubject { kind: Some(Kind::Cycle(args.cycle)) };
    let options = ReportsCreateOptions {
        context: settings.context()?,
        request_id: Some(settings.request_id(args.request_id)?),
    };
    let report = client.reports.create_with(subject, "", options).await?;
    let reference = report.file.ok_or("Fetch Report: Missing File Reference")?;
    crate::commands::download::save(&client, &settings, reference, &args.out, "Fetch Report").await
}

/* === Private Functions === */

/* === Tests === */
