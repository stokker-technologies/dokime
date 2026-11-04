/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Repeated Sections Expand Over Every Distinct Value In The Execution's
// Evidence, Not Over The Bounded Rows A Report Previews.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{execution::ExecutionView, records::RecordReader, telemetry::TelemetryReader};
use crate::document::DocumentSelection;
use crate::document::input::{Artifacts, BorrowedData, Context, Records};
use crate::document::resolve::{RepeatScan, RepeatValues};
use crate::runtime::Result;
use dokime_domain::model::execution::saved::ExecutionOutput;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Folds the scan over the execution's context, its complete output inventory
/// and, when a repeat reads them, every record and telemetry page in order.
pub(crate) async fn resolve(
    view: &ExecutionView, local: &crate::storage::sqlite::State, settings: &harmos::config::SchemaSettings, selection: &DocumentSelection,
    context: Context<'_>, outputs: &[ExecutionOutput], mut scan: RepeatScan,
) -> Result<RepeatValues> {
    if scan.is_empty() {
        return Ok(scan.finish());
    }
    let empty = BorrowedData {
        context,
        records: Records::default(),
        telemetry: &[],
        artifacts: Artifacts::default(),
    };
    scan.fold(
        selection,
        BorrowedData {
            artifacts: Artifacts::Outputs(outputs),
            ..empty
        },
    );
    if scan.needs_records() {
        let mut reader = RecordReader::open(view).await?;
        while let Some(page) = reader.next().await? {
            let rows: Vec<_> = page.iter().map(|row| row.record.clone()).collect();
            scan.fold(
                selection,
                BorrowedData {
                    records: Records::Native((&rows).into()),
                    ..empty
                },
            );
        }
    }
    if scan.needs_telemetry(context) {
        // One Agreed Stream Is Read Alone; Anything Else Reads Every Stream.
        let stream = scan.telemetry_stream(context).and_then(|declared| {
            let captured = view.captured_source()?;
            let mut matches = captured.telemetry_contracts.keys().filter(|id| crate::plot::stream_matches_alias(id.as_str(), declared));
            let stream = matches.next()?;
            matches.next().is_none().then(|| stream.clone())
        });
        let execution = &view.execution().test_execution_id;
        let mut reader = TelemetryReader::selected(view, local, stream.as_ref(), None, 0, settings).await?;
        while let Some(page) = reader.next().await? {
            let samples: Vec<_> = page
                .iter()
                .map(|row| {
                    let mut sample = row.sample.clone();
                    if let Some(scope) = sample.captured_scopes.iter().find(|scope| scope.test_execution_id.as_ref() == Some(execution)) {
                        sample.scope = Some(scope.clone());
                    }
                    sample
                })
                .collect();
            scan.fold(selection, BorrowedData { telemetry: &samples, ..empty });
        }
    }
    Ok(scan.finish())
}

/* === Private Functions === */

/* === Tests === */
