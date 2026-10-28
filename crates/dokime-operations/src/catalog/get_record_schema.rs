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

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<GetRecordSchema>], response = RecordSchema, id = "dokime.catalog.get_record_schema",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Catalog", method = "GetRecordSchema"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetRecordSchema {
    pub record_type: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetRecordSchema);
}
