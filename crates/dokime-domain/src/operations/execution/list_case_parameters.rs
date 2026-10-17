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
use serde::{Deserialize, Serialize};
// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[harmos::operation(response = ListCaseParametersReply, id = "dokime.execution.list_case_parameters", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "ListCaseParameters"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListCaseParameters {
    pub cycle_id: String,
    pub case_id: String,
    pub limit: u32,
    pub after_name: String,
    pub cursor: Option<ParameterCursor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
