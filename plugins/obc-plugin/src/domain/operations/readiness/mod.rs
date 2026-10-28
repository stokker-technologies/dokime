/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::domain::model::readiness::Readiness;

#[dokime::operation(
    id = "obc.bench.check_readiness",
    response = Readiness,
    description = "Reports whether the testbench will still take work. A template calls it with `AskReadiness {}`.",
    policy = super::POLICY,
    resources = [crate::Resources]
)]
#[derive(Clone, PartialEq)]
pub struct AskReadiness {}
