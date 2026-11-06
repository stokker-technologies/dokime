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
use crate::domain::model::chamber::TemperatureRow;
use crate::operations::ReadChamber;
use dokime::harmos::operations::Host;
use dokime::harmos::sidecar::{ObserverContext, Publisher};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub struct Temperature;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[dokime::observer(interval_ms = 1_000, streams(TemperatureRow), activation = "demand")]
impl Temperature {
    async fn observe(&mut self, ctx: &ObserverContext<'_, Publisher<crate::Plugin>>) -> Result<(), dokime::Refusal> {
        // Read Chamber Temperature
        let operation = ReadChamber {};
        let reading = dokime::plugin::refusal(ctx.host().supervisor().call::<ReadChamber>(operation).await)?;
        let row = TemperatureRow { celsius: reading.celsius };
        dokime::plugin::refusal(ctx.publish_on::<TemperatureRow>(&row).await)?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
