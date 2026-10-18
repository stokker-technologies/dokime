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

/* === Constants === */
pub const RECORD_PUBLICATION_VERSION: u32 = 1;

const RECORD_POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
    timeout: std::time::Duration::from_secs(30),
    attempt_timeout: std::time::Duration::from_secs(30),
    retries: harmos::operations::RetryPolicy::Never,
    restart: harmos::operations::RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Files one process-scoped record against the calling plugin.
#[harmos::message]
#[harmos::operation(response = (), id = "dokime.records.publish", version = 1, policy = RECORD_POLICY, visibility = harmos::operations::Visibility::Shared)]
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
