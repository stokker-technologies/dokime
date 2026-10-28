/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Match The Runtime's Nested Native Startup Future Layout.
#![recursion_limit = "256"]

mod app;
pub mod bindings;
pub mod connection;
mod error;
pub mod native_session;
pub mod parameters;
pub mod rpc;

pub use app::run;
pub use error::Error;

pub const PRODUCT: &str = env!("DOKIME_BRAND_LABEL");
