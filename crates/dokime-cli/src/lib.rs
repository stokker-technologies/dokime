/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![recursion_limit = "256"]

mod cli;
mod commands;

pub use cli::Cli;
pub use dokime::client_error::{Error, Result};

use dokime::{connection, installation, terminal, version};
