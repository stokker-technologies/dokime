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
use super::Resources;
use dokime::harmos::operations::Host;
use dokime::records::{PublishRecord, RecordPublication};
use dokime::{Refusal, Supervisor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::message]
pub(crate) struct Published {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Resources {
    pub(super) async fn publish_record(
        &self,
        host: &Supervisor,
        publication: RecordPublication,
    ) -> Result<Published, Refusal> {
        // Relayed as it came, so the host's own admission is what refuses a
        // publication it cannot file under this plugin's identity.
        host.call_as::<PublishRecord>(publication)
            .await
            .map(|response| response.into_owned())
            .map_err(|error| Refusal::of(error.to_string()))?;
        Ok(Published {})
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
