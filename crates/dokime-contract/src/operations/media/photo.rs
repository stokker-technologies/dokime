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

// Local Imports
use super::{ArtifactMetadata, PhotoRequest, TRANSFER};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::operation(request = PhotoRequest, response = ArtifactMetadata, id = "dokime.media.photo", version = 1,
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Media", method = "Photo"))]
pub struct Photo;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
