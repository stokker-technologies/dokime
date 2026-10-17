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
use crate::{Error, Resources, TelemetrySidecar};
use dokime_runtime::harmos::sidecar::{ObserverContext, Publisher};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::stream(
    id = "validation-telemetry.continuous",
    window = 4096,
    label = "Fixture Continuous",
    description = "Continuous samples produced outside any execution.\nThe stream this fixture measures, declared by the row that travels it."
)]
pub struct FixtureRow {
    value: f64,
}

#[derive(Default)]
pub(crate) struct Continuous;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::observer(interval_ms = 5, activation = "demand", streams(FixtureRow))]
impl Continuous {
    async fn observe(
        &mut self,
        ctx: &ObserverContext<'_, Resources, Publisher<TelemetrySidecar>>,
    ) -> Result<(), Error> {
        let value = (ctx.tick().ordinal() - 1) as f64 * 0.5;
        ctx.publish_on::<FixtureRow>(&FixtureRow { value }).await?;

        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
