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
use crate::{Bench, Plugin};
use dokime::harmos::sidecar::{ObserverContext, PublishError, Publisher};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub struct Temperature;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[dokime::observer(streams(TemperatureRow), activation = "demand")]
impl Temperature {
    fn interval(_: &dokime::harmos::ExecutionContext) -> Result<std::time::Duration, PublishError> {
        Ok(std::time::Duration::from_millis(1_000))
    }

    async fn observe(&mut self, ctx: &ObserverContext<'_, Publisher<Plugin>>) -> Result<(), PublishError> {
        // A sensor that cannot be read publishes nothing this tick; the next
        // tick reads it again.
        let Ok(celsius) = ctx.resource::<Bench>().expect("plugin prepares its bench").sensor.read_celsius() else {
            return Ok(());
        };
        ctx.publish_on::<TemperatureRow>(&TemperatureRow { celsius }).await
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
