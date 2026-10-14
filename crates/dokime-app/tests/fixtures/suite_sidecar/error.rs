/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub(super) enum Error {
    #[error("fixture initialization refused")]
    InitializationRefused,
    #[error("fixture marker: {0}")]
    Marker(#[from] std::io::Error),
    #[error("dokime sdk: {0}")]
    Sdk(#[from] dokime::Error),
    #[error("fixture host call: {0}")]
    Host(#[from] dokime::harmos::sidecar::CallError),
    #[error("the host surface and the step call disagree")]
    HostDisagreed,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
