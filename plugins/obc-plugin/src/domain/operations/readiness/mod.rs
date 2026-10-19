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
use crate::domain::model::readiness::Readiness;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime_runtime::message]
#[dokime_runtime::operation(
    id = "obc-plugin.check_readiness",
    response = Readiness,
    version = 1,
    description = "Reports whether the testbench will still take work. A template calls it with `AskReadiness {}`.",
    policy = super::POLICY
)]
#[derive(Clone, PartialEq)]
pub struct AskReadiness {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
