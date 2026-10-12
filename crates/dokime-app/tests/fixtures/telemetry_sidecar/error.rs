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
    #[error(transparent)]
    Publication(#[from] dokime::harmos::sidecar::PublishError),
    #[error(transparent)]
    Published(#[from] dokime::Error),
    #[error("fixture lifecycle marker: {0}")]
    LifecycleMarker(#[from] std::io::Error),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
