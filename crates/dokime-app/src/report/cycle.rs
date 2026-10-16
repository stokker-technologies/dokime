/* ***
 *
 * Dokime
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod index;

/* === Re-Exports === */
pub(crate) use index::{AnnotationKind, Index, Member, Sample};

/* === Imports === */
// Standard Library Imports
use std::{fmt::Write as _, num::NonZeroUsize};
// Third Party Imports
use harmos::{
    operations::Host,
    storage::{FileMetadata, FileReference, QueryValue},
};
// Local Imports
use super::html::HtmlReport;
use crate::{
    Error, Result,
    runtime::{progress::Operation, storage::cycle::CycleSnapshot},
};
use dokime::{core::Identifier, storage::Manager};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct CycleReport {
    manager: Manager,
    file: super::saved::ReportFile,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl CycleReport {
    pub(crate) async fn prepare(
        manager: &Manager,
        snapshot: CycleSnapshot,
        captured_members: Vec<QueryValue<super::CapturedExecutionReportInput>>,
        operation: Option<&Operation>,
    ) -> Result<Self> {
        let source = HtmlReport::from_cycle(manager, snapshot, captured_members, operation).await?;
        let id_length =
            source.header().cycle.test_cycle_id.as_str().len() + 36 + ".cycle-report.".len();
        let metadata = manager
            .produce(
                allowance(
                    std::mem::size_of::<FileMetadata>()
                        + id_length
                        + source.header().cycle.test_cycle_id.as_str().len()
                        + "text/html".len(),
                )?,
                |_| {
                    let mut name = String::with_capacity(id_length);
                    write!(name, "{}.", source.header().cycle.test_cycle_id).expect("String write");
                    write!(name, "{}", uuid::Uuid::now_v7()).expect("String write");
                    name.push_str(".cycle-report");
                    Ok(FileMetadata {
                        id: Identifier::new(name).map_err(|source| {
                            harmos::Error::backend(
                                "cycle report identity",
                                harmos::BackendFailure::Permanent,
                                source,
                            )
                        })?,
                        scope: Some(
                            Identifier::new(source.header().cycle.test_cycle_id.to_string())
                                .map_err(|source| {
                                    harmos::Error::backend(
                                        "cycle report scope",
                                        harmos::BackendFailure::Permanent,
                                        source,
                                    )
                                })?,
                        ),
                        media_type: "text/html".into(),
                        expected_length: None,
                        expected_sha256: None,
                    })
                },
            )?
            .try_into_value()?;
        let manager = manager.clone();
        tokio::task::spawn_blocking(move || {
            let file = super::saved::ReportFile::render(&manager, &source, metadata)?;
            Ok(Self { manager, file })
        })
        .await?
    }

    pub(crate) async fn finish(
        &mut self,
        operations: &impl Host<Error = harmos::Error>,
    ) -> Result<FileReference> {
        self.file.finish(&self.manager, operations).await
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(crate) fn allowance(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes)
        .filter(|_| bytes != usize::MAX)
        .ok_or_else(|| Error::InvalidReportInputs("cycle report memory overflows".into()))
}

/* === Tests === */
