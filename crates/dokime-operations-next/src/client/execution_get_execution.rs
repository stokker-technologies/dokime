/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.get_execution";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionGetExecutionRequest, v1::ExecutionDetail>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionGetExecutionRequest) -> Result<v1::ExecutionDetail> {
    super::complete(ID, context.settings.read_timeout, async {
        let snapshot = context.host.store().read(&request.execution_id)?;
        let summary = super::projection::summary(&snapshot);
        let schemas = super::projection::schemas(&snapshot)?;
        Ok(v1::ExecutionDetail {
            summary: Some(summary),
            cycle_id: snapshot.execution.cycle.clone(),
            scope: Some(super::projection::scope(&snapshot)),
            origin: snapshot.execution.origin.clone(),
            outcome: snapshot.execution.outcome.clone(),
            seal: snapshot.seal.clone(),
            schemas: Some(schemas),
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
