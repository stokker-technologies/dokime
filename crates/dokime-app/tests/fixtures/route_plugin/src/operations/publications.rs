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
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::Resources;
use dokime::records::RecordPublication;
use dokime::{Refusal, Supervisor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::record(id = "fixture.route.observed", label = "Route Observed")]
pub struct Observed {
    pub(crate) value: f64,
}

#[dokime::message]
pub(crate) struct Published {}

pub(crate) struct PublishRecord;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime::operation(
    id = "route-fixture.publish_record",
    version = 1,
    description = "Relays one record publication to the host as it came.",
    policy = dokime::harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(1),
        attempt_timeout: std::time::Duration::from_secs(1),
        retries: dokime::harmos::operations::RetryPolicy::Never,
    }
)]
impl PublishRecord {
    pub(crate) fn new(_: Arc<Resources>) -> Self {
        Self
    }

    async fn run(
        &mut self,
        host: &Supervisor,
        request: &RecordPublication,
    ) -> Result<Published, Refusal> {
        // The host's own admission is what refuses a publication this plugin
        // did not declare.
        dokime::plugin::refusal(
            host.operation::<dokime::records::PublishRecord>(request.clone())
                .await,
        )?;
        Ok(Published {})
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
