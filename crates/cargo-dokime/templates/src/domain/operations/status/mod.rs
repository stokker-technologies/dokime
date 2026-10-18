/* ***
 *
 * {{package}}
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
// Each request is its operation: the struct is what a caller sends, and the
// attribute is everything the host reads about it before the plugin runs.
#[dokime_runtime::message]
#[dokime_runtime::operation(
    id = "{{owner}}.get_status",
    response = StatusResponse,
    version = 1,
    description = "Reports whether the bench is reachable.",
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
