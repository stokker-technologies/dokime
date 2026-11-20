/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// System Streams and Their Observers

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::Host;
use crate::domain::telemetry::System;
use crate::settings;
use dokime::Result;
use dokime::operations::Context;
use dokime::plugin::{Observer, Plugin, Stream};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the System Stream, Sampled at the Editable System Interval.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    let stream = Stream::<System>::new(System::STREAM, System::LABEL);
    let observer = Observer::operation(&stream, collect).interval_setting(settings::SYSTEM_INTERVAL);
    Ok(plugin.stream(stream).observer(observer))
}

/* === Private Functions === */

async fn collect(ctx: &Context) -> Result<System> {
    ctx.plugin::<Host>()?.system.collect(ctx).await
}

/* === Tests === */
