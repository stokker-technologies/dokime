/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Network Stream and Its Observer

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::Host;
use crate::domain::telemetry::Network;
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

/// Declare the Network Stream, Sampled at the Editable Network Interval While Observed.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    let stream = Stream::<Network>::new(Network::STREAM, Network::LABEL);
    let observer = Observer::operation(&stream, collect).interval_setting(settings::NETWORK_INTERVAL);
    Ok(plugin.stream(stream).observer(observer))
}

/* === Private Functions === */

async fn collect(ctx: &Context) -> Result<Network> {
    ctx.plugin::<Host>()?.network.collect(ctx).await
}

/* === Tests === */
