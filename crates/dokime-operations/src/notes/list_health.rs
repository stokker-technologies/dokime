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

#[harmos::operation(resources = [crate::hosting::Host<ListHealth>], response = ListHealthResponse, id = "dokime.health.list_health",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Health", method = "ListHealth"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListHealth {
    pub selection: Option<AnnotationSelection>,

    // default 50, maximum 100
    #[row(description = "default 50, maximum 100")]
    pub limit: u32,

    pub after_id: String,

    // Exact runtime revision and selection, required with after_id. Restart on change.
    #[row(description = "Exact runtime revision and selection, required with after_id. Restart on change.")]
    pub cursor: Option<Cursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListHealth);
}
