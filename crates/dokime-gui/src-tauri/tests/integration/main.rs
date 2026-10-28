/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![recursion_limit = "256"]

mod connection;
mod http;
mod native_session;
mod parameters;
mod rpc;
mod storage;

#[allow(dead_code)]
#[path = "../support/mod.rs"]
mod support;

use support::wire;
