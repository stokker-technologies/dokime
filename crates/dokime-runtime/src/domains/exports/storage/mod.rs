/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod diagnostics;
pub(crate) mod error;
pub(crate) mod paths;

/* === Re-Exports === */
pub use diagnostics::{
    DIAGNOSTICS_COLLECT_BUNDLE_FILENAME, DIAGNOSTICS_SNAPSHOT_FILENAME, DiagnosticFileFormat, DiagnosticPathConvention, DiagnosticPathKey,
    HUB_LOG_FILENAME, STARTUP_DOCTOR_FILENAME, path_conventions,
};
pub use error::{Result, StorageError};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
