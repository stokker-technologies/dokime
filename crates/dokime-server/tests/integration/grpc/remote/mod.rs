/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Pairing, remote sessions, listener policy and operation response publication.

/* === Modules === */

mod persistence;
mod server;

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn pairing() {
    stage(Box::pin(server::unsecured_pairing_has_no_key())).await;
    stage(Box::pin(server::pairing_rotation_survives_restart())).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn listener_policy() {
    stage(Box::pin(server::listener_policy_isolation())).await;
    stage(Box::pin(server::remote_cycle_seal())).await;
    stage(Box::pin(server::operation_response_files())).await;
    stage(Box::pin(server::discovery_shares_independently())).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn transport() {
    stage(Box::pin(server::malformed_secrets_wire())).await;
    stage(Box::pin(server::remote_requires_binding())).await;
    stage(Box::pin(server::tls_listener_clients())).await;
    stage(Box::pin(server::names_refresh_preserves_remote_stream())).await;
    stage(Box::pin(server::client_retries_reads_without_replaying_mutations())).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn publication() {
    stage(Box::pin(server::operations::response_publication_faults())).await;
    stage(Box::pin(server::operations::response_unavailable_after_effect())).await;
}
