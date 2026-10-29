/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[cfg(feature = "native")]
pub(crate) mod binding;

#[cfg(feature = "native")]
pub mod bounded;

#[cfg(feature = "native")]
pub mod capture;

#[cfg(feature = "native")]
mod context;
pub mod error;

#[cfg(feature = "native")]
pub(crate) mod frame;

#[cfg(feature = "native")]
pub mod input;

#[cfg(feature = "native")]
pub mod instance;

#[cfg(feature = "native")]
pub mod markers;

#[cfg(feature = "native")]
pub mod payload;

#[cfg(feature = "native")]
pub mod plot_inputs;

#[cfg(feature = "native")]
pub mod registry;

#[cfg(feature = "native")]
pub mod resolve;

pub use dokime_domain::model::document::*;
pub use dokime_domain::model::expressions::RecordField;
pub use dokime_domain::model::expressions::RecordMarker;
pub use dokime_domain::model::expressions::detail;
pub use dokime_domain::model::expressions::marker;
pub use dokime_domain::model::expressions::per_case;
pub use error::{DocumentFailure, Result};

#[cfg(feature = "native")]
pub use input::{DocumentDataInput, DocumentResolveInput, DocumentSelection};

#[cfg(feature = "native")]
pub use instance::DocumentInstance;

#[cfg(feature = "native")]
pub use markers::record_markers_for_selection;

#[cfg(feature = "native")]
pub use payload::{
    ArtifactListDescriptor, DocumentCardBodyPayload, DocumentCardPayload, DocumentCardPlacementPayload, DocumentLayoutPayload,
    DocumentNodePayload, DocumentPayload, DocumentSectionNodePayload, DocumentWarning, TelemetrySummaryGroupPayload,
    TelemetrySummaryTilePayload,
};

#[cfg(feature = "native")]
pub use plot_inputs::{plot_descriptor_for_definition, plot_input_for_definition};

#[cfg(feature = "native")]
pub use registry::{DocumentRegistry, DocumentRegistryError};

#[cfg(feature = "native")]
pub use resolve::resolve_document;
