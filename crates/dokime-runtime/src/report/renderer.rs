/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Report Renderer

use super::{CapturedExecutionReportInput, PreparedExecutionReport, html::HtmlReport};
use dokime_domain::model::execution::saved::ExecutionOutput;

#[derive(Clone, Copy, Debug, Default)]
pub struct Renderer;

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
        let _span = tracing::info_span!("report.render.execution").entered();
        let source = tracing::info_span!("report.render.source").in_scope(|| {
            harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, crate::runtime::Error>(captured.source(outputs.as_slice())))
        })?;
        let html = tracing::info_span!("report.render.index").in_scope(|| HtmlReport::from_capture(captured, source))?;
        tracing::info_span!("report.render.html").in_scope(|| PreparedExecutionReport::prepare_html(html, outputs, saved))
    }
}
