/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

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

pub use dokime_domain as domain;
pub use dokime_runtime::*;

#[cfg(feature = "client")]
pub use dokime_client as client;
