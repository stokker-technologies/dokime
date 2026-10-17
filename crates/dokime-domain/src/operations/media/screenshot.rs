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
#[harmos::operation(response = ArtifactMetadata, id = "dokime.media.screenshot", version = 1,
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Media", method = "Screenshot"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Screenshot {
    pub scope: Option<ArtifactScope>,
    // Zero-based display index; absent selects the primary display.
    #[row(description = "Zero-based display index; absent selects the primary display.")]
    pub display: Option<u32>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
