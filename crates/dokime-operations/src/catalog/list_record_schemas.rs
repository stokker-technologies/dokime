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

// Current admitted schemas, ordered by type/stream ID. These catalogs exclude
// retired definitions retained only for immutable historical evidence. A cursor
// belongs to one runtime epoch/revision; restart pagination when it expires.
#[harmos::operation(resources = [crate::hosting::Host<ListRecordSchemas>], response = ListRecordSchemasReply, id = "dokime.catalog.list_record_schemas",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Catalog", method = "ListRecordSchemas"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListRecordSchemas {
    pub limit: u32,
    pub after_record_type: String,
    pub cursor: Option<Cursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListRecordSchemas);
}
