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
use dokime::harmos::plugin::sidecar::{DemandKey, ObserverContext, Publisher};

/* === Constants === */
const STARTED: DemandKey = DemandKey::new("dokime.record", "validation-telemetry.started");

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::record(id = "validation-telemetry.started", label = "Observer Started")]
pub struct Started {
    ordinal: u64,
}

pub(crate) struct RecordDemand {
    marker: Option<std::path::PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime::observer(interval_ms = 5, activation = "demand", demand_keys(STARTED))]
impl RecordDemand {
    async fn initialize(resources: &Resources) -> Result<Self, Error> {
        let marker = resources.record_lifecycle.clone();
        if let Some(path) = &marker {
            std::fs::write(path.with_extension("started"), b"started")?;
        }

        Ok(Self { marker })
    }

    async fn observe(
        &mut self,
        ctx: &ObserverContext<'_, Resources, Publisher<TelemetrySidecar>>,
    ) -> Result<(), Error> {
        if ctx.tick().ordinal() == 1 {
            dokime::plugin::Publisher::new(ctx.capabilities().supervisor())
                .publish(Started { ordinal: 1 })
                .await?;
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Error> {
        if let Some(path) = &self.marker {
            std::fs::write(path.with_extension("stopped"), b"stopped")?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
