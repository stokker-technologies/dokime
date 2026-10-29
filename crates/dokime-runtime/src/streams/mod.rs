/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::{Deserialize, Serialize};

use dokime_domain::model::identity::Metadata;
use dokime_domain::model::values::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AggregateOutput {
    pub key: String,
    pub metadata: Metadata,
    pub value: Value,
}
