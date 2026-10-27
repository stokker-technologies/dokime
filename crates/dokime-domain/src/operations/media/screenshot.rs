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

#[harmos_domain::operation(response = ArtifactMetadata, id = "dokime.media.screenshot",
        policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
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
