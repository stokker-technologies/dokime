/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Backend Profile Use

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    config: super::Config,

    #[arg(long, help = "Saved Backend Profile Identifier.")]
    identifier: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn run(args: Args) -> crate::Result<()> {
    args.config.store()?.update(|document| {
        dokime::client::profiles::get(document, &args.identifier)?;
        document.default = Some(args.identifier);
        Ok(())
    })?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
