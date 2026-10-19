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
use super::{CONTROL, Resources};
use dokime_runtime::harmos::operations::{Host, Visibility};
use dokime_runtime::records::RecordPublication;
use dokime_runtime::{Refusal, Supervisor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::message]
pub(crate) struct Published {}

// A record the story hands the fixture to file under its own identity.
#[dokime_runtime::message]
#[dokime_runtime::operation(
    id = "fixture-sidecar.publish-record",
    response = Published,
    version = 1,
    policy = CONTROL,
    visibility = Visibility::Shared
)]
#[derive(Clone, PartialEq)]
pub(crate) struct RelayRecord {
    request_json: String,
    produced_at: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Resources {
    pub(super) async fn publish_record(&self, host: &Supervisor, relayed: RelayRecord) -> Result<Published, Refusal> {
        // Relayed as it came, so the host's own admission is what refuses a
        // publication it cannot file under this plugin's identity.
        let publication = RecordPublication { request_json: relayed.request_json, produced_at: relayed.produced_at };
        host.call(publication).await.map(|response| response.into_owned()).map_err(|error| Refusal::of(error.to_string()))?;
        Ok(Published {})
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
