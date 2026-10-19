/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Backend Profile Remove

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

    #[arg(help = "Saved Backend Profile Name.")]
    name: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn run(args: Args) -> crate::Result<()> {
    args.config.store()?.update(|document| {
        document.get(&args.name)?;
        document.profiles.retain(|profile| profile.name != args.name);
        if document.default.as_deref() == Some(&args.name) {
            document.default = None;
        }
        Ok(())
    })?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
