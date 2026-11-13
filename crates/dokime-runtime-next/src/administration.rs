/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Host, Result, host::lock};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub(crate) fn validate_plugin_admission(&self, candidate: &crate::plugin::Entry) -> Result<()> {
        use prost::Message;
        let catalog = candidate.description.catalog.as_ref().ok_or_else(|| Error::Invalid("Admit Plugin: Supply Its Catalog".into()))?;
        let mut candidate_records = Vec::new();
        for section in &catalog.sections {
            if section.r#type == "dokime.v1.RecordContracts" {
                candidate_records.extend(
                    dokime_protocol::v1::RecordContracts::decode(section.payload.as_slice())?
                        .records
                        .into_iter()
                        .filter_map(|record| record.metadata.map(|metadata| metadata.identifier)),
                );
            }
        }
        for existing in lock(&self.plugins)?.values() {
            if existing.description.id == candidate.description.id {
                continue;
            }
            let other = existing.description.catalog.as_ref().unwrap();
            let operation_conflict =
                catalog.operations.iter().any(|operation| other.operations.iter().any(|other| other.id == operation.id));
            let stream_conflict = catalog.streams.iter().any(|stream| other.streams.iter().any(|other| other.id == stream.id));
            let suite_conflict = candidate.suites.suites.iter().any(|suite| {
                existing.suites.suites.iter().any(|other| {
                    suite.metadata.as_ref().map(|metadata| &metadata.identifier)
                        == other.metadata.as_ref().map(|metadata| &metadata.identifier)
                })
            });
            let mut record_conflict = false;
            for section in &other.sections {
                if section.r#type == "dokime.v1.RecordContracts" {
                    record_conflict |= dokime_protocol::v1::RecordContracts::decode(section.payload.as_slice())?
                        .records
                        .iter()
                        .any(|record| record.metadata.as_ref().is_some_and(|metadata| candidate_records.contains(&metadata.identifier)));
                }
            }
            if operation_conflict || stream_conflict || suite_conflict || record_conflict {
                return Err(Error::Invalid(
                    "Admit Plugin: Remove the Conflicting Plugin or Give Its Operations, Suites, Streams and Records Unique Identifiers"
                        .into(),
                ));
            }
        }
        Ok(())
    }
    pub async fn detach_plugin(&self, identifier: &str) -> Result<()> {
        if self.executions.uses_plugin(identifier)? {
            return Err(Error::Invalid("Cancel the Plugin's Active Executions and Wait for Cleanup Before Changing Its Entry".into()));
        }
        let entry = lock(&self.plugins)?.get(identifier).cloned();
        if let Some(entry) = entry {
            let closed = if let Some(mut process) = entry.process.lock().await.take() { process.close().await } else { Ok(()) };
            lock(&self.plugins)?.remove(identifier);
            closed?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
