/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Plugin Stories: The Admitted Catalog, Its Settings and Each Area's Collection and Controls

/* === Modules === */

mod adapters;
mod bluetooth;
mod controls;
mod cpu;
mod network;
mod power;
mod settings;
mod system;
mod wifi;

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

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn the_host_plugin_composes_an_admissible_catalog() -> dokime::Result<()> {
    // Compose Every Area's Settings, Streams, Observers and Operations Exactly as the Served Plugin Does.
    plugin_host::declarations()?;
    Ok(())
}
