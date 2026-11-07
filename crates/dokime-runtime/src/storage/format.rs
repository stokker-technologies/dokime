/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Format Identity And Refusal

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::{Error, Result};

/* === Constants === */

pub const IDENTITY: &str = env!("DOKIME_STORAGE_FORMAT");
pub const BUILD: &str = env!("DOKIME_STORAGE_BUILD");
pub const RECORD: &str = "storage-format.json";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    format: String,
    writer_build: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// A Read-Only Preflight: Even Creating The Lease Must Follow This Refusal.
pub fn check(root: &Path) -> Result<bool> {
    let bytes = match std::fs::read(root.join(RECORD)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let entries = match std::fs::read_dir(root) {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
                Err(error) => return Err(error.into()),
            };
            for entry in entries {
                if entry?.file_name() != "local.lock" {
                    return Err(mismatch(root, None));
                }
            }
            return Ok(false);
        }
        Err(error) => return Err(error.into()),
    };
    let record: Record = serde_json::from_slice(&bytes).map_err(|_| mismatch(root, None))?;
    if record.format != IDENTITY {
        return Err(mismatch(root, Some(record)));
    }
    Ok(true)
}

// Called Only While Holding The Root's Existing Exclusive Lease.
pub(super) fn record(root: &Path) -> Result<()> {
    if check(root)? {
        return Ok(());
    }
    let record = Record {
        format: IDENTITY.to_owned(),
        writer_build: BUILD.to_owned(),
    };
    let bytes = serde_json::to_vec(&record)?;
    harmos::filesystem::publication::write(&root.join(RECORD), &bytes)?;
    Ok(())
}

/* === Private Functions === */

fn mismatch(root: &Path, record: Option<Record>) -> Error {
    let (writer_build, writer_format) = match record {
        Some(record) => (record.writer_build.into_boxed_str(), Some(record.format.into_boxed_str())),
        None => ("unknown (missing or invalid format record)".into(), None),
    };
    Error::StorageFormatMismatch {
        root: root.to_owned(),
        writer_build,
        running_build: BUILD,
        writer_format,
        running_format: IDENTITY,
    }
}

/* === Tests === */
