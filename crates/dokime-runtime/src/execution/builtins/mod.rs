/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operations this process serves for its own steps through `ctx.host()`.
// `RunProcess` is always served here, so its child shares the execution's
// artifact folder, log and records. Without a host, as under the harness,
// `WatchFile` is served here too over a local file.

/* === Modules === */
mod process;
mod watch;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// Third Party Imports
use harmos::Message;
use harmos::operations::OperationDeclaration;
use tokio::sync::watch as signal;

// Local Imports
use super::instances::{ExecutionIdentity, Handles};
use crate::artifacts::Artifacts;
use dokime_domain::operations::files::WatchFile;
use dokime_domain::operations::process::RunProcess;
use crate::telemetry::Streams;
use crate::transport::Journal;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub(crate) type Served = Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + Send>>;

pub(crate) type Serve = fn(Binding, Vec<u8>, signal::Receiver<bool>) -> Served;

/* === Structs === */
// What a served operation reaches of the stage that called it, owned so the
// instance may outlive the call's borrow.
#[derive(Clone)]
pub(crate) struct Binding {
    pub(crate) artifacts: Artifacts,
    pub(crate) streams: Streams,
    pub(crate) journal: Arc<dyn Journal>,
    pub(crate) identity: Arc<ExecutionIdentity>,
    pub(crate) handles: Handles,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
// The served operation an id names, if this process serves it; `hosted` says
// whether a host is there to serve the rest.
pub(crate) fn served(operation: &str, hosted: bool) -> Option<Serve> {
    if operation == RunProcess::ID {
        return Some(|binding, request, cancel| {
            Box::pin(typed(request, move |request| {
                process::run(binding, request, cancel)
            }))
        });
    }
    if operation == WatchFile::ID && !hosted {
        return Some(|binding, request, cancel| {
            Box::pin(typed(request, move |request| {
                watch::run(binding, request, cancel)
            }))
        });
    }
    None
}

/* === Private Functions === */
async fn typed<Q, A, F>(request: Vec<u8>, serve: impl FnOnce(Q) -> F) -> Result<Vec<u8>, String>
where
    Q: Message,
    A: Message,
    F: Future<Output = Result<A, String>>,
{
    let request = Q::decode(request.as_slice()).map_err(|error| error.to_string())?;
    Ok(serve(request).await?.encode_to_vec())
}

/* === Tests === */
