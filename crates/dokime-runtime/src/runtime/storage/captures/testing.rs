/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Count Physical Seals Across Concurrent And Cancelled Close Waiters.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

// Third Party Imports
use futures_util::FutureExt;
use tokio::sync::oneshot;

// Local Imports
use super::Captures;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn joined_drains() {
    let captures = Arc::new(Captures::new());
    let execution = TestExecutionId::from_seed("joined-capture-drains");
    let invocations = Arc::new(AtomicUsize::new(0));
    let mut requests = Vec::new();
    for _ in 0..2 {
        let (request, received) = oneshot::channel();
        captures.register(&execution, request).unwrap();
        requests.push(received);
    }
    let first = {
        let captures = Arc::clone(&captures);
        let execution = execution.clone();
        tokio::spawn(async move { captures.close(&execution).await })
    };
    let mut completions = Vec::new();
    for request in requests {
        completions.push(request.await.expect("original seal request"));
        invocations.fetch_add(1, Ordering::SeqCst);
    }
    let (late, _) = oneshot::channel();
    assert!(captures.register(&execution, late).is_err(), "closing an execution fences new capture admission");
    let second = captures.close(&execution);
    tokio::pin!(second);
    assert!(second.as_mut().now_or_never().is_none(), "a concurrent closer must await the original seals");
    first.abort();
    assert!(first.await.unwrap_err().is_cancelled());
    assert!(second.as_mut().now_or_never().is_none(), "cancelling the first waiter must not complete its drain");
    completions.remove(0).send(()).unwrap();
    assert!(second.as_mut().now_or_never().is_none(), "all captures must seal before returning");
    completions.remove(0).send(()).unwrap();
    second.await;
    captures.close(&execution).await;
    let (late, _) = oneshot::channel();
    assert!(captures.register(&execution, late).is_err(), "sealed captures cannot reopen during a late retry");
    assert_eq!(invocations.load(Ordering::SeqCst), 2, "each capture receives exactly one physical seal request");
    let state = captures.state.lock().unwrap();
    assert!(state.open.is_empty() && state.closing.is_empty(), "completed executions retain no drain history");
}

/* === Private Functions === */

/* === Tests === */
