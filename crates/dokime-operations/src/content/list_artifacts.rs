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

// Metadata-only inventory for operator views. Entries are ordered by artifact
// ID. A continuation is valid only while the runtime epoch and complete
// metadata fingerprint remain unchanged.
#[harmos::operation(resources = [crate::hosting::Host<ListArtifacts>], response = ListArtifactsReply, id = "dokime.content.list_artifacts",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "ListArtifacts"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListArtifacts {
    pub limit: u32,
    pub after_artifact_id: String,
    pub cursor: Option<ArtifactCursor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListArtifacts);
}
