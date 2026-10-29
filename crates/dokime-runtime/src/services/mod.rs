/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod telemetry;

// One lease on a resident service instance of the operation manager; the
// last lease released stops the instance.
pub(crate) type Lease = harmos::Running<harmos::operations::resident::Activation>;
