/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(all(feature = "runtime", unix))]
mod unix;
#[cfg(all(feature = "runtime", target_os = "windows"))]
mod windows;

/* === Re-Exports === */

#[cfg(all(feature = "runtime", unix))]
pub use unix::active_plan;
#[cfg(all(feature = "runtime", target_os = "windows"))]
pub use windows::active_plan;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::operation(id = "host.power.plan.get", response = Plan, policy = super::system::POLICY, visibility = Public,
    resources = [crate::Workers])]
#[derive(Clone, Default, PartialEq)]
pub struct GetPlan {}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Plan {
    pub id: String,
    pub name: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
