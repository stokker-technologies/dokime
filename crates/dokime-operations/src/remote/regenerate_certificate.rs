/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Explicitly Regenerate Remote Certificate Identity

use serde::{Deserialize, Serialize};

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<RegenerateCertificate>], response = RemoteListenerStatus, id = "dokime.remote.regenerate_certificate",
    policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "RegenerateCertificate"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegenerateCertificate {
    pub certificate_names: Vec<String>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(RegenerateCertificate);
}
