/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod binding;
pub mod bounded;
pub mod capture;
mod context;
pub(crate) mod fold;
pub(crate) mod frame;
pub mod input;
pub mod instance;
pub mod markers;
pub mod payload;
pub mod plot_inputs;
pub mod registry;
pub mod resolve;

/* === Re-Exports === */
pub use dokime::document::*;
pub use input::{DocumentDataInput, DocumentResolveInput, DocumentSelection};
pub use instance::DocumentInstance;
pub use markers::record_markers_for_selection;
pub use payload::{
    ArtifactListDescriptor, DocumentCardBodyPayload, DocumentCardPayload,
    DocumentCardPlacementPayload, DocumentLayoutPayload, DocumentNodePayload, DocumentPayload,
    DocumentSectionNodePayload, DocumentWarning, TelemetrySummaryGroupPayload,
    TelemetrySummaryTilePayload,
};
pub use plot_inputs::{plot_descriptor_for_definition, plot_input_for_definition};
pub use registry::{DocumentRegistry, DocumentRegistryError};
pub use resolve::resolve_document;

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
