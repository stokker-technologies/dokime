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

#[harmos::operation(resources = [crate::hosting::Host<GetSuite>], response = SuiteDefinition, id = "dokime.catalog.get_suite",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Catalog", method = "GetSuite"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSuite {
    pub suite_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetSuite);
}
