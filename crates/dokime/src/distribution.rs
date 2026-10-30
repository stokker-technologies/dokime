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
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
};

// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use crate::domain::model::plugins::{
    release::{self, Member, Plugin},
    wire::INSTALL_ARCHIVE_MAX_BYTES,
};

/* === Constants === */

pub const ARCHIVE_MAX_BYTES: usize = 2 * 1024 * 1024 * 1024;
pub const EXPANDED_MAX_BYTES: u64 = 4 * 1024 * 1024 * 1024;
pub const ARCHIVE_MAX_ENTRIES: usize = 10_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn verify(bytes: &[u8], expected: &str) -> Result<(), String> {
    if !release::sha256(expected) || !digest(bytes).eq_ignore_ascii_case(expected) {
        return Err("Release SHA-256 mismatch".into());
    }
    Ok(())
}

pub fn checksums(bytes: &[u8]) -> Result<BTreeMap<String, String>, String> {
    let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    let mut sums = BTreeMap::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let (hash, name) = line.split_once("  ").or_else(|| line.split_once(" *")).ok_or("Invalid SHA256SUMS line")?;
        if !release::sha256(hash) || !release::filename(name) || sums.insert(name.into(), hash.into()).is_some() {
            return Err("Invalid or duplicate SHA256SUMS entry".into());
        }
    }
    Ok(sums)
}

pub fn plugin(bytes: &[u8], member: &Member, selected: &Plugin) -> Result<Vec<u8>, String> {
    let bytes = member_bytes(bytes, &member.archive, &member.member)?;
    verify(&bytes, &member.sha256)?;
    let bundle = harmos_packaging::bundle::Bundle::inspect(&bytes, harmos_packaging::bundle::Limits::default())
        .map_err(|error| error.to_string())?;
    let identity = &bundle.manifest().identity;
    if identity.name != selected.name || identity.version.to_string() != selected.version {
        return Err("Plugin bundle identity does not match the selected release".into());
    }
    Ok(bytes)
}

pub fn member_bytes(bytes: &[u8], archive_name: &str, wanted: &str) -> Result<Vec<u8>, String> {
    if bytes.len() > ARCHIVE_MAX_BYTES || !release::filename(wanted) {
        return Err("Plugin archive or member exceeds its limits".into());
    }
    if archive_name.ends_with(".zip") {
        return zip_member(bytes, wanted);
    }
    if !archive_name.ends_with(".tar.gz") {
        return Err("Unsupported plugin archive format".into());
    }
    let decoder = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder.take(EXPANDED_MAX_BYTES + 1));
    let mut found = None;
    let entries = archive.entries().map_err(|error| error.to_string())?;
    let mut expanded = 0u64;
    for (index, entry) in entries.enumerate() {
        if index >= ARCHIVE_MAX_ENTRIES {
            return Err("Plugin archive has too many entries".into());
        }
        let mut entry = entry.map_err(|error| error.to_string())?;
        expanded = expanded.saturating_add(entry.size());
        if expanded > EXPANDED_MAX_BYTES {
            return Err("Expanded plugin archive exceeds its limit".into());
        }
        let path = entry.path().map_err(|error| error.to_string())?;
        if path.as_os_str() != wanted {
            continue;
        }
        if found.is_some() || !entry.header().entry_type().is_file() {
            return Err("Plugin archive member must be a unique regular file".into());
        }
        found = Some(read_member(&mut entry)?);
    }
    found.ok_or_else(|| "Plugin archive member is missing".into())
}

pub fn member_names(bytes: &[u8], archive_name: &str) -> Result<Vec<String>, String> {
    if bytes.len() > ARCHIVE_MAX_BYTES {
        return Err("Plugin archive exceeds its byte limit".into());
    }
    let names = if archive_name.ends_with(".zip") {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|error| error.to_string())?;
        if archive.has_duplicate_names() || archive.len() > ARCHIVE_MAX_ENTRIES {
            return Err("Plugin archive has too many entries".into());
        }
        let mut names = Vec::new();
        for index in 0..archive.len() {
            let entry = archive.by_index(index).map_err(|error| error.to_string())?;
            names.push(entry.name().map_err(|error| error.to_string())?.into_owned());
        }
        names
    } else if archive_name.ends_with(".tar.gz") {
        let decoder = flate2::read::GzDecoder::new(bytes);
        let mut archive = tar::Archive::new(decoder.take(EXPANDED_MAX_BYTES + 1));
        let mut names = Vec::new();
        let entries = archive.entries().map_err(|error| error.to_string())?;
        let mut expanded = 0u64;
        for entry in entries {
            if names.len() >= ARCHIVE_MAX_ENTRIES {
                return Err("Plugin archive has too many entries".into());
            }
            let entry = entry.map_err(|error| error.to_string())?;
            expanded = expanded.saturating_add(entry.size());
            if expanded > EXPANDED_MAX_BYTES {
                return Err("Expanded plugin archive exceeds its limit".into());
            }
            names.push(entry.path().map_err(|error| error.to_string())?.to_string_lossy().into_owned());
        }
        names
    } else {
        return Err("Unsupported plugin archive format".into());
    };
    let names: Vec<_> = names.into_iter().filter(|name| name.ends_with(".dokime")).collect();
    if names.is_empty() || names.iter().any(|name| !release::filename(name)) {
        return Err("Plugin archive requires root-level .dokime members".into());
    }
    Ok(names)
}

/* === Private Functions === */

fn zip_member(bytes: &[u8], wanted: &str) -> Result<Vec<u8>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|error| error.to_string())?;
    if archive.has_duplicate_names() || archive.len() > ARCHIVE_MAX_ENTRIES {
        return Err("Plugin archive has too many entries".into());
    }
    let mut selected = None;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let name = entry.name().map_err(|error| error.to_string())?;
        if name != wanted {
            continue;
        }
        let mode = entry.unix_mode().unwrap_or(0o100644) & 0o170000;
        if selected.is_some() || entry.is_dir() || mode != 0o100000 {
            return Err("Plugin archive member must be a unique regular file".into());
        }
        if entry.size() > INSTALL_ARCHIVE_MAX_BYTES {
            return Err("Plugin archive member exceeds its byte limit".into());
        }
        selected = Some(index);
    }
    let index = selected.ok_or("Plugin archive member is missing")?;
    let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
    read_member(&mut entry)
}

fn read_member(reader: &mut impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader.take(INSTALL_ARCHIVE_MAX_BYTES + 1).read_to_end(&mut bytes).map_err(|error| error.to_string())?;
    if bytes.len() as u64 > INSTALL_ARCHIVE_MAX_BYTES {
        return Err("Plugin archive member exceeds its byte limit".into());
    }
    Ok(bytes)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            archive.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
            archive.write_all(bytes).unwrap();
        }
        archive.finish().unwrap().into_inner()
    }

    fn tar_archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut archive = tar::Builder::new(encoder);
        for (name, bytes) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            archive.append_data(&mut header, name, *bytes).unwrap();
        }
        archive.into_inner().unwrap().finish().unwrap()
    }

    #[test]
    fn release_digest() {
        assert_eq!(digest(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn release_extraction() {
        let bytes = archive(&[("selected.dokime", b"selected"), ("../unselected", b"ignored")]);
        assert_eq!(member_bytes(&bytes, "sidecars.zip", "selected.dokime").unwrap(), b"selected");
        assert!(member_bytes(&bytes, "sidecars.zip", "../unselected").is_err());
        assert!(member_bytes(&bytes, "sidecars.zip", "missing.dokime").is_err());
        assert!(verify(&bytes, &"0".repeat(64)).is_err());
        verify(&bytes, &digest(&bytes)).unwrap();

        let mut duplicate_zip = archive(&[("selected.dokime", b"first"), ("replaced.dokime", b"second")]);
        for offset in 0..duplicate_zip.len() - 15 {
            if duplicate_zip[offset..].starts_with(b"replaced.dokime") {
                duplicate_zip[offset..offset + 15].copy_from_slice(b"selected.dokime");
            }
        }
        assert!(member_bytes(&duplicate_zip, "sidecars.zip", "selected.dokime").is_err());

        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        zip.add_symlink("selected.dokime", "outside", zip::write::SimpleFileOptions::default()).unwrap();
        let link = zip.finish().unwrap().into_inner();
        assert!(member_bytes(&link, "sidecars.zip", "selected.dokime").is_err());

        let bytes = tar_archive(&[("selected.dokime", b"selected"), ("unselected.dokime", b"ignored")]);
        assert_eq!(member_bytes(&bytes, "sidecars.tar.gz", "selected.dokime").unwrap(), b"selected");
        assert_eq!(member_names(&bytes, "sidecars.tar.gz").unwrap(), ["selected.dokime", "unselected.dokime"]);
        let duplicate = tar_archive(&[("selected.dokime", b"first"), ("selected.dokime", b"second")]);
        assert!(member_bytes(&duplicate, "sidecars.tar.gz", "selected.dokime").is_err());
    }
}
