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
#[harmos_domain::operation(response = ExecutionFileWindow, id = "dokime.content.read_execution_file", version = 1,
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "ReadExecutionFile"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadExecutionFile {
    pub execution_id: String,
    pub relative_path: String,
    pub offset: u64,
    pub max_bytes: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
