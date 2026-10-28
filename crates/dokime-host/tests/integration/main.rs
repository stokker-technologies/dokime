/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// In-process host stories: the fixture host, the platform parsers and this
// machine's native collectors.

mod bluetooth;
mod collectors;
mod host;
#[path = "../support/mod.rs"]
pub mod support;
mod telemetry;
mod wifi;
