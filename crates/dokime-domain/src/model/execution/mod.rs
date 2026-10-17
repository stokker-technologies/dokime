/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod command;
pub mod scope;

/* === Re-Exports === */
pub use command::CommandContext;
pub use scope::{CapturePhase, EvidenceOrigin, RecordScope, scope_visible_in};

/* === Public Functions === */

/* === Private Functions === */
