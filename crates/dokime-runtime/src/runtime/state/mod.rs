/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod aggregate;

pub use crate::state::RuntimeState;
pub(crate) use aggregate::{evidence_counts, overview_entity_counts, runtime_overview_snapshot};
