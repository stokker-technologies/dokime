/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Hand Confirmed Identities To Scripts Without Capturing Standard Output

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
    #[arg(
        long,
        help = "Also Write Each Confirmed UUID On Its Own Line To This File (Created Or Truncated Before Connecting), So Callers Need Not Capture Standard Output."
    )]
    id_file: Option<PathBuf>,
}

/// An Identity File Opened Before Any Mutation, Written Once Confirmed.
pub(crate) struct IdFile(File);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Args {
    pub(crate) fn open(&self) -> crate::Result<Option<IdFile>> {
        let Some(path) = &self.id_file else {
            return Ok(None);
        };
        let refused = |error: std::io::Error| {
            let message = format!("cannot create id file {}: {error}", path.display());
            crate::Error::Input(message)
        };
        let file = File::create(path).map_err(refused)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_before_mutation_and_writes_one_identity_per_line() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("ids.txt");
        std::fs::write(&path, b"stale identity\n").unwrap();
        let args = Args { id_file: Some(path.clone()) };

        let file = args.open().unwrap().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"", "an interrupted command must not leave stale identities");
        file.write(["first", "second"]).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "first\nsecond\n");

        assert!(Args::default().open().unwrap().is_none());
        let missing = Args { id_file: Some(directory.path().join("missing").join("ids.txt")) };
        assert!(matches!(missing.open(), Err(crate::Error::Input(_))));
    }
}
