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
// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{AggregateReply, AggregateRequest, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Read-only and idempotent. Requires the read role; no permission beyond the one every other read method uses. Replies are bounded by `limit` and by a server ceiling of 10,000 rows, and additionally by the negotiated gRPC message limit — a reply that would exceed it fails with ResourceExhausted rather than being silently truncated. Safe to retry: the call has no effect, and repeating it observes the original saved frontier selected for that call.  Coverage: objects contribute their latest logical revision at this call's original saved frontier. Steps use each execution's frozen case snapshot; records retain their original journal sequence and process identity. Staged evidence is absent until saved. Planned executions with no saved revision and executions retired by deletion are absent. Page decode, reduction, outcome projection and canonical response encoding share the original query's configured decoded allowance and active-time budget. The response retains its original loan through transport encoding.
#[harmos::operation(request = AggregateRequest, response = AggregateReply, id = "dokime.query.aggregate", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Query", method = "Aggregate"))]
pub struct Aggregate;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
