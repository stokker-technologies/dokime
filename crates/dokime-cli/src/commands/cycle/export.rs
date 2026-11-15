/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Download a Self-Contained Cycle Evidence ZIP

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports

// Local Imports
use dokime::client::evidence_subject::Kind;
use dokime::client::{EvidenceArchiveOptions, EvidenceSubject};

/* === Constants === */

const ACTION: &str = "Export Cycle Evidence";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    cycle: String,

    #[arg(long, help = "New ZIP Path; an Existing File Is Refused")]
    out: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    crate::commands::download::refuse_existing(&args.out, ACTION)?;
    let client = settings.sdk(endpoint).await?;
    let subject = EvidenceSubject { kind: Some(Kind::Cycle(args.cycle)) };
    let options = EvidenceArchiveOptions {
        context: settings.context()?,
        ..Default::default()
    };
    let archive = client.evidence.archive_with(subject, options).await?;
    let reference = archive.file.ok_or_else(|| format!("{ACTION}: Missing File Reference"))?;
    crate::commands::download::save(&client, &settings, reference, &args.out, ACTION).await
}

/* === Private Functions === */

/* === Tests === */
