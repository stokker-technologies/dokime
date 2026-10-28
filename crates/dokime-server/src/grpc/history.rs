/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History operations: browsing, reading and inspecting recorded executions.

mod browse;
mod capture;
mod context;
mod cycles;
pub(super) mod document;
mod evidence;
mod inspection;
mod read;
mod records;
mod telemetry;
mod window;
mod wire;

pub(super) use inspection::register;
pub(crate) use wire::traceability;
