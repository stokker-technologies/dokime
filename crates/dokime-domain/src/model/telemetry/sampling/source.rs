/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use crate::model::identity::Identifier;
use crate::model::identity::Metadata;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetrySourceDescriptor {
    pub identifier: Identifier,
    pub metadata: Metadata,
    pub streams: Vec<Identifier>,
}
