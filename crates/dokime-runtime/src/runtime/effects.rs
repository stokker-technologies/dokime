/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::model::time::now_utc;
use crate::transactions::effects::{EffectChange, RecordEffect};
use crate::{
    RuntimeHandle,
    execution::ExecutionScope,
    operations::{
        effects::*,
        hosting::{Serve, Served},
    },
    runtime::sidecars::core::RecordIngress,
};
use dokime_domain::model::{
    execution::{RecordScope, effects::PendingEffect},
    time::Timestamp,
};
use harmos::{
    Context, Message,
    operations::{CallContext, HostContext},
};
use tonic::Status;

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<ReadEnvironment> for RecordIngress {
    fn serve<'a>(&'a self, _: ReadEnvironment, ctx: Context<'a, ReadEnvironment>) -> Served<'a, Environment> {
        Box::pin(async move {
            let runtime = self.runtime()?;
            let owner = ctx.resource::<HostContext>().borrow().caller_scope().map(|scope| scope.owner().to_owned());
            let active = execution_scope(&runtime, ctx.call(), owner.as_deref())?;
            Ok(harmos::Response::new(Environment {
                backend_pid: std::process::id(),
                in_step: active.is_some_and(|(_, _, step)| step),
            }))
        })
    }
}

impl Serve<PrepareEffect> for RecordIngress {
    fn serve<'a>(&'a self, request: PrepareEffect, ctx: Context<'a, PrepareEffect>) -> Served<'a, Ticket> {
        Box::pin(async move {
            let runtime = self.runtime()?;
            let owner = ctx.resource::<HostContext>().borrow().caller_scope().map(|scope| scope.owner().to_owned());
            let _admission = runtime.effect_admission.lock().await;
            let Some((scope, source, _)) = execution_scope(&runtime, ctx.call(), owner.as_deref())? else {
                return Ok(harmos::Response::new(Ticket { id: String::new() }));
            };
            if request.resource.is_empty() || request.inverse_operation.is_empty() {
                return Err(Status::invalid_argument("effect needs a resource and encoded inverse"));
            }
            if request.inverse_operation.split_once('.').is_none_or(|(inverse_owner, _)| inverse_owner != source) {
                return Err(Status::permission_denied("an effect inverse must belong to its calling provider"));
            }
            let ticket = uuid::Uuid::now_v7().to_string();
            let at = now_utc();
            let effect = PendingEffect {
                ticket: ticket.clone(),
                scope,
                source,
                resource: request.resource,
                inverse_operation: request.inverse_operation,
                inverse: request.inverse,
                description: request.description,
                prepared_at: at.clone(),
                applied_at: None,
                coalesced: Vec::new(),
            };
            runtime
                .commit(RecordEffect {
                    ticket: ticket.clone(),
                    change: EffectChange::Prepared(Box::new(effect)),
                    at,
                })
                .await
                .map_err(|error| Status::failed_precondition(error.to_string()))?;
            Ok(harmos::Response::new(Ticket { id: ticket }))
        })
    }
}

impl Serve<ConfirmEffect> for RecordIngress {
    fn serve<'a>(&'a self, request: ConfirmEffect, ctx: Context<'a, ConfirmEffect>) -> Served<'a, ()> {
        Box::pin(async move {
            if request.ticket.is_empty() {
                return Ok(().into());
            }
            let runtime = self.runtime()?;
            let scope = ExecutionScope::current(ctx.call())
                .ok_or_else(|| Status::permission_denied("effect confirmation needs its execution scope"))?;
            let owner = ctx.resource::<HostContext>().borrow().caller_scope().map(|scope| scope.owner().to_owned());
            {
                let state = runtime.state_read();
                let effect = state
                    .pending_effects
                    .iter()
                    .find(|effect| {
                        effect.ticket == request.ticket || effect.coalesced.iter().any(|prepare| prepare.ticket == request.ticket)
                    })
                    .ok_or_else(|| Status::not_found("effect ticket is no longer pending"))?;
                let source = effect
                    .coalesced
                    .iter()
                    .find(|prepare| prepare.ticket == request.ticket)
                    .map_or(&effect.source, |prepare| &prepare.source);
                if effect.scope.test_execution_id.as_ref().map(|id| id.as_str()) != Some(scope.execution.as_str())
                    || owner.as_ref().is_some_and(|owner| owner != source)
                {
                    return Err(Status::permission_denied("effect belongs to another execution"));
                }
            }
            let at = Timestamp::new(request.at);
            at.to_datetime().map_err(|error| Status::invalid_argument(error.to_string()))?;
            runtime
                .commit(RecordEffect {
                    ticket: request.ticket,
                    change: if request.restored { EffectChange::RolledBack } else { EffectChange::Applied },
                    at,
                })
                .await
                .map_err(|error| Status::failed_precondition(error.to_string()))?;
            Ok(().into())
        })
    }
}

impl RuntimeHandle {
    // Native Embedding Owners Use The Same Journal As The Plugin Operation Bridge.
    pub async fn begin_external_change(&self, effect: PendingEffect) -> crate::runtime::Result<()> {
        let _admission = self.effect_admission.lock().await;
        if let Some(session) = effect.scope.test_execution_id.as_ref().and_then(|id| self.execution_sessions.get(id.as_str()))
            && session.effects_closed()
        {
            return Err(crate::runtime::Error::Operation("execution is no longer accepting external changes".into()));
        }
        let at = effect.prepared_at.clone();
        self.commit(RecordEffect {
            ticket: effect.ticket.clone(),
            change: EffectChange::Prepared(Box::new(effect)),
            at,
        })
        .await
        .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
        Ok(())
    }

    pub async fn complete_external_change(&self, ticket: String, at: Timestamp) -> crate::runtime::Result<()> {
        self.commit(RecordEffect {
            ticket,
            change: EffectChange::Applied,
            at,
        })
        .await
        .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
        Ok(())
    }

    pub(crate) async fn capture_run_start(&self, scope: RecordScope) -> crate::runtime::Result<()> {
        let captures = self
            .state_read()
            .installed_plugins
            .values()
            .filter(|plugin| plugin.enabled && self.plugin_available(&plugin.source))
            .filter_map(|plugin| plugin.run_start.as_ref().map(|capture| (plugin.source.clone(), capture.clone())))
            .collect::<Vec<_>>();
        for (source, capture) in captures {
            let bytes = self
                .operations
                .call_encoded(&format!("{source}.{}", capture.operation), capture.request)
                .await
                .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
            let capture = StartEvidence::decode(&bytes).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
            let json = serde_json::from_str::<serde_json::Value>(&capture.payload_json)
                .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
            let mut request = dokime_domain::model::records::publication::RecordPublishRequest::new(
                crate::records::BuiltinRecordType::ExecutionStartEvidence.identifier(),
                crate::records::RecordSeverity::Info,
            );
            request.scope = Some(scope.clone());
            request.message = Some(format!("Run Start Evidence From {source}"));
            request.details.insert("publisher".into(), dokime_domain::model::values::Value::from(source));
            request.details.insert("snapshot".into(), json_value(json));
            self.commit(crate::transactions::PublishRecord::host(request, now_utc()))
                .await
                .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
        }
        Ok(())
    }

    pub(crate) async fn restore_effects(&self, execution: Option<&str>) -> crate::runtime::Result<()> {
        let admission = self.effect_admission.lock().await;
        if let Some(session) = execution.and_then(|execution| self.execution_sessions.get(execution)) {
            session.close_effects();
        }
        let pending = self
            .state_read()
            .pending_effects
            .iter()
            .rev()
            .filter(|effect| {
                execution.is_none_or(|execution| effect.scope.test_execution_id.as_ref().is_some_and(|id| id.as_str() == execution))
            })
            .cloned()
            .collect::<Vec<_>>();
        drop(admission);
        for effect in pending {
            let bytes = self.operations.call_encoded(&effect.inverse_operation, effect.inverse).await.map_err(|error| {
                crate::runtime::Error::Operation(format!(
                    "pending inverse {} could not restore {}: {error}",
                    effect.ticket, effect.resource
                ))
            })?;
            let result = EffectResult::decode(&bytes).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
            let at = Timestamp::new(result.at);
            at.to_datetime().map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
            if !self.state_read().pending_effects.iter().any(|pending| pending.ticket == effect.ticket) {
                continue;
            }
            self.commit(RecordEffect {
                ticket: effect.ticket,
                change: EffectChange::Restored,
                at,
            })
            .await
            .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn json_value(value: serde_json::Value) -> dokime_domain::model::values::Value {
    use dokime_domain::model::values::Value;
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(value) => Value::Bool(value),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                Value::Integer(value)
            } else if let Some(value) = value.as_u64() {
                Value::Unsigned(value)
            } else {
                Value::Number(value.as_f64().expect("JSON number"))
            }
        }
        serde_json::Value::String(value) => Value::Text(value),
        serde_json::Value::Array(values) => Value::Array(values.into_iter().map(json_value).collect()),
        serde_json::Value::Object(values) => Value::Object(values.into_iter().map(|(key, value)| (key, json_value(value))).collect()),
    }
}

fn execution_scope(
    runtime: &RuntimeHandle, call: &CallContext, owner: Option<&str>,
) -> Result<Option<(RecordScope, String, bool)>, Status> {
    let Some(scope) = ExecutionScope::current(call) else {
        return Ok(None);
    };
    let session = runtime
        .execution_sessions
        .get(&scope.execution)
        .ok_or_else(|| Status::not_found("execution is no longer accepting external changes"))?;
    if session.effects_closed() {
        return Err(Status::failed_precondition("execution is no longer accepting external changes"));
    }
    if let Some(owner) = owner
        && owner != session.source()
    {
        let delegated = runtime.routing.get().is_some_and(|router| {
            router.graph().ordered().iter().any(|provider| {
                provider.id == owner && provider.contributions.operations.iter().any(|operation| operation.id.as_str() == scope.route)
            })
        });
        if !delegated {
            return Err(Status::permission_denied("caller neither owns this execution nor serves its routed operation"));
        }
    }
    Ok(Some((session.effect_scope(), owner.unwrap_or(session.source()).to_owned(), !scope.stage.is_empty())))
}

/* === Tests === */
