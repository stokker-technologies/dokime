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

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message]
#[harmos_domain::operation(response = (), output = ExecutionFileDownloadFrame, id = "dokime.content.download_execution_file", version = 1,
        policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "DownloadExecutionFile"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadExecutionFile {
    pub execution_id: String,
    pub relative_path: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
