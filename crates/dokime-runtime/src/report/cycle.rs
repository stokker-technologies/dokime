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
use harmos::storage::FileReference;

// Local Imports
use super::html::HtmlReport;
use crate::storage::Manager;
use crate::{
    runtime::Result,
    runtime::{progress::Operation, storage::cycle::CycleSnapshot},
};
use dokime_domain::model::execution::result::TestExecution;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct CycleReport {
    manager: Manager,
    file: super::saved::ReportFile,
}

/* === Enums === */

// One Cycle Member In Cycle Order: Captured Evidence, Or A Planned Row Nobody Started.
pub(crate) enum CycleMember {
    Captured(harmos::storage::ReadBatch<super::CapturedExecutionReportInput>),
    Planned(Box<TestExecution>),
}

/* === Traits === */

/* === Implementations === */

impl CycleReport {
    pub(crate) async fn prepare(
        manager: &Manager, snapshot: CycleSnapshot, members: Vec<CycleMember>, operation: &Operation,
        report_defaults: &crate::config::ReportDefaults,
    ) -> Result<Self> {
        let source = HtmlReport::from_cycle(snapshot, members, operation, report_defaults)?;
        let nonce = uuid::Uuid::now_v7();
        let manager = manager.clone();
        tokio::task::spawn_blocking(move || {
            let owner = source.header().cycle.test_cycle_id.as_str();
            let file = super::saved::ReportFile::render(&source, owner, nonce, ".cycle-report")?;
            Ok(Self { manager, file })
        })
        .await?
    }

    pub(crate) async fn finish(mut self) -> Result<FileReference> {
        self.file.finish(&self.manager).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
