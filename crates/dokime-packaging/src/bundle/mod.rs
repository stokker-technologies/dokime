/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bundle Any Native Plugin Executable Without Compiling

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    env::consts::{ARCH, OS},
    fs::File,
    io::Read,
    iter,
    path::{Path, PathBuf},
};

// Third Party Imports
use harmos_runtime::bundle::{Bundle, Identity, Limits, Manifest};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tar::{Builder, EntryType, Header};

// Local Imports
use crate::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, PartialEq, Eq)]
pub enum DevelopmentInput {
    Bundle,
    PythonProject,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn development_input(path: &Path) -> Result<DevelopmentInput> {
    if path.is_dir() && path.join("pyproject.toml").is_file() && path.join("plugin.py").is_file() {
        return Ok(DevelopmentInput::PythonProject);
    }
    if path.is_file() && inspect(path).is_ok() {
        return Ok(DevelopmentInput::Bundle);
    }
    Err("Develop Plugin: Supply a Bundle File or a Python Folder with pyproject.toml and plugin.py; Bundle Other Plugins First".into())
}

pub fn native_target() -> &'static str {
    match (ARCH, OS) {
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        _ => "unsupported",
    }
}

pub fn create(executable: &Path, assets: &[PathBuf], identity: Identity, description: Option<&[u8]>) -> Result<Vec<u8>> {
    let limits = Limits::default();
    if native_target() == "unsupported" {
        return Err("Create Plugin Bundle: Select a Supported Native Target".into());
    }
    let name = executable.file_name().and_then(|name| name.to_str()).ok_or("Create Plugin Bundle: Supply a UTF-8 Executable Filename")?;
    let entrypoint = format!("bin/{name}");
    if assets.len() + 2 + usize::from(description.is_some()) > limits.files {
        return Err("Create Plugin Bundle: Reduce Its File Inventory".into());
    }
    let mut contents = BTreeMap::from([(entrypoint.clone(), read(executable, limits.file_bytes as u64)?)]);
    let mut total = contents.values().map(Vec::len).sum::<usize>();
    for asset in assets {
        let name = asset.file_name().and_then(|name| name.to_str()).ok_or("Create Plugin Bundle: Supply a UTF-8 Asset Filename")?;
        let remaining = limits.total_file_bytes.saturating_sub(total);
        let bytes = read(asset, limits.file_bytes.min(remaining) as u64)?;
        total += bytes.len();
        if contents.insert(format!("bin/{name}"), bytes).is_some() {
            return Err("Create Plugin Bundle: Give Each Asset a Distinct Filename".into());
        }
    }
    contents.insert(
        "dokime.json".into(),
        serde_json::to_vec(&serde_json::json!({"schema":1,"requires_dokime":format!("={}",env!("CARGO_PKG_VERSION"))}))?,
    );
    if let Some(description) = description {
        contents.insert("description.pb".into(), description.to_vec());
    }
    if contents.len() > limits.files || contents.values().map(Vec::len).sum::<usize>() > limits.total_file_bytes {
        return Err("Create Plugin Bundle: Reduce Its Files to the Bundle Bounds".into());
    }
    let files = contents.iter().map(|(path, bytes)| {
        let mut value = serde_json::json!({"path":path,"size":bytes.len(),"sha256":Sha256::digest(bytes).iter().map(|byte|format!("{byte:02x}")).collect::<String>()});
        if path == &entrypoint { value["executable"] = serde_json::json!({"facility":"sidecar","target":native_target(),"registrations":[]}); }
        value
    }).collect::<Vec<_>>();
    let manifest: Manifest = serde_json::from_value(serde_json::json!({"schema":1,"identity":identity,"files":files}))?;
    let mut archive = Builder::new(Vec::new());
    let manifest = serde_json::to_vec(&manifest)?;
    for (path, bytes) in iter::once(("manifest.json".to_owned(), manifest)).chain(contents) {
        let mut header = Header::new_ustar();
        header.set_entry_type(EntryType::Regular);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_size(bytes.len() as u64);
        header.set_mode(if path == entrypoint { 0o755 } else { 0o644 });
        header.set_cksum();
        archive.append_data(&mut header, path, bytes.as_slice())?;
    }
    let bytes = archive.into_inner()?;
    Bundle::inspect(&bytes, limits)?;
    Ok(bytes)
}

pub fn inspect(path: &Path) -> Result<Value> {
    let limits = Limits::default();
    let bytes = read(path, limits.archive_bytes as u64)?;
    let bundle = Bundle::inspect(&bytes, limits)?;
    Ok(serde_json::to_value(bundle.manifest())?)
}

/* === Private Functions === */

fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = File::open(path)?;
    if file.metadata()?.len() > limit {
        return Err("Read Plugin Input: File Exceeds the Byte Bound".into());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("Read Plugin Input: File Grew Beyond the Byte Bound".into());
    }
    Ok(bytes)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::inspect;
    use harmos_runtime::bundle::Manifest;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::error::Error;
    use std::io::ErrorKind;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::{
        env,
        fs::{self, OpenOptions},
        iter, process,
    };
    use tar::{Builder, EntryType, Header};

    fn digest(bytes: &[u8]) -> String {
        Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
    }

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Bundle(PathBuf);

    impl Bundle {
        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Bundle {
        fn drop(&mut self) {
            fs::remove_file(&self.0).expect("Remove Owned Bundle Fixture");
        }
    }

    fn bundle(files: &[(&str, &[u8])], manifest: &Value) -> Result<Bundle, Box<dyn Error + Send + Sync>> {
        let (fixture, mut file) = loop {
            let next = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = env::temp_dir().join(format!("dokime-bundle-{}-{next}.tar", process::id()));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => break (Bundle(path), file),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        };
        let mut archive = Builder::new(&mut file);
        let manifest: Manifest = serde_json::from_value(manifest.clone())?;
        let bytes = serde_json::to_vec(&manifest)?;
        for (name, bytes) in iter::once(("manifest.json", bytes.as_slice())).chain(files.iter().copied()) {
            let mut header = Header::new_ustar();
            header.set_entry_type(EntryType::Regular);
            header.set_uid(0);
            header.set_gid(0);
            header.set_mtime(0);
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            archive.append_data(&mut header, name, bytes)?;
        }
        archive.finish()?;
        drop(archive);
        file.flush()?;
        drop(file);
        Ok(fixture)
    }

    #[test]
    fn inspect_bundle_preserves_inventory_and_refuses_tampered_content() -> Result<(), Box<dyn Error + Send + Sync>> {
        // Inspect the Declared Content Without Extracting It
        let bytes = b"plugin executable";
        let manifest = serde_json::json!({"schema": 1, "identity": {"name": "example", "version": "1.0.0", "description": "", "package": {"authors": [], "license": null, "repository": null, "homepage": null, "documentation": null, "keywords": [], "categories": []}},
            "files": [{"path": "bin/plugin", "size": bytes.len(), "sha256": digest(bytes)}]});
        let file = bundle(&[("bin/plugin", bytes)], &manifest)?;
        assert_eq!(inspect(file.path())?, manifest);

        // Refuse Different Bytes at the Same Declared Path
        let file = bundle(&[("bin/plugin", b"changed executable")], &manifest)?;
        assert!(inspect(file.path()).is_err());

        // Refuse an Unlisted File While Preserving the Input Archive
        let file = bundle(&[("bin/plugin", bytes), ("extra", b"unlisted")], &manifest)?;
        let before = fs::read(file.path())?;
        assert!(inspect(file.path()).is_err());
        assert_eq!(fs::read(file.path())?, before);
        Ok(())
    }

    #[test]
    fn inspect_bundle_refuses_duplicate_members_and_missing_inventory() -> Result<(), Box<dyn Error + Send + Sync>> {
        // Refuse Ambiguous Duplicate Archive Members
        let bytes = b"plugin executable";
        let manifest = serde_json::json!({"schema": 1, "identity": {"name": "example", "version": "1.0.0", "description": ""},
            "files": [{"path": "bin/plugin", "size": bytes.len(), "sha256": digest(bytes)}]});
        let file = bundle(&[("bin/plugin", bytes), ("bin/plugin", bytes)], &manifest)?;
        assert!(inspect(file.path()).is_err());

        // Refuse an Inventory That Names Absent Content
        let file = bundle(&[], &manifest)?;
        assert!(inspect(file.path()).is_err());
        Ok(())
    }

    #[test]
    fn development_paths_require_a_bundle_or_python_project() -> Result<(), Box<dyn Error + Send + Sync>> {
        let root = tempfile::tempdir()?;
        let executable = root.path().join("native-plugin");
        fs::write(&executable, b"native executable")?;
        assert!(super::development_input(&executable).is_err());
        assert!(super::development_input(root.path()).is_err());
        fs::write(root.path().join("pyproject.toml"), "[project]\nname='fixture'\nversion='0.1.0'\n")?;
        assert!(super::development_input(root.path()).is_err());
        fs::write(root.path().join("plugin.py"), "print('http://127.0.0.1:1234')\n")?;
        assert_eq!(super::development_input(root.path())?, super::DevelopmentInput::PythonProject);
        let bytes = b"native executable";
        let manifest = serde_json::json!({"schema": 1, "identity": {"name": "example", "version": "1.0.0", "description": ""},
            "files": [{"path": "bin/plugin", "size": bytes.len(), "sha256": digest(bytes)}]});
        let file = bundle(&[("bin/plugin", bytes)], &manifest)?;
        assert_eq!(super::development_input(file.path())?, super::DevelopmentInput::Bundle);
        Ok(())
    }
}
