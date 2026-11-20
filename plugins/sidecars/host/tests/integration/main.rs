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
use prost_types::field_descriptor_proto::Label;

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

#[test]
fn every_host_stream_row_holds_only_fields_that_exist_together() -> dokime::Result<()> {
    // Admission Checks Only Interval Rows, So Every Producer Row Is Checked Here: No Optional, Repeated or Map Field.
    let declarations = plugin_host::declarations()?;
    let types = declarations.types.unwrap_or_default();
    let rows = types
        .file
        .iter()
        .flat_map(|file| file.message_type.iter().map(move |message| (format!("{}.{}", file.package(), message.name()), message)))
        .collect::<Vec<_>>();
    assert!(!declarations.streams.is_empty());
    for stream in &declarations.streams {
        let (_, row) = rows.iter().find(|(name, _)| name == stream.row.trim_start_matches('.')).expect("Declared Row Type");
        for field in &row.field {
            let together = field.label() != Label::Repeated && !field.proto3_optional() && field.oneof_index.is_none();
            assert!(together, "Stream {} Field {} Is Optional or Repeated", stream.id, field.name());
        }
    }
    Ok(())
}
