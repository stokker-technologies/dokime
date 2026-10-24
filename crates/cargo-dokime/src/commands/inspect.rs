/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports
use dokime::harmos::plugin::inventory::CarrierSection;
use harmos_packaging::bundle::{Bundle, Limits};

// Local Imports
use crate::error::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
#[command(about = "Verify a plugin bundle and print its manifest and derived declarations as JSON")]
pub struct Args {
    pub bundle: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn run(args: Args) -> Result<(), Error> {
    let limits = Limits::default();
    let bundle = Bundle::inspect(&dokime_packaging::bundle::read(&args.bundle)?, limits)?;
    let mut declarations = std::collections::BTreeMap::new();
    for file in &bundle.manifest().files {
        let Some(declaration) = bundle.declaration(&file.path) else {
            continue;
        };
        if !matches!(declaration.carrier, CarrierSection::Sidecar { .. }) {
            return Err(crate::error::guest_refused(&file.path));
        }
        let value = declaration;
        declarations.insert(
            file.path.clone(),
            serde_json::json!({
                "id": value.id, "version": value.version, "description": value.description,
                "operations": value.operations
            }),
        );
    }
    serde_json::to_writer_pretty(
        std::io::stdout().lock(),
        &serde_json::json!({"manifest": bundle.manifest(), "declarations": declarations}),
    )?;
    println!();
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
