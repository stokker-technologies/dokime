/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod app;
pub mod bindings;
pub mod connection;
mod error;
pub mod native_session;
pub mod parameters;
pub mod rpc;

/* === Re-Exports === */

pub use app::run;
pub use error::Error;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

pub const PRODUCT: &str = env!("DOKIME_BRAND_LABEL");

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
