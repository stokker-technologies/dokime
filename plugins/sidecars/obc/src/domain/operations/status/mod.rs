/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use dokime::prelude::StatusResponse;

#[dokime::operation(
    id = "obc.bench.get_status",
    response = StatusResponse,
    description = "Reports bench readiness without advancing the Charger. The settings section selects this operation as its readiness probe.",
    policy = super::POLICY,
    resources = [crate::Resources]
)]
#[derive(Clone, PartialEq)]
pub struct GetStatus {}
