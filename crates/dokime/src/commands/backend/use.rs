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
        document.default = Some(args.name);
        Ok(())
    })?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
