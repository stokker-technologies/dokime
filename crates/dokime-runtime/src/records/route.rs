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

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RecordPublication {
    #[row(
        description = "The record, JSON-encoded as a [`RecordPublishRequest`](crate::records::RecordPublishRequest). Its type is resolved against the contracts the publishing sidecar declared, and it is dropped when it resolves nowhere."
    )]
    pub request_json: String,
    #[row(
        description = "When the sidecar published it, RFC 3339. Stamped at publication rather than on arrival, so a queued record keeps the instant it happened instead of the instant the host got around to it."
    )]
    pub produced_at: String,
}

pub struct PublishRecord;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::operations::OperationDeclaration for PublishRecord {
    type Request = RecordPublication;
    type Response = ();
    const ID: &'static str = "dokime.records.publish";
    const DESCRIPTION: &'static str = "Files one process-scoped record against the calling plugin.";
    const POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(30),
        attempt_timeout: std::time::Duration::from_secs(30),
        retries: harmos::operations::RetryPolicy::Never,
        restart: harmos::operations::RestartPolicy::Never,
    };
    const VISIBILITY: harmos::operations::Visibility = harmos::operations::Visibility::Shared;
}

impl harmos::operations::WireOperation for PublishRecord {
    const VERSION: u32 = 1;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
