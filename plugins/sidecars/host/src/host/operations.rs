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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
pub enum Operations {
    CaptureAdapters(super::adapters::CaptureAdapters),
    RadioSet(super::controls::RadioSet),
    AdapterSet(super::controls::AdapterSet),
    PlanSet(super::controls::PlanSet),
    Restore(super::controls::Restore),
    Adapters(super::adapters::Adapters),
    CpuLoad(super::cpu::CpuLoad),
    GetPlan(super::power::GetPlan),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
