/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File Upload Convenience

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{fs::File, io::Cursor, path::Path};

// Third Party Imports
use uuid::Uuid;

// Local Imports
use super::CallOptions;
use super::resources::Files;
use crate::{
    Error, FileReference, Result,
    transfers::{self, Uploads},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Files {
    pub async fn upload(&self, path: impl AsRef<Path>) -> Result<FileReference> {
        let file =
            File::open(path.as_ref()).map_err(|error| Error::Invalid(format!("Upload File: Open {}: {error}", path.as_ref().display())))?;
        let length = file.metadata().map_err(|error| Error::Invalid(format!("Upload File: Inspect the Source Length: {error}")))?.len();
        let endpoint = Uploads::Client {
            files: self,
            context: CallOptions::default(),
        };
        transfers::upload(endpoint, file, length, &Uuid::now_v7().to_string()).await
    }

    pub(crate) async fn upload_bytes(&self, data: Vec<u8>, context: CallOptions, request_id: &str) -> Result<FileReference> {
        let length = data.len() as u64;
        let endpoint = Uploads::Client { files: self, context };
        transfers::upload(endpoint, Cursor::new(data), length, request_id).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
