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

#[tokio::test]
async fn server_pairing_remote() {
    Box::pin(server::unsecured_pairing_creates_no_revocable_client()).await;
    Box::pin(server::independent_pairings_revoke_and_survive_restart()).await;
    Box::pin(server::remote_sessions_recorded()).await;
    Box::pin(server::listener_policy_isolation()).await;
    Box::pin(server::remote_cycle_seal()).await;
    Box::pin(server::operation_response_files()).await;
    Box::pin(server::discovery_shares_independently()).await;
    Box::pin(server::malformed_secrets_wire()).await;
    Box::pin(server::remote_requires_binding()).await;
    Box::pin(server::tls_listener_clients()).await;
    Box::pin(server::operations::response_publication_faults()).await;
    Box::pin(server::operations::response_unavailable_after_effect()).await;
}
