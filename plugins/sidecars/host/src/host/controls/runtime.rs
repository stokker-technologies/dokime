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

#[cfg(unix)]
use super::unix as native;
#[cfg(target_os = "windows")]
use super::windows as native;
use super::{AdapterSet, PlanSet, RadioSet, Restore};
use crate::{Error, Result, Workers, domain::controls::ControlState};
use dokime::operations::effects::{ConfirmEffect, EffectResult, PrepareEffect, ReadEnvironment};
use dokime::{
    Supervisor,
    harmos::{
        Context, Message, Operation, Response,
        operations::Host,
        state::{Invertible, Transaction},
    },
};
use std::sync::Arc;

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

macro_rules! implementation {
    ($request:ty, $variant:ident) => {
        impl Operation for $request {
            type Error = dokime::Refusal;
            async fn run(self, ctx: Context<'_, Self>) -> std::result::Result<Response<EffectResult>, Self::Error> {
                let state = ctx.resource::<State>();
                let host = dokime::plugin::refusal(state.caller(ctx.call()))?;
                let workers = ctx.resource::<Workers>();
                let owner = workers.clone();
                // The Owned Task Keeps The Gate Until Native Work Settles, Even If Its Caller Cancels.
                let task = workers.0.spawn(async move { state.execute(Change::$variant(self), host, owner).await });
                let result = dokime::plugin::refusal(dokime::plugin::refusal(task.await)?)?;
                Ok(Response::new(result))
            }
        }
    };
}
implementation!(RadioSet, Radio);
implementation!(AdapterSet, Adapter);
implementation!(PlanSet, Plan);
implementation!(Restore, Restore);

/* === Type Aliases === */

/* === Structs === */

pub struct State {
    owner: &'static str,
    gate: tokio::sync::Mutex<()>,
    host: std::sync::OnceLock<Supervisor>,
}

/* === Enums === */

#[derive(Clone)]
enum Change {
    Radio(RadioSet),
    Adapter(AdapterSet),
    Plan(PlanSet),
    Restore(Restore),
}

/* === Traits === */

/* === Implementations === */

impl State {
    pub(crate) fn new(owner: &'static str) -> Self {
        Self {
            owner,
            gate: tokio::sync::Mutex::new(()),
            host: std::sync::OnceLock::new(),
        }
    }

    pub(crate) fn attach(&self, host: Supervisor) -> Result<()> {
        self.host.set(host).map_err(|_| Error::Probe("control caller initialized twice".into()))
    }

    fn caller(&self, call: &dokime::harmos::operations::CallContext) -> Result<Supervisor> {
        let host = self.host.get().ok_or_else(|| Error::Probe("control caller is not initialized".into()))?.clone();
        match dokime::execution::ExecutionScope::current(call) {
            Some(scope) => host.with_value(scope).map_err(probe),
            None => Ok(host),
        }
    }

    async fn execute(&self, change: Change, host: Supervisor, workers: Arc<Workers>) -> Result<EffectResult> {
        let _gate = self.gate.lock().await;
        if let Change::Restore(restore) = change {
            let task = workers.0.spawn_blocking(move || native::apply(&restore.state, true));
            task.await.map_err(probe)??;
            return Ok(EffectResult { at: now() });
        }
        if matches!(change, Change::Adapter(_)) {
            require_administrator()?;
        }
        let environment = host.call::<ReadEnvironment>(ReadEnvironment {}).await.map_err(probe)?;
        let reading = change.clone();
        let task = workers.0.spawn_blocking(move || match reading {
            Change::Radio(request) => native::read_radio(&request.radio),
            Change::Adapter(request) => native::read_adapter(&request.adapter_id),
            Change::Plan(_) => crate::host::power::active_plan().map(|plan| ControlState {
                power_plan: Some(plan.id),
                ..Default::default()
            }),
            Change::Restore(_) => unreachable!(),
        });
        let before = task.await.map_err(probe)??;
        let mut after = before.clone();
        let (inverse, resource, description, allow_disconnect, disabling, radio, adapter) = match &change {
            Change::Radio(request) => {
                request.check(&before).map_err(probe)?;
                let inverse = request.inverse(&before);
                request.apply(&mut after);
                (
                    inverse,
                    format!("host.radio.{}", request.radio),
                    format!("Set {} Radio {}", request.radio, request.state),
                    request.allow_disconnect,
                    request.state == "off",
                    Some(request.radio.clone()),
                    None,
                )
            }
            Change::Adapter(request) => {
                request.check(&before).map_err(probe)?;
                let inverse = request.inverse(&before);
                request.apply(&mut after);
                (
                    inverse,
                    format!("host.adapter.{}", request.adapter_id),
                    format!("Set Adapter {} Enabled {}", request.adapter_id, request.enabled),
                    request.allow_disconnect,
                    !request.enabled,
                    None,
                    Some(request.adapter_id.clone()),
                )
            }
            Change::Plan(request) => {
                request.check(&before).map_err(probe)?;
                let inverse = request.inverse(&before);
                request.apply(&mut after);
                (inverse, "host.power.plan".into(), format!("Set Power Plan {}", request.plan_id), false, false, None, None)
            }
            Change::Restore(_) => unreachable!(),
        };
        if before == after {
            return Ok(EffectResult { at: now() });
        }
        if disabling {
            let pid = environment.backend_pid;
            let in_step = environment.in_step;
            let task =
                workers.0.spawn_blocking(move || guard_connection(pid, radio.as_deref(), adapter.as_deref(), allow_disconnect, in_step));
            task.await.map_err(probe)??;
        }
        let ticket = host
            .call::<PrepareEffect>(PrepareEffect {
                resource,
                inverse_operation: format!("{}.{}", self.owner, <Restore as dokime::harmos::operations::OperationDeclaration>::ID),
                inverse: inverse.encode(),
                description,
            })
            .await
            .map_err(probe)?
            .id
            .clone();
        let task = workers.0.spawn_blocking(move || native::apply(&after, false));
        if let Err(error) = task.await.map_err(probe)? {
            let task = workers.0.spawn_blocking(move || native::apply(&inverse.state, true));
            match task.await.map_err(probe)? {
                Ok(()) => {
                    host.call::<ConfirmEffect>(ConfirmEffect { ticket, at: now(), restored: true }).await.map_err(probe)?;
                }
                Err(restore) => return Err(Error::Probe(format!("{error}; immediate inverse failed: {restore}"))),
            }
            return Err(error);
        }
        let at = now();
        host.call::<ConfirmEffect>(ConfirmEffect { ticket, at: at.clone(), restored: false }).await.map_err(probe)?;
        Ok(EffectResult { at })
    }
}

/* === Public Functions === */

pub fn require_administrator() -> Result<()> {
    if !native::administrator()? {
        return Err(Error::Probe("host.network.adapter.set requires administrator".into()));
    }
    Ok(())
}

// The Caller Supplies Only A Backend-Verified Override, Never A Request Flag By Itself.
pub fn guard_connection(pid: u32, radio: Option<&str>, adapter: Option<&str>, allow_disconnect: bool, in_step: bool) -> Result<()> {
    if allow_disconnect && in_step {
        return Ok(());
    }
    let active = native::connected_adapters(pid)?;
    let adapters = crate::host::adapters::snapshot()?.adapters;
    for entry in &adapters {
        if !active.contains(&entry.id) {
            continue;
        }
        let affected_radio = match radio {
            Some(radio) => native::radio_affects(radio, entry)?,
            None => false,
        };
        if adapter == Some(entry.id.as_str()) || affected_radio {
            return Err(Error::Probe(
                "change would cut an active remote client or backend connection; an explicit step allow_disconnect override is required"
                    .into(),
            ));
        }
    }
    Ok(())
}

/* === Private Functions === */

fn now() -> String {
    dokime_domain::model::time::Timestamp::from_system_time(std::time::SystemTime::now()).to_string()
}
fn probe(error: impl std::fmt::Display) -> Error {
    Error::Probe(error.to_string())
}
/* === Tests === */
