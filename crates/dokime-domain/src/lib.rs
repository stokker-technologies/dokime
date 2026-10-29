/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[macro_use]
mod macros;
mod error;
pub mod model;
pub mod process;
pub mod state;

pub use error::Error;
pub use error::Result;

pub const DEV_PROTOCOL: u32 = 4;
pub const READINESS_PROTOCOL: u64 = 3;
