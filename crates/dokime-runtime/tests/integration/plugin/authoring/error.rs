/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[derive(Debug, thiserror::Error)]
pub(super) enum Error {
    #[error("row {0} carries no value")]
    Unmeasured(usize),
}
