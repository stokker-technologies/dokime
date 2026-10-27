/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Report Renderer

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{CapturedExecutionReportInput, PreparedExecutionReport, html::HtmlReport};
use dokime_domain::model::execution::saved::ExecutionOutput;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, Default)]
pub struct Renderer;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Renderer {
    pub fn execution(
        &self, captured: harmos::storage::ReadBatch<CapturedExecutionReportInput>, outputs: Vec<ExecutionOutput>,
        saved: &harmos::storage::StorageCursor,
    ) -> crate::runtime::Result<PreparedExecutionReport> {
        let outputs = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, crate::runtime::Error>(outputs))?;
        self.execution_with_outputs(captured, std::sync::Arc::new(outputs), saved)
    }

    pub(crate) fn execution_with_outputs(
        &self, captured: harmos::storage::ReadBatch<CapturedExecutionReportInput>,
        outputs: std::sync::Arc<harmos::storage::ReadBatch<Vec<ExecutionOutput>>>, saved: &harmos::storage::StorageCursor,
    ) -> crate::runtime::Result<PreparedExecutionReport> {
        let source =
            harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, crate::runtime::Error>(captured.source(outputs.as_slice())))?;
        let html = HtmlReport::from_capture(captured, source)?;
        PreparedExecutionReport::prepare_html(html, outputs, saved)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
