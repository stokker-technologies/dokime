/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Dokime Plugin Schema Preparation Allowances

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

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

pub fn schema_defaults() -> harmos::config::SchemaSettings {
    let archive = dokime_domain::model::plugins::wire::INSTALL_ARCHIVE_MAX_BYTES as usize;
    harmos::config::SchemaSettings {
        // Metadata Decoding Charges Encoded Bytes, Fragment Storage And Resolved Graphs.
        preparation_bytes: NonZeroUsize::new(archive.checked_mul(64).expect("supported native address space")).unwrap(),
        preparation_work: NonZeroUsize::new(archive.checked_mul(16).expect("supported native address space")).unwrap(),
    }
}

/* === Private Functions === */

/* === Tests === */
