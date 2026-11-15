/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Write Confirmed Identities for Scripts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{fmt::Display, fs::File, io::Write, path::PathBuf};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, Default, clap::Args)]
#[group(id = "identity_file")]
pub(crate) struct Args {
    #[arg(long, help = "Also Write Each Confirmed UUID on Its Own Line to This File, Created or Truncated Before Connecting")]
    id_file: Option<PathBuf>,
}

/// An Identity File Opened Before Any Mutation and Written Once the Host Confirms It.
pub(crate) struct IdFile(File);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Args {
    pub(crate) fn open(&self) -> crate::Result<Option<IdFile>> {
        let Some(path) = &self.id_file else {
            return Ok(None);
        };
        let file = File::create(path)
            .map_err(|error| crate::error::Error::Invalid(format!("Create Identity File {}: {error}", path.display())))?;
        Ok(Some(IdFile(file)))
    }
}

impl IdFile {
    pub(crate) fn write<I: Display>(mut self, ids: impl IntoIterator<Item = I>) -> crate::Result<()> {
        for id in ids {
            writeln!(self.0, "{id}")?;
        }
        self.0.sync_all()?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
