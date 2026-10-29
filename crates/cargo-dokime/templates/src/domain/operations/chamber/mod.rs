/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


use crate::domain::model::chamber::ChamberReading;
use dokime::prelude::Discovered;


#[dokime::operation(
    id = "{{crate}}.get_chamber",
    response = ChamberReading,
    description = "Reads the chamber. A template calls it with `ctx.call::<ReadChamber>(ReadChamber {})`.",
    policy = super::POLICY,
    resources = [crate::Bench]
)]
#[derive(Clone, PartialEq)]
pub struct ReadChamber {}

#[dokime::operation(
    id = "{{crate}}.detect_port",
    response = Discovered,
    description = "Finds the sensor port the settings card offers before anybody saved one.",
    policy = super::POLICY,
    resources = [crate::Bench]
)]
#[derive(Clone, PartialEq)]
pub struct DetectPort {}
