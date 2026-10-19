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
use crate::domain::model::chamber::ChamberReading;
use dokime_runtime::prelude::Discovered;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::message]
#[dokime_runtime::operation(
    id = "{{owner}}.get_chamber",
    response = ChamberReading,
    version = 1,
    description = "Reads the chamber. A template calls it with `ctx.call::<ReadChamber>(ReadChamber {})`.",
    policy = super::POLICY
)]
#[derive(Clone, PartialEq)]
pub struct ReadChamber {}

#[dokime_runtime::message]
#[dokime_runtime::operation(
    id = "{{owner}}.detect_port",
    response = Discovered,
    version = 1,
    description = "Finds the sensor port the settings card offers before anybody saved one.",
    policy = super::POLICY
)]
#[derive(Clone, PartialEq)]
pub struct DetectPort {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
