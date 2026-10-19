/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[cfg(feature = "host")]
pub(crate) mod binding;
#[cfg(feature = "host")]
pub mod bounded;
#[cfg(feature = "host")]
pub mod capture;
#[cfg(feature = "host")]
mod context;
pub mod error;
#[cfg(feature = "host")]
pub(crate) mod frame;
#[cfg(feature = "host")]
pub mod input;
#[cfg(feature = "host")]
pub mod instance;
#[cfg(feature = "host")]
pub mod markers;
#[cfg(feature = "host")]
pub mod payload;
#[cfg(feature = "host")]
pub mod plot_inputs;
#[cfg(feature = "host")]
pub mod registry;
#[cfg(feature = "host")]
pub mod resolve;

/* === Re-Exports === */
pub use dokime_domain::model::document::*;
pub use dokime_domain::model::expressions::RecordField;
pub use dokime_domain::model::expressions::RecordMarker;
pub use dokime_domain::model::expressions::detail;
pub use dokime_domain::model::expressions::marker;
pub use dokime_domain::model::expressions::per_case;
pub use error::{DocumentFailure, Result};
#[cfg(feature = "host")]
pub use input::{DocumentDataInput, DocumentResolveInput, DocumentSelection};
#[cfg(feature = "host")]
pub use instance::DocumentInstance;
#[cfg(feature = "host")]
pub use markers::record_markers_for_selection;
#[cfg(feature = "host")]
pub use payload::{
    ArtifactListDescriptor, DocumentCardBodyPayload, DocumentCardPayload, DocumentCardPlacementPayload, DocumentLayoutPayload,
    DocumentNodePayload, DocumentPayload, DocumentSectionNodePayload, DocumentWarning, TelemetrySummaryGroupPayload,
    TelemetrySummaryTilePayload,
};
#[cfg(feature = "host")]
pub use plot_inputs::{plot_descriptor_for_definition, plot_input_for_definition};
#[cfg(feature = "host")]
pub use registry::{DocumentRegistry, DocumentRegistryError};
#[cfg(feature = "host")]
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
