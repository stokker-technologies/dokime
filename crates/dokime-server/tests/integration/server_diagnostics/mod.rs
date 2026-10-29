/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Diagnostic collection, log streaming and trace journal watches.

/* === Modules === */

mod diagnostics;
mod trace;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn server_diagnostics() {
    let journal = dokime_server::trace::Journal::new(dokime_server::trace::Limits::default());
    let (environment, server, client) = diagnostics::fixture(journal.clone()).await;
    diagnostics::collected_archive_material(&client).await;
    let mut logs = diagnostics::log_roundtrip(&environment, &client).await;
    trace::roundtrip(&environment, &client, &journal).await;
    diagnostics::collected_tail_preserves_selected_bytes(&environment, &client).await;
    server.shutdown().await.unwrap();
    assert_eq!(logs.message().await.unwrap_err().code(), Code::Unavailable);
    drop(environment);
}
