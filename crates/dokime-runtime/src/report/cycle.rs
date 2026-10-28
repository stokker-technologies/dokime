/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod index;

pub(crate) use index::{AnnotationKind, Index, Member, Sample};

use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};

use harmos::storage::FileReference;

use super::html::HtmlReport;
use crate::storage::Manager;
use crate::{
    runtime::Result,
    runtime::{progress::Operation, storage::cycle::CycleSnapshot},
};
use dokime_domain::model::execution::result::TestExecution;

pub(crate) struct CycleReport {
    manager: Manager,
    file: super::saved::ReportFile,
}

pub(crate) struct CapturedMember {
    spool: std::sync::Mutex<std::fs::File>,

    #[cfg(test)]
    pub(in crate::report) decodes: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

// One Cycle Member In Cycle Order: Captured Evidence, Or A Planned Row Nobody Started.
pub(crate) enum CycleMember {
    Captured(std::sync::Arc<CapturedMember>),
    Planned(Box<TestExecution>),
}

impl CycleMember {
    pub(crate) fn capture(input: harmos::storage::ReadBatch<super::CapturedExecutionReportInput>) -> Result<Self> {
        Ok(Self::Captured(std::sync::Arc::new(CapturedMember::spool(&input)?)))
    }
}

impl CapturedMember {
    // Spools One Materialized Member So Every Cycle Report Reuses It Instead Of Materializing It Again.
    pub(crate) fn spool(input: &super::CapturedExecutionReportInput) -> Result<Self> {
        let spool = tempfile::tempfile()?;
        let mut output = BufWriter::with_capacity(1024 * 1024, spool);
        serde_json::to_writer(&mut output, input).map_err(crate::storage::Error::from)?;
        output.flush()?;
        let spool = output.into_inner().map_err(|error| error.into_error())?;
        Ok(Self {
            spool: std::sync::Mutex::new(spool),
            #[cfg(test)]
            decodes: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        })
    }

    pub(crate) fn read(&self) -> Result<harmos::storage::ReadBatch<super::CapturedExecutionReportInput>> {
        let mut spool =
            self.spool.lock().map_err(|_| crate::runtime::Error::InvalidReportInputs("member spool lock is poisoned".into()))?;
        spool.seek(SeekFrom::Start(0))?;
        let input = serde_json::from_reader(BufReader::with_capacity(1024 * 1024, &mut *spool)).map_err(crate::storage::Error::from)?;
        #[cfg(test)]
        self.decodes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(harmos::storage::ReadBatch::new(input))
    }
}

impl CycleReport {
    pub(crate) async fn prepare(
        manager: &Manager, snapshot: CycleSnapshot, members: Vec<CycleMember>, operation: &Operation,
        report_defaults: &crate::config::ReportDefaults,
    ) -> Result<Self> {
        let operation = operation.clone();
        let report_defaults = *report_defaults;
        let nonce = uuid::Uuid::now_v7();
        let manager = manager.clone();
        tokio::task::spawn_blocking(move || {
            let source = tracing::info_span!("report.render.cycle.members", members = members.len())
                .in_scope(|| HtmlReport::from_cycle(snapshot, members, &operation, &report_defaults))?;
            let owner = source.header().cycle.test_cycle_id.as_str();
            let file = tracing::info_span!("report.render.cycle.html")
                .in_scope(|| super::saved::ReportFile::render(&source, owner, nonce, ".cycle-report"))?;
            Ok(Self { manager, file })
        })
        .await?
    }

    pub(crate) async fn finish(mut self) -> Result<FileReference> {
        self.file.finish(&self.manager).await
    }
}
