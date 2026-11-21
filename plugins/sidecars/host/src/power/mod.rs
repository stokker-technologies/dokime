/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Power Plan Operations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::operations::{Context, Operation};
use dokime::plugin::Plugin;
use dokime::{Error, Message, Result, Visibility};

/* === Constants === */

pub const PLAN_GET: &str = "host.power.plan.get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Power State Retained Across Operation Calls.
#[derive(Default)]
pub struct State {}

/// Read the Active Power Plan Without Changing It.
#[derive(Message)]
#[dokime(package = "host")]
pub struct PlanRead {}

/// The Active Power Plan: a Windows Scheme GUID or a Linux ACPI Platform Profile, With Its Friendly Name.
#[derive(Message)]
#[dokime(package = "host")]
pub struct Plan {
    pub id: String,
    pub name: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the Power Operations.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    let read = Operation::new(PLAN_GET, Visibility::Public, async |_: &Context, _: PlanRead| {
        let task = tokio::task::spawn_blocking(active_plan);
        task.await.map_err(|error| Error::Invalid(format!("Read Power Plan: Retry After the Power Worker Stopped: {error}")))?
    })
    .description("Report the Active Power Plan and Its Friendly Name.");
    Ok(plugin.operation(read))
}

/// The Active Power Plan, Read Natively Without Starting a Process.
#[cfg(windows)]
pub fn active_plan() -> Result<Plan> {
    let (id, name) = crate::platform::windows::power::active_plan()?;
    Ok(Plan { id, name })
}

/// The Active Power Plan, Read From the ACPI Platform Profile.
#[cfg(unix)]
pub fn active_plan() -> Result<Plan> {
    let (id, name) = crate::platform::unix::power::active_plan()?;
    Ok(Plan { id, name })
}

/* === Private Functions === */

/* === Tests === */
