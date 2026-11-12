/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Observation Watch Runtime

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Error, Result, client::{Context, Output}};
use dokime_protocol::v1;

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

pub const ID: &str = "dokime.observation.watch_runtime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::ObservationWatchRuntimeRequest,v1::ObservationWatchRuntimeOutput>(ID,settings.watch_timeout) }
pub async fn handle(context: &Context, request: v1::ObservationWatchRuntimeRequest) -> Result<Output<v1::ObservationWatchRuntimeOutput>> {
    let mut changed = context.host.changes();
    let current = super::execution_state::observed(context);
    if request.cursor.as_ref().is_some_and(|cursor| cursor.epoch != current.epoch || cursor.stream != current.stream || cursor.position > current.position) { return Err(Error::Conflict("Watch Runtime: Reconnect Without a Cursor to Resynchronize".into())); }
    let (sender,receiver) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    let context = context.clone();
    let timeout = context.settings.watch_timeout;
    tokio::spawn(async move {
        let work = async {
            let mut initial = true;
            loop {
                let revision = *changed.borrow_and_update();
                let cycles = match super::execution_state::cycles(&context) { Ok(cycles) => cycles,Err(error) => { let _ = sender.send(Err(super::status(error))).await; return; } };
                let shutting_down = context.host.shutting_down();
                let frame = v1::RuntimeStateFrame { cursor: Some(v1::Cursor { epoch: context.host.identity().epoch,stream: "runtime.state".into(),position: revision }),resynchronized: initial,shutting_down,cycle_count: cycles.len() as u64,operations: Vec::new() };
                let output = v1::ObservationWatchRuntimeOutput { value: Some(v1::observation_watch_runtime_output::Value::Item(frame)) };
                if sender.send(Ok(output)).await.is_err() { return; }
                if shutting_down { let _ = sender.send(Ok(v1::ObservationWatchRuntimeOutput { value: Some(v1::observation_watch_runtime_output::Value::Response(v1::ClientEmpty {})) })).await; return; }
                initial = false;
                tokio::select! { _ = sender.closed() => return,changed = changed.changed() => if changed.is_err() { return; } }
            }
        };
        if tokio::time::timeout(timeout,work).await.is_err() { let _ = sender.try_send(Err(tonic::Status::deadline_exceeded("Watch Runtime: Reconnect After the Watch Deadline"))); }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(receiver))
}

/* === Private Functions === */

/* === Tests === */
