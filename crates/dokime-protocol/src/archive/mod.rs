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
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
// Third Party Imports
use prost::Message;
use sha2::{Digest, Sha256};
// Local Imports
use dokime_protocol::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[tracing::instrument(level = "debug", skip_all)]
pub(super) fn run() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut args = std::env::args().skip(1);
    let destination = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/protocol/dokime-protocol.tar"));
    let revision = args
        .next()
        .map(Ok)
        .unwrap_or_else(|| source_revision(&root))?;
    if args.next().is_some() || revision.trim().is_empty() {
        return Err("usage: export_protocol [ARCHIVE.tar [SOURCE_REVISION]]".into());
    }
    let files = files(&root, &revision)?;
    let parent = destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    {
        let mut archive = tar::Builder::new(&mut temporary);
        for (name, data) in &files {
            let mut header = tar::Header::new_ustar();
            header.set_path(name)?;
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_uid(0);
            header.set_gid(0);
            header.set_mtime(0);
            header.set_cksum();
            archive.append(&header, data.as_slice())?;
        }
        archive.finish()?;
    }
    temporary.flush()?;
    let hash = digest(&fs::read(temporary.path())?);
    let name = destination
        .file_name()
        .ok_or("archive needs a file name")?
        .to_string_lossy();
    let checksum_path = destination.with_extension("tar.sha256");
    let mut checksum = tempfile::NamedTempFile::new_in(parent)?;
    writeln!(checksum, "{hash}  {name}")?;
    checksum.flush()?;
    // The archive rename is atomic. Publish its checksum second; any failure
    // leaves either the previous archive or the new archive without a checksum,
    // never a successful-looking stale checksum for newly published bytes.
    match fs::remove_file(&checksum_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    temporary.persist(&destination)?;
    checksum.persist(&checksum_path)?;
    println!("{} ({hash})", destination.display());
    Ok(())
}

/* === Private Functions === */
fn source_revision(root: &Path) -> Result<String> {
    if let Ok(value) = std::env::var("DOKIME_SOURCE_REVISION") {
        return Ok(value);
    }
    for (program, args) in [
        (
            "jj",
            vec!["log", "-r", "@", "--no-graph", "-T", "commit_id"],
        ),
        ("git", vec!["rev-parse", "HEAD"]),
    ] {
        if let Ok(output) = Command::new(program).args(args).current_dir(root).output()
            && output.status.success()
        {
            return Ok(String::from_utf8(output.stdout)?.trim().to_owned());
        }
    }
    Err("pass SOURCE_REVISION or set DOKIME_SOURCE_REVISION outside a versioned checkout".into())
}

fn files(root: &Path, revision: &str) -> Result<BTreeMap<String, Vec<u8>>> {
    let source = root.join("crates/dokime-protocol/proto");
    let includes = protoc_bin_vendored::include_path()?;
    let descriptor = prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?;
    let mut files = BTreeMap::new();
    let mut canonical = Vec::new();
    collect(&source.join("dokime"), &source, &mut canonical)?;
    canonical.retain(|name| name.ends_with(".proto"));
    canonical.sort();
    for name in &canonical {
        if !descriptor
            .file
            .iter()
            .any(|file| file.name.as_ref() == Some(name))
        {
            return Err(format!("canonical source {name} is missing from the protocol crate descriptor; update build.rs").into());
        }
    }
    for file in &descriptor.file {
        let name = file.name.as_deref().ok_or("descriptor file has no name")?;
        if Path::new(name)
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err("descriptor has an unsafe source path".into());
        }
        for dependency in &file.dependency {
            if !descriptor
                .file
                .iter()
                .any(|file| file.name.as_ref() == Some(dependency))
            {
                return Err(format!("descriptor is missing import {dependency}").into());
            }
        }
        let path = if source.join(name).is_file() {
            source.join(name)
        } else {
            includes.join(name)
        };
        files.insert(format!("proto/{name}"), fs::read(path)?);
    }
    validate_descriptor(&files, &canonical, descriptor)?;
    files.insert(
        "dokime-v1.bin".into(),
        dokime_protocol::FILE_DESCRIPTOR_SET.to_vec(),
    );
    let compiler = Command::new(protoc_bin_vendored::protoc_bin_path()?)
        .arg("--version")
        .output()?;
    if !compiler.status.success() {
        return Err("protoc version query failed".into());
    }
    let manifest = serde_json::json!({
        "format_version": 1,
        "product_version": env!("CARGO_PKG_VERSION"),
        "source_revision": revision,
        "api": {"package": dokime_protocol::API_PACKAGE, "version": dokime_protocol::API_VERSION, "descriptor_sha256": digest(dokime_protocol::FILE_DESCRIPTOR_SET)},
        "compiler": {"protoc": String::from_utf8(compiler.stdout)?.trim(), "protoc_bin_vendored": "3.2.0", "prost_build": "0.14.4", "tonic_prost_build": "0.14.6"},
        "canonical_sources": canonical,
    });
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    files.insert("manifest.json".into(), bytes);
    let checksums = files
        .iter()
        .map(|(name, bytes)| format!("{}  {name}\n", digest(bytes)))
        .collect::<String>();
    files.insert("SHA256SUMS".into(), checksums.into_bytes());
    Ok(files)
}

fn validate_descriptor(
    files: &BTreeMap<String, Vec<u8>>,
    canonical: &[String],
    mut expected: prost_types::FileDescriptorSet,
) -> Result<()> {
    let snapshot = tempfile::tempdir()?;
    for (name, bytes) in files {
        let path = snapshot.path().join(name);
        fs::create_dir_all(path.parent().ok_or("source has no parent")?)?;
        fs::write(path, bytes)?;
    }
    let output = snapshot.path().join("descriptor.bin");
    let result = Command::new(protoc_bin_vendored::protoc_bin_path()?)
        .arg(format!(
            "--proto_path={}",
            snapshot.path().join("proto").display()
        ))
        .arg(format!("--descriptor_set_out={}", output.display()))
        .arg("--include_imports")
        .args(canonical)
        .output()?;
    if !result.status.success() {
        return Err(format!(
            "archived sources do not compile: {}",
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    let mut actual = prost_types::FileDescriptorSet::decode(fs::read(output)?.as_slice())?;
    // File ordering can differ with compiler input ordering; the type contract
    // must be identical. The archive still carries the original host bytes.
    expected.file.sort_by(|a, b| a.name.cmp(&b.name));
    actual.file.sort_by(|a, b| a.name.cmp(&b.name));
    if actual != expected {
        return Err("source schema differs from the compiled descriptor; rebuild with mise run protocol:archive".into());
    }
    Ok(())
}

fn collect(directory: &Path, root: &Path, names: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect(&path, root, names)?;
        } else if entry.file_type()?.is_file() {
            names.push(
                path.strip_prefix(root)?
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

/* === Tests === */
