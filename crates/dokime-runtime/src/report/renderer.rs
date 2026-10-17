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
use std::sync::Arc;

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
        &self,
        captured: Arc<CapturedExecutionReportInput>,
        outputs: Vec<ExecutionOutput>,
        saved: &harmos::storage::StorageCursor,
    ) -> crate::runtime::Result<PreparedExecutionReport> {
        let source = Arc::new(captured.source(&outputs));
        let html = HtmlReport::from_capture(captured, source)?;
        PreparedExecutionReport::prepare_html(html, outputs, saved)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
