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

impl dokime::harmos::Operation for super::GetPlan {
    type Error = dokime::prelude::Refusal;
    async fn run(self, ctx: dokime::harmos::Context<'_, Self>) -> Result<dokime::harmos::Response<super::Plan>, Self::Error> {
        let task = ctx.resource::<crate::Workers>().0.spawn_blocking(active_plan);
        let plan = dokime::plugin::refusal(dokime::plugin::refusal(task.await)?)?;
        Ok(dokime::harmos::Response::new(plan))
    }
}

/* === Public Functions === */

pub fn active_plan() -> crate::Result<super::Plan> {
    let profile = std::fs::read_to_string("/sys/firmware/acpi/platform_profile")
        .map_err(|error| crate::Error::Probe(format!("ACPI platform power profile unavailable: {error}")))?;
    let id = profile.trim().to_owned();
    Ok(super::Plan { name: Some(id.clone()), id })
}

/* === Private Functions === */

/* === Tests === */
