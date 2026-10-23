/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "client")]
#[path = "error.rs"]
pub mod client_error;
#[cfg(feature = "client")]
pub mod connection;
#[cfg(feature = "client")]
pub mod installation;
#[cfg(feature = "terminal")]
pub mod terminal;
pub mod version;

/* === Re-Exports === */

pub use dokime_domain as domain;
pub use dokime_runtime::*;

#[cfg(feature = "client")]
pub use dokime_client as client;

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
