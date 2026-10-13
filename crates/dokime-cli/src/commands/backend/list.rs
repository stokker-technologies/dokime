/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Backend Profile List

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
    #[arg(long, help = "Emit The Saved Profile Configuration As JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn run(args: Args) -> crate::Result<()> {
    let document = args.config.store()?.load()?;
    if args.json {
        println!("{}", serde_json::to_string(&document)?);
    } else {
        for profile in &document.profiles {
            let selected = if document.default.as_deref() == Some(&profile.name) {
                "*"
            } else {
                " "
            };
            println!("{selected} {}\t{}", profile.name, profile.endpoint);
        }
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
