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
#[harmos::operation(response = ListHealthResponse, id = "dokime.health.list_health", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Health", method = "ListHealth"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListHealth {
    pub selection: Option<AnnotationSelection>,
    // default 50, maximum 100
    #[row(description = "default 50, maximum 100")]
    pub limit: u32,
    pub after_id: String,
    // Exact runtime revision and selection, required with after_id. Restart on change.
    #[row(
        description = "Exact runtime revision and selection, required with after_id. Restart on change."
    )]
    pub cursor: Option<Cursor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
