/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Captured Record Schema

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use prost::Message;


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */


pub const ID: &str = "dokime.evidence.get_captured_record_schema";
pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::EvidenceGetCapturedRecordSchemaRequest, v1::RecordSchema>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::EvidenceGetCapturedRecordSchemaRequest) -> Result<v1::RecordSchema> {
    super::complete(ID, context.settings.read_timeout, async {
        if !request.epoch.is_empty() && request.epoch != context.host.identity().epoch {
            return Err(Error::Invalid("Get Captured Schema: Refresh the Backend Epoch".into()));
        }
        if request.schema_version != 0 && request.schema_version != 1 {
            return Err(Error::Invalid("Get Captured Schema: Select Captured Schema Version 1".into()));
        }
        let mut found = None;
        for id in context.host.store().identifiers()? {
            let cut = context.host.store().metadata(&id)?;
            let catalog = harmos_protocol::v1::Catalog::decode(cut.execution.catalog.as_slice())?;
            for section in &catalog.sections {
                if section.r#type != "dokime.v1.RecordContracts" { continue; }
                for record in v1::RecordContracts::decode(section.payload.as_slice())?.records {
                    if record.metadata.as_ref().is_none_or(|metadata| metadata.identifier != request.record_type) { continue; }
                    let candidate = v1::RecordSchema { record_type: request.record_type.clone(), schema_version: 1, metadata: record.metadata, ..Default::default() };
                    if found.as_ref().is_some_and(|existing| existing != &candidate) {
                        return Err(Error::Invalid("Get Captured Schema: Inspect the Execution's Frozen Catalog to Resolve Conflicting Captured Types".into()));
                    }
                    found = Some(candidate);
                }
            }
        }
        found.ok_or_else(|| Error::Invalid(format!("Get Captured Schema: Select a Captured Record Type {}", request.record_type)))
    }).await
}


/* === Private Functions === */

/* === Tests === */
