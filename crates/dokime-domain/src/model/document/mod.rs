/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod addition;
pub mod column;
pub mod definition;
pub mod format;
pub mod host;
pub mod validation;

/* === Re-Exports === */
pub use addition::DocumentAddition;
pub use column::Column;
pub use definition::{
    AxisScale, Band, Card, DEFAULT_PLOT_MAX_POINTS, DURATION_UNIT_SYMBOLS, Document, DocumentCard,
    DocumentCardBody, DocumentCardBodyKind, DocumentCardDefinition, DocumentCardDensity,
    DocumentCardLayout, DocumentCardPlacement, DocumentCardRail, DocumentDataset,
    DocumentDatasetGrain, DocumentDatasetWarning, DocumentDefinition, DocumentFilter,
    DocumentLayout, DocumentNode, DocumentSection, DocumentSectionNode, DocumentSectionRepeat,
    DocumentTagged, DownsampleTechnique, FftDetrend, FftOptions, FftWindow, FilterOperator,
    FrequencyLevelUnit, HistogramOptions, Line, Plot, PlotAxisKind, PlotDefinition, PlotDescriptor,
    PlotInput, PlotStyle, PlotType, ReferenceAxis, ReferenceBand, ReferenceLine, ResponsiveSpan,
    Section, SigmaIntervalOptions, TelemetrySummaryCondition, TelemetrySummaryDefinition,
    TelemetrySummaryFormat, TelemetrySummaryGroupDefinition, TelemetrySummaryPresentation,
    TelemetrySummaryStat, TelemetrySummaryState, TelemetrySummaryTileDefinition,
    resolve_document_plot_axes, validate_plot_definition,
};
pub use validation::validate_document_definition;

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
