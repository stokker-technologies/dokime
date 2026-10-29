/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::domain::model::connections::Connections;

#[dokime::operation(
    id = "obc.bench.list_connections",
    response = Connections,
    description = "Reports the units this process has been told about. The connection listener accumulates them across executions; a template calls the operation without holding the plugin.",
    policy = super::POLICY,
    resources = [crate::Resources]
)]
#[derive(Clone, PartialEq)]
pub struct ListConnections {}
