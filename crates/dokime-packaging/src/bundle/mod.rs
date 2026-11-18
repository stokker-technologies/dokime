/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Package and Inspect Plugin Bundles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    env::consts::{ARCH, OS},
    fs,
    io::Read,
    iter,
    path::{Path, PathBuf},
};

// Third Party Imports
use harmos_packaging::bundle::{self, Source};
use harmos_runtime::bundle::{Bundle, Executable, Facility, File, Identity, Limits, Manifest};
use ignore::WalkBuilder;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tar::{Builder, EntryType, Header};

// Local Imports
use crate::{Result, project::PluginProject};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, PartialEq, Eq)]
pub enum DevelopmentInput {
    Bundle,
    Project,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn development_input(path: &Path) -> Result<DevelopmentInput> {
    if path.is_dir() && path.join("dokime.toml").is_file() {
        PluginProject::read(path)?;
        return Ok(DevelopmentInput::Project);
    }
    if path.is_file() && inspect(path).is_ok() {
        return Ok(DevelopmentInput::Bundle);
    }
    Err("Develop Plugin: Supply a Bundle File or a Plugin Folder Containing dokime.toml".into())
}

pub fn create_project(project: &PluginProject, identity: Identity, description: Option<&[u8]>) -> Result<Vec<u8>> {
    project.validate_inputs()?;
    if project.manifest.executable.is_some() {
        let executable = project.executable()?;
        let assets = project.manifest.assets.iter().map(|asset| project.path(asset)).collect::<Result<Vec<_>>>()?;
        return create(&executable, &assets, identity, description);
    }
    let launch = project.manifest.launch.as_deref().ok_or("Create Plugin Bundle: Set launch in dokime.toml")?;
    let limits = Limits::default();
    let mut contents = BTreeMap::new();
    let mut walker = WalkBuilder::new(&project.root);
    walker
        .hidden(false)
        .parents(false)
        .ignore(false)
        .git_global(false)
        .git_exclude(false)
        .require_git(false)
        .filter_entry(|entry| entry.file_name() != ".git");
    for entry in walker.build() {
        let entry = entry?;
        if entry.file_type().is_some_and(|kind| kind.is_dir()) {
            continue;
        }
        let relative = entry.path().strip_prefix(&project.root)?;
        let name = relative.to_str().ok_or("Create Plugin Bundle: Use UTF-8 File Paths")?.replace('\\', "/");
        if matches!(name.as_str(), "manifest.json" | "dokime.json" | "description.pb") {
            return Err(format!("Create Plugin Bundle: Rename Reserved Root File {name}").into());
        }
        insert(&mut contents, name, entry.path(), limits)?;
    }
    metadata(&mut contents, description)?;
    let sources = contents.iter().map(|(path, bytes)| Source { path, bytes, executable: None }).collect::<Vec<_>>();
    let bundle = bundle::build(identity, &sources, Some(launch), limits)?;
    Ok(bundle.bytes().to_vec())
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
    let mut contents = BTreeMap::new();
    insert(&mut contents, entrypoint.clone(), executable, limits)?;
    for asset in assets {
        let name = asset.file_name().and_then(|name| name.to_str()).ok_or("Create Plugin Bundle: Supply a UTF-8 Asset Filename")?;
        let destination = if Some(asset.as_path()) == executable.parent() { "bin".into() } else { format!("bin/{name}") };
        let mut pending = vec![(asset.clone(), destination)];
        while let Some((path, name)) = pending.pop() {
            let kind = fs::symlink_metadata(&path)?.file_type();
            if kind.is_dir() {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    let filename = entry.file_name().into_string().map_err(|_| "Create Plugin Bundle: Use UTF-8 Asset Filenames")?;
                    pending.push((entry.path(), format!("{name}/{filename}")));
                }
                continue;
            }
            if path == executable {
                continue;
            }
            insert(&mut contents, name, &path, limits)?;
        }
    }
    metadata(&mut contents, description)?;
    if contents.len() > limits.files || contents.values().map(Vec::len).sum::<usize>() > limits.total_file_bytes {
        return Err("Create Plugin Bundle: Reduce Its Files to the Bundle Bounds".into());
    }
    let files = contents
        .iter()
        .map(|(path, bytes)| File {
            path: path.clone(),
            size: bytes.len() as u64,
            sha256: Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect(),
            executable: (path == &entrypoint).then(|| Executable {
                facility: Facility::Sidecar,
                target: native_target().into(),
                registrations: Vec::new(),
            }),
        })
        .collect();
    let manifest = Manifest { schema: 1, identity, files, launch: None };
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

fn insert(contents: &mut BTreeMap<String, Vec<u8>>, name: String, path: &Path, limits: Limits) -> Result<()> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(format!("Create Plugin Bundle: Replace {} with a Regular File; Symbolic Links Are Not Packed", path.display()).into());
    }
    if contents.len() >= limits.files {
        return Err("Create Plugin Bundle: Reduce Its File Inventory".into());
    }
    let total = contents.values().map(Vec::len).sum::<usize>();
    let remaining = limits.total_file_bytes.saturating_sub(total);
    let bytes = read(path, limits.file_bytes.min(remaining) as u64)?;
    if contents.insert(name, bytes).is_some() {
        return Err("Create Plugin Bundle: Give Each File a Distinct Bundle Path".into());
    }
    Ok(())
}

fn metadata(contents: &mut BTreeMap<String, Vec<u8>>, description: Option<&[u8]>) -> Result<()> {
    contents.insert(
        "dokime.json".into(),
        serde_json::to_vec(&serde_json::json!({"schema":1,"requires_dokime":format!("={}",env!("CARGO_PKG_VERSION"))}))?,
    );
    if let Some(description) = description {
        contents.insert("description.pb".into(), description.to_vec());
    }
    Ok(())
}

fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = fs::File::open(path)?;
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
    use harmos_runtime::bundle::{self, Identity, Limits, Manifest};
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
    fn development_paths_require_a_bundle_or_manifest_project() -> Result<(), Box<dyn Error + Send + Sync>> {
        let root = tempfile::tempdir()?;
        let executable = root.path().join("native-plugin");
        fs::write(&executable, b"native executable")?;
        assert!(super::development_input(&executable).is_err());
        assert!(super::development_input(root.path()).is_err());
        fs::write(root.path().join("pyproject.toml"), "[project]\nname='fixture'\nversion='0.1.0'\n")?;
        assert!(super::development_input(root.path()).is_err());
        fs::write(root.path().join("dokime.toml"), "executable = 'target/debug/plugin'\n")?;
        fs::remove_file(root.path().join("pyproject.toml"))?;
        assert_eq!(super::development_input(root.path())?, super::DevelopmentInput::Project);
        fs::write(root.path().join("dokime.toml"), "launch = ['fixture', 'entry']\n")?;
        assert_eq!(super::development_input(root.path())?, super::DevelopmentInput::Project);
        let bytes = b"native executable";
        let manifest = serde_json::json!({"schema": 1, "identity": {"name": "example", "version": "1.0.0", "description": ""},
            "files": [{"path": "bin/plugin", "size": bytes.len(), "sha256": digest(bytes)}]});
        let file = bundle(&[("bin/plugin", bytes)], &manifest)?;
        assert_eq!(super::development_input(file.path())?, super::DevelopmentInput::Bundle);
        Ok(())
    }

    #[test]
    fn launch_folder_round_trip_runs_the_native_interpreter_in_its_root() -> crate::Result<()> {
        // The Reexecuted Native Test Program Interprets the Packaged Payload
        if env::var_os("DOKIME_LAUNCH_FIXTURE").is_some() {
            assert_eq!(fs::read("payload.bin")?, [2, 3, 5, 7]);
            assert!(Path::new("dokime.toml").is_file());
            assert!(!Path::new("ignored.bin").exists());
            return Ok(());
        }

        // Pack a Project Without Network or an Installed Interpreter
        let root = tempfile::tempdir()?;
        let program = format!("interpreter{}", env::consts::EXE_SUFFIX);
        fs::copy(env::current_exe()?, root.path().join(&program))?;
        let test = "bundle::tests::launch_folder_round_trip_runs_the_native_interpreter_in_its_root";
        fs::write(root.path().join("dokime.toml"), format!("launch = ['./{program}', '--exact', '{test}', '--nocapture']\n"))?;
        fs::write(root.path().join("payload.bin"), [2, 3, 5, 7])?;
        fs::write(root.path().join(".gitignore"), "ignored.bin\n")?;
        fs::write(root.path().join("ignored.bin"), [11])?;
        fs::create_dir(root.path().join(".git"))?;
        fs::write(root.path().join(".git/config"), b"private repository data")?;
        let project = crate::project::PluginProject::read(root.path())?;
        let identity = Identity {
            name: "launch-fixture".into(),
            version: "1.0.0".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = super::create_project(&project, identity, Some(b"description"))?;
        let bundle = bundle::Bundle::inspect(&bytes, Limits::default())?;
        assert_eq!(bundle.manifest().launch, project.manifest.launch);
        assert!(bundle.manifest().files.iter().all(|file| file.executable.is_none()));
        assert!(bundle.file("ignored.bin").is_none());
        assert!(bundle.file(".git/config").is_none());
        assert_eq!(bundle.file("description.pb"), Some(b"description".as_slice()));

        // Extract and Run the Declared Command Exactly From the Bundle Root
        let parent = tempfile::tempdir()?;
        let extracted = bundle.extract(parent.path())?;
        let launch = bundle.manifest().launch.as_ref().unwrap();
        #[allow(clippy::disallowed_methods, reason = "the packaging test cannot depend on its runtime consumer's process constructor")]
        let output = process::Command::new(extracted.path().join(&launch[0]))
            .args(&launch[1..])
            .current_dir(extracted.path())
            .env("DOKIME_LAUNCH_FIXTURE", "1")
            .output()?;
        assert!(
            output.status.success(),
            "stdout: {}; stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }

    #[test]
    fn native_assets_include_nested_folders_and_runtime_output_siblings() -> crate::Result<()> {
        let root = tempfile::tempdir()?;
        let executable = root.path().join("plugin");
        fs::write(&executable, [2, 3])?;
        fs::write(root.path().join("runtime.dll"), [5, 7])?;
        fs::create_dir(root.path().join("assets"))?;
        fs::write(root.path().join("assets/data.bin"), [11, 13])?;
        let identity = Identity {
            name: "native-fixture".into(),
            version: "1.0.0".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = super::create(&executable, &[root.path().to_path_buf()], identity, None)?;
        let bundle = bundle::Bundle::inspect(&bytes, Limits::default())?;
        assert_eq!(bundle.file("bin/plugin"), Some([2, 3].as_slice()));
        assert_eq!(bundle.file("bin/runtime.dll"), Some([5, 7].as_slice()));
        assert_eq!(bundle.file("bin/assets/data.bin"), Some([11, 13].as_slice()));
        Ok(())
    }
}
