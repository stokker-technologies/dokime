/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod authoring;

mod dispatch;
#[cfg(feature = "host")]
pub(crate) mod secrets;

#[cfg(feature = "host")]
pub(crate) mod documents;

#[cfg(feature = "host")]
pub mod evidence;

#[cfg(feature = "host")]
pub mod execution;

#[cfg(feature = "host")]
pub mod exports;

#[cfg(feature = "host")]
pub mod files;

#[cfg(feature = "host")]
pub mod installed_plugins;

#[cfg(feature = "host")]
pub mod media;

#[cfg(feature = "host")]
pub mod parameters;

#[cfg(feature = "host")]
pub mod prompts;

#[cfg(feature = "host")]
pub mod settings;

#[cfg(feature = "host")]
pub mod telemetry;

#[cfg(feature = "host")]
pub(crate) mod traceability;

/* === Re-Exports === */

pub use dispatch::{ActionDispatcher, ActionDispatcherFuture, DispatchContext, OptionsResponse, StatusResponse};
pub use dokime_domain::model::plugins::*;

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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
