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
use std::time::Duration;

// Third Party Imports
use harmos::operations::OperationPolicy;

// Local Imports

/* === Constants === */

const RECORD: OperationPolicy = crate::policy(Duration::from_secs(30));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Files one process-scoped record against the calling plugin.
#[harmos::operation(resources = [crate::hosting::Host<RecordPublication>, harmos::operations::HostContext], id = "dokime.records.publish",
    policy = RECORD, visibility = Public)]
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RecordPublication {
    #[row(
        description = "The record, JSON-encoded as a [`RecordPublishRequest`](dokime_domain::model::records::publication::RecordPublishRequest). Its type is resolved against the contracts the publishing sidecar declared, and it is dropped when it resolves nowhere."
    )]
    pub request_json: String,

    #[row(
        description = "When the sidecar published it, RFC 3339. Stamped at publication rather than on arrival, so a queued record keeps the instant it happened instead of the instant the host got around to it."
    )]
    pub produced_at: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
