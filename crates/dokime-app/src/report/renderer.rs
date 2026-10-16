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
use dokime::{execution::saved::ExecutionOutput, storage::Manager};
use harmos::storage::{QueryValue, SharedQueryValue};
use std::num::NonZeroUsize;

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
        manager: Manager,
        captured: SharedQueryValue<CapturedExecutionReportInput>,
        outputs: QueryValue<Vec<ExecutionOutput>>,
    ) -> crate::Result<PreparedExecutionReport> {
        captured.check_origin(&manager)?;
        outputs.check_origin(&manager)?;
        let bytes = captured.source_memory(&outputs);
        let allowance = NonZeroUsize::new(bytes)
            .filter(|_| bytes != usize::MAX)
            .ok_or_else(|| {
                crate::Error::InvalidReportInputs("report source memory overflows".into())
            })?;
        let source = captured
            .try_project(allowance, |captured, _| Ok(captured.source(&outputs)))?
            .try_into_shared()?;
        let html = HtmlReport::from_capture(&manager, &captured, &source)?;
        PreparedExecutionReport::prepare_html(html, outputs)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
