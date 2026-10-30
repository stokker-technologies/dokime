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

/* === Traits === */

/* === Implementations === */

impl dokime::harmos::Operation for super::CaptureAdapters {
    type Error = dokime::Refusal;
    async fn run(
        self, ctx: dokime::harmos::Context<'_, Self>,
    ) -> Result<dokime::harmos::Response<dokime::operations::effects::StartEvidence>, Self::Error> {
        let timeout = std::time::Duration::from_millis(ctx.resource::<crate::HostOptions>().timing.network_probe_timeout_ms);
        let task = ctx.resource::<crate::Workers>().0.spawn_blocking(move || super::snapshot(timeout));
        let snapshot = dokime::plugin::refusal(dokime::plugin::refusal(task.await)?)?;
        let payload_json = dokime::plugin::refusal(serde_json::to_string(&snapshot))?;
        Ok(dokime::harmos::Response::new(dokime::operations::effects::StartEvidence { payload_json }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
