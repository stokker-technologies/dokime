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
use std::io::Cursor;

// Third Party Imports

// Local Imports
use super::CallOptions;
use super::resources::Files;
use crate::{
    FileReference, Result,
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
    pub(crate) async fn upload_bytes(&self, data: Vec<u8>, context: CallOptions, request_id: &str) -> Result<FileReference> {
        let length = data.len() as u64;
        let endpoint = Uploads::Client { files: self, context };
        transfers::upload(endpoint, Cursor::new(data), length, request_id).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
