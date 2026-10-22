/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::domain::model::connections::Connections;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::message]
#[dokime::operation(
    id = "obc-plugin.list_connections",
    response = Connections,
    version = 1,
    description = "Reports the units this process has been told about. The connection listener accumulates them across executions; a template calls the operation without holding the plugin.",
    policy = super::POLICY
)]
#[derive(Clone, PartialEq)]
pub struct ListConnections {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
