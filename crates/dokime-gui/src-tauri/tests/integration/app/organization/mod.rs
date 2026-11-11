/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Organization Pack Lifecycle Stories

/* === Modules === */

#[allow(dead_code)]
#[path = "../../../../src/app/organization.rs"]
mod implementation;

mod a_staged_pack_activates_on_the_next_boot_and_an_invalid_one_falls_back;
mod failed_promotion_restores_the_active_pack;
mod invalid_pending_pack_preserves_the_last_valid_active_pack;
mod invalid_update_policy_preserves_the_previous_pending_pack;
mod malformed_policy_diagnostics_do_not_echo_manifest_values;
mod partial_pack_inherits_assets;
mod the_repository_organization_folder_zipped_as_is_loads;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::Write,
    path::{Path, PathBuf},
};

// Third Party Imports
use dokime::config::organization::policy;
use zip::write::SimpleFileOptions;

// Local Imports
use implementation::testing::{inspect_boot as boot, inspect_pack as load, replace_pack, stage_pack as stage, validate_archive};

/* === Constants === */

const MANIFEST: &str = "organization.toml";
const ACTIVE: &str = "active";
const PENDING: &str = "pending";

const LOGO: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg'/>";
const MANIFEST_TEXT: &str = "[app]\nlabel = \"Stokker Technologies\"\n[app.logos.transparent]\non_light = \"logos/mark.svg\"\n";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, contents) in entries {
        writer.start_file(*name, SimpleFileOptions::default()).unwrap();
        writer.write_all(contents).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn organization_folder() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("nextest sets the manifest directory")).join("../../../organization")
}

fn zip_folder(root: &Path, prefix: &str) -> Vec<u8> {
    fn walk(root: &Path, dir: &Path, prefix: &str, writer: &mut zip::ZipWriter<std::io::Cursor<Vec<u8>>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let name = format!("{prefix}{}", path.strip_prefix(root).unwrap().to_str().unwrap().replace('\\', "/"));
            if path.is_dir() {
                walk(root, &path, prefix, writer);
            } else {
                writer.start_file(name, SimpleFileOptions::default()).unwrap();
                writer.write_all(&std::fs::read(&path).unwrap()).unwrap();
            }
        }
    }
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    walk(root, root, prefix, &mut writer);
    writer.finish().unwrap().into_inner()
}

/* === Tests === */
