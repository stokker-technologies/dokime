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
use super::{CapturedExecutionReportInput, RenderedExecutionReport, html::HtmlReport};
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
    ) -> crate::runtime::Result<RenderedExecutionReport> {
        let outputs = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, crate::runtime::Error>(outputs))?;
        self.execution_with_outputs(captured, std::sync::Arc::new(outputs), saved)
    }

    pub(crate) fn execution_with_outputs(
        &self, captured: harmos::storage::ReadBatch<CapturedExecutionReportInput>,
        outputs: std::sync::Arc<harmos::storage::ReadBatch<Vec<ExecutionOutput>>>, saved: &harmos::storage::StorageCursor,
    ) -> crate::runtime::Result<RenderedExecutionReport> {
        let _span = tracing::info_span!("report.render.execution").entered();
        captured.validate()?;
        let source = tracing::info_span!("report.render.source").in_scope(|| {
            harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, crate::runtime::Error>(captured.source(outputs.as_slice())))
        })?;
        let html = tracing::info_span!("report.render.index").in_scope(|| HtmlReport::from_capture(captured, source))?;
        tracing::info_span!("report.render.html").in_scope(|| RenderedExecutionReport::render(html, outputs, saved))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
