/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Manager Graphs Need This Depth To Prove Generated RPC Futures Are Send.

#![recursion_limit = "256"]

mod error;
pub mod grpc;
pub mod host;
pub mod trace;

pub use error::{Error, Result};
