/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod wire_cursor;
pub mod wire_checkpoint;
pub mod wire;
pub mod command;
pub mod scope;

/* === Re-Exports === */
pub use command::CommandContext;
pub use scope::CapturePhase;
pub use scope::EvidenceOrigin;
pub use scope::RecordScope;
pub use scope::scope_visible_in;

/* === Public Functions === */

/* === Private Functions === */
