/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Recover Disconnected Remote Scripts Without Disturbing Another Client.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Session;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn scene(session: &mut Session) {
    let root = session.root.path().join("remote scripts/disconnections");
    Box::pin(super::super::remote::disconnections(&session.client, session.host.endpoint(), &root)).await;
}

/* === Private Functions === */

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn disconnections() {
    let mut session = Session::ready().await;
    scene(&mut session).await;
    session.stop().await;
}
