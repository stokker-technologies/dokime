/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod memory;
mod plugins;
mod preferences;
mod sources;
mod wire;
/* === Re-Exports === */
/* === Imports === */
// Standard Library Imports
// Third Party Imports
// Local Imports
use dokime_app::RuntimeHandle;
/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
/* === Enums === */
/* === Traits === */
/* === Implementations === */
/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    preferences::register(runtime, maximum)?;
    sources::register(runtime, maximum)?;
    plugins::register(runtime, maximum)
}
/* === Private Functions === */
/* === Tests === */
