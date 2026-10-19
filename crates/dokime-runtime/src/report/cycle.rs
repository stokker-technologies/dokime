/* ***
 *
 * Dokime
 *
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

// Third Party Imports
use harmos::{
    operations::Host,
    storage::{FileMetadata, FileReference},
};

// Local Imports
use super::html::HtmlReport;
use crate::storage::Manager;
use crate::{
    runtime::Result,
    runtime::{progress::Operation, storage::cycle::CycleSnapshot},
};
use dokime_domain::model::identity::Identifier;

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
        manager: &Manager, snapshot: CycleSnapshot, captured_members: Vec<super::CapturedExecutionReportInput>, operation: &Operation,
    ) -> Result<Self> {
        let source = HtmlReport::from_cycle(snapshot, captured_members, operation)?;
        let cycle = source.header().cycle.test_cycle_id.to_string();
        let metadata = FileMetadata {
            id: Identifier::new(format!("{cycle}.{}.cycle-report", uuid::Uuid::now_v7())).map_err(crate::storage::Error::from)?,
            scope: Some(Identifier::new(cycle).map_err(crate::storage::Error::from)?),
            media_type: "text/html".into(),
            expected_length: None,
            expected_sha256: None,
        };
        let manager = manager.clone();
        tokio::task::spawn_blocking(move || {
            let file = super::saved::ReportFile::render(&source, metadata)?;
            Ok(Self { manager, file })
        })
        .await?
    }

    pub(crate) async fn finish(mut self, operations: &impl Host<Error = harmos::Error>) -> Result<FileReference> {
        self.file.finish(&self.manager, operations).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
