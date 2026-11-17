/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Create

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashSet;

// Third Party Imports
use harmos_protocol::host::PluginState;
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::CycleStatus;
use dokime_protocol::host::ExecutionMode;
use dokime_protocol::host::ExecutionsCreateRequest;
use dokime_protocol::host::ExecutionsCreateResponse;
use dokime_protocol::model::origin::Kind;
use dokime_storage::Execution;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ExecutionsCreateRequest) -> Result<ExecutionsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let _guard = context.administration.lock().await;
        let cycle = crate::host::execution_state::cycle(context, &request.parent).await?;
        if cycle.sealed || request.executions.is_empty() || request.executions.len() > context.settings.max_cases {
            return Err(Error::Invalid("Append Executions: Supply Bounded Bindings to an Unsealed Cycle".into()));
        }
        let mut identifiers = HashSet::new();
        let plugins = context.host.plugins().await?;
        let mut prepared = Vec::new();
        for binding in &request.executions {
            if uuid::Uuid::parse_str(&binding.execution_id).is_err() || !identifiers.insert(&binding.execution_id) {
                return Err(Error::Invalid("Append Executions: Supply Unique Execution UUIDs".into()));
            }
            let case = crate::host::cases::storage::read(context, &binding.case_id, Some(&request.parent)).await?;
            let label = case.metadata.as_ref().map(|metadata| metadata.label.clone()).unwrap_or_default();
            let origin = case.origin.ok_or_else(|| Error::Invalid("Append Executions: Materialize the Selected Case".into()))?;
            let run = match origin.kind.as_ref() {
                Some(Kind::PluginRun(run)) => run,
                _ => return Err(Error::Invalid("Append Executions: Use PluginRun Cases; Import Evidence Separately".into())),
            };
            let plugin = plugins
                .iter()
                .find(|plugin| plugin.state == PluginState::Ready as i32 && plugin.name == run.plugin)
                .ok_or_else(|| Error::Precondition("Append Executions: Reconnect the Case's Plugin Before Admission".into()))?;
            let description = plugin
                .description
                .as_ref()
                .ok_or_else(|| Error::Precondition("Append Executions: Discover the Plugin Before Admission".into()))?;
            let catalog = description
                .declarations
                .as_ref()
                .ok_or_else(|| Error::Precondition("Append Executions: Discover the Plugin Catalog Before Admission".into()))?;
            let catalog_bytes = catalog.encode_to_vec();
            let origin = if catalog.digest != run.digest || description.started_at != run.started_at {
                let test_case =
                    run.test_case.as_ref().ok_or_else(|| Error::Invalid("Append Executions: Supply the Frozen TestCase".into()))?;
                context.host.materialize(&run.plugin, &run.template, test_case.parameters.clone()).await?
            } else {
                origin
            };
            prepared.push((
                Execution {
                    identifier: binding.execution_id.clone(),
                    cycle: request.parent.clone(),
                    origin: Some(origin),
                    catalog: catalog_bytes,
                    ..Default::default()
                },
                binding.case_id.clone(),
                label,
            ));
        }
        context.host.store().bind_executions(&request.parent, &prepared)?;
        context.host.signal_change();
        if cycle.continue_automatically && cycle.status != CycleStatus::Paused as i32 {
            for binding in &request.executions {
                context.host.start_existing(&binding.execution_id, ExecutionMode::Automatic).await?;
            }
        }
        Ok(ExecutionsCreateResponse {
            executions: request
                .executions
                .iter()
                .map(|binding| crate::host::execution_state::detail(context, &binding.execution_id))
                .collect::<Result<Vec<_>>>()?,
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ExecutionsCreateRequest, ExecutionsCreateResponse, context, "dokime.host.v1", "Executions", "Create");
