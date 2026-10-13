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
use crate::publication::{Publication, Publishable};
use crate::records::{PublishRecord, RecordPublication};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub struct Publisher {
    host: harmos::plugin::sidecar::Supervisor,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Publisher {
    pub fn new(host: &harmos::plugin::sidecar::Supervisor) -> Self {
        Self { host: host.clone() }
    }

    pub async fn publish<V>(&self, value: V) -> crate::Result<()>
    where
        V: Publishable,
    {
        let Publication::Record(request) = value.publication().into() else {
            return Err(crate::Error::Publish(
                crate::error::PublishFailure::Undeclared {
                    declared_as: "a stream row; publish rows through the Harmos publisher".into(),
                },
            ));
        };
        let publication = RecordPublication {
            request_json: serde_json::to_string(&request).map_err(|error| crate::Error::Call {
                operation: <PublishRecord as harmos::operations::OperationDeclaration>::ID,
                reason: error.to_string(),
            })?,
            produced_at: crate::core::Timestamp::now_utc().as_str().to_owned(),
        };
        self.host
            .operation::<PublishRecord>(publication)
            .await
            .map_err(|error| crate::Error::Call {
                operation: <PublishRecord as harmos::operations::OperationDeclaration>::ID,
                reason: error.to_string(),
            })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
