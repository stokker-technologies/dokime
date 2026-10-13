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
use crate::{Bench, Plugin};
use dokime::harmos::plugin::sidecar::{ObserverContext, PublishError, Publisher};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// The row type declares the stream, so the id is written once and every
// column comes with a handle a plot can name it by:
// `TemperatureRow::celsius()` stops resolving the day the field is renamed.
// The observer below says how often, and everything that reads these rows names
// this type.
#[dokime::stream(
    id = "{{owner}}.temperature",
    window = 4096,
    label = "Bench Temperature"
, description = "The chamber's temperature, independent of any execution.")]
pub struct TemperatureRow {
    #[row(unit = "°C")]
    pub celsius: f64,
}

#[derive(Default)]
pub struct Temperature;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime::observer(interval_ms = 1_000, streams(TemperatureRow), activation = "demand")]
impl Temperature {
    async fn observe(
        &mut self,
        ctx: &ObserverContext<'_, Bench, Publisher<Plugin>>,
    ) -> Result<(), PublishError> {
        // A sensor that cannot be read publishes nothing this tick; the next
        // tick reads it again.
        let Ok(celsius) = ctx.resources().sensor.read_celsius() else {
            return Ok(());
        };
        ctx.publish_on::<TemperatureRow>(&TemperatureRow { celsius })
            .await
    }

    // An observer that opened a port, a file or a child process closes it in an
    // `async fn stop(&mut self) -> Result<(), Error>` here, which runs on this observer's own task when
    // the last lease ends — before the plugin is handed back its own handles.
    // This one borrows the bench's shared sensor and has nothing of its own to
    // close, so it keeps the trait's empty ending.
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
