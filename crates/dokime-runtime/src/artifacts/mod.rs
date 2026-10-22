/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod directory;
pub mod stream;

/* === Re-Exports === */

pub use dokime_domain::model::artifacts::ArtifactRecord;
pub use dokime_domain::model::artifacts::ArtifactStatus;
pub use dokime_domain::model::artifacts::ArtifactUploadInput;
pub use dokime_domain::model::artifacts::MediaKind;
pub use dokime_domain::model::artifacts::media_kind_for;
pub use dokime_domain::model::artifacts::name;
pub use dokime_domain::model::artifacts::record;
pub use stream::{ArtifactSink, ArtifactWriter, Artifacts};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
