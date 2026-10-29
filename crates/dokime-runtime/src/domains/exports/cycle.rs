/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::artifacts::ArtifactRecord;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactContent {
    pub record: ArtifactRecord,
    pub bytes: Vec<u8>,
}
