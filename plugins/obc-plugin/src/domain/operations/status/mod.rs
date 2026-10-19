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
use dokime_runtime::prelude::StatusResponse;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime_runtime::message]
#[dokime_runtime::operation(
    id = "obc-plugin.get_status",
    response = StatusResponse,
    version = 1,
    description = "Reports bench readiness without advancing the Charger. The settings section selects this operation as its readiness probe.",
    policy = super::POLICY
)]
#[derive(Clone, PartialEq)]
pub struct GetStatus {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
