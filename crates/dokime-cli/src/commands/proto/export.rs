/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Export Authoritative Protocol Sources

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{fs, path::PathBuf};

// Third Party Imports

// Local Imports
use dokime_protocol::{EXPORT_FILES, SOURCES};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long)]
    out: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn run(args: Args) -> crate::Result<()> {
    for (relative, source) in SOURCES.iter().chain(EXPORT_FILES) {
        let path = args.out.join(relative);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, source)?;
    }
    println!("Exported {}", args.out.display());
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
