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
    error, fs,
    io::Read,
    path::{Path, PathBuf},
};

// Third Party Imports
use harmos_packaging::bundle::{self, Source};
use harmos_runtime::bundle::{Bundle, Executable, Facility, Identity, Limits};
use ignore::WalkBuilder;
use serde_json::Value;

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
    if path.is_file() {
        inspect(path)?;
        return Ok(DevelopmentInput::Bundle);
    }
    Err("Develop Plugin: Supply a Bundle File or a Plugin Folder Containing dokime.toml".into())
}

pub fn create_project(project: &PluginProject, identity: Identity, description: Option<&[u8]>) -> Result<Vec<u8>> {
    project.validate_inputs()?;
    project.check_compatibility(env!("CARGO_PKG_VERSION"))?;
    let (identity, resolved) = project.resolve(identity)?;
    let limits = Limits::default();
    let mut contents = BTreeMap::new();
    let executable = if project.manifest.scripts.executable.is_some() { Some(project.executable()?) } else { None };
    if executable.is_some() && native_target() == "unsupported" {
        return Err("Create Plugin Bundle: Select a Supported Native Target".into());
    }
    let entrypoint = executable
        .as_ref()
        .map(|path| {
            let name = path.file_name().and_then(|name| name.to_str()).ok_or("Create Plugin Bundle: Use a UTF-8 Executable Filename")?;
            Ok::<_, Box<dyn error::Error + Send + Sync>>(format!("bin/{name}"))
        })
        .transpose()?;
    if let (Some(executable), Some(entrypoint)) = (&executable, &entrypoint) {
        insert(&mut contents, entrypoint.clone(), executable, limits)?;
    } else {
        let mut walker = WalkBuilder::new(&project.root);
        walker
            .hidden(false)
            .parents(false)
            .ignore(false)
            .git_global(false)
            .git_exclude(false)
            .require_git(false)
            .filter_entry(|entry| !excluded(entry.file_name().to_str().unwrap_or("")));
        for entry in walker.build() {
            let entry = entry?;
            if entry.file_type().is_some_and(|kind| kind.is_dir()) {
                continue;
            }
            let relative = entry.path().strip_prefix(&project.root)?;
            project.path(relative)?;
            let name = relative.to_str().ok_or("Create Plugin Bundle: Use UTF-8 File Paths")?.replace('\\', "/");
            if matches!(name.as_str(), "manifest.json" | "dokime.json" | "description.pb") {
                return Err(format!("Create Plugin Bundle: Rename Reserved Root File {name}").into());
            }
            insert(&mut contents, name, entry.path(), limits)?;
        }
        // Metadata and Lock Inputs Are Required Even When Ignored by Git.
        for path in project.source_metadata()? {
            let relative = path.strip_prefix(&project.root)?;
            if relative.components().any(|part| excluded(part.as_os_str().to_str().unwrap_or(""))) {
                continue;
            }
            let name =
                path.strip_prefix(&project.root)?.to_str().ok_or("Create Plugin Bundle: Use UTF-8 Metadata Paths")?.replace('\\', "/");
            if !contents.contains_key(&name) {
                insert(&mut contents, name, &path, limits)?;
            }
        }
        // Required Inputs Must Not Disappear Because of Ignore or Exclusion Rules.
        for (script, arguments) in [("launch", &project.manifest.scripts.launch), ("build", &project.manifest.scripts.build)] {
            for argument in arguments.as_deref().unwrap_or_default() {
                let path = project.root.join(argument);
                let local_input = argument.contains(['/', '\\']) || argument.ends_with(".py") || argument.ends_with(".dll");
                if local_input && !argument.starts_with('-') && !path.is_file() {
                    return Err(
                        format!("Create Plugin Bundle: Build Required scripts.{script} Input {argument} or Correct Its Path").into()
                    );
                }
                if path.is_file() {
                    let path = project.path(Path::new(argument))?;
                    let name = path
                        .strip_prefix(&project.root)?
                        .to_str()
                        .ok_or("Create Plugin Bundle: Use UTF-8 Launch Paths")?
                        .replace('\\', "/");
                    let excluded = Path::new(&name).components().any(|part| excluded(part.as_os_str().to_str().unwrap_or("")));
                    if excluded {
                        return Err(format!(
                            "Create Plugin Bundle: Move Required scripts.{script} Input {} Outside Development and Private Input Paths",
                            path.display()
                        )
                        .into());
                    }
                    if !contents.contains_key(&name) && !name.starts_with("assets/") {
                        return Err(format!("Create Plugin Bundle: Unignore Required scripts.{script} Input {}", path.display()).into());
                    }
                }
            }
            let module = arguments.as_ref().and_then(|command| command.windows(2).find(|pair| pair[0] == "-m").map(|pair| &pair[1]));
            if let Some(module) = module {
                let module = module.replace('.', "/");
                let candidates = [
                    format!("src/{module}/__main__.py"),
                    format!("src/{module}.py"),
                    format!("{module}/__main__.py"),
                    format!("{module}.py"),
                ];
                let ignored = candidates
                    .iter()
                    .find(|input| project.root.join(input).is_file())
                    .filter(|input| !contents.contains_key(input.as_str()));
                if let Some(input) = ignored {
                    if Path::new(input).components().any(|part| excluded(part.as_os_str().to_str().unwrap_or(""))) {
                        return Err(format!("Create Plugin Bundle: Move Required scripts.{script} Module {input} Outside Development and Private Input Paths").into());
                    }
                    return Err(format!("Create Plugin Bundle: Unignore Required scripts.{script} Module {input}").into());
                }
            }
        }
    }
    // Assets Bypass Ignore Rules but Never Path Confinement or Sensitive Inputs.
    let assets = project.path(Path::new("assets"))?;
    if assets.exists() {
        if !fs::symlink_metadata(&assets)?.file_type().is_dir() {
            return Err("Create Plugin Bundle: Replace assets With a Regular Directory Inside the Project".into());
        }
        let mut pending = vec![assets];
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(directory)? {
                let entry = entry?;
                if excluded(entry.file_name().to_str().unwrap_or("")) {
                    continue;
                }
                let relative = entry.path().strip_prefix(&project.root)?.to_path_buf();
                let path = project.path(&relative)?;
                let kind = entry.file_type()?;
                if kind.is_dir() {
                    pending.push(path);
                    continue;
                }
                let name = relative.to_str().ok_or("Create Plugin Bundle: Use UTF-8 Asset Filenames")?.replace('\\', "/");
                if !contents.contains_key(&name) {
                    insert(&mut contents, name, &path, limits)?;
                }
            }
        }
    }
    for relative in [&project.manifest.plugin.icon, &project.manifest.plugin.readme].into_iter().flatten() {
        if relative.components().any(|part| excluded(part.as_os_str().to_str().unwrap_or(""))) {
            return Err("Create Plugin Bundle: Move Referenced Metadata Outside Private Input Folders".into());
        }
        let path = project.path(relative)?;
        let name = path.strip_prefix(&project.root)?.to_str().ok_or("Create Plugin Bundle: Use UTF-8 Metadata Paths")?.replace('\\', "/");
        if !contents.contains_key(&name) {
            insert(&mut contents, name, &path, limits)?;
        }
    }
    metadata(&mut contents, description)?;
    let requirement = project.manifest.compatibility.as_ref().and_then(|value| value.dokime.as_deref());
    contents.insert(
        "dokime.json".into(),
        serde_json::to_vec(&serde_json::json!({"schema":1,"requires_dokime":requirement,"plugin":resolved}))?,
    );
    let sources = contents
        .iter()
        .map(|(path, bytes)| Source {
            path,
            bytes,
            executable: entrypoint.as_ref().filter(|entrypoint| *entrypoint == path).map(|_| Executable {
                facility: Facility::Sidecar,
                target: native_target().into(),
                registrations: Vec::new(),
            }),
        })
        .collect::<Vec<_>>();
    let bundle = bundle::build(identity, &sources, project.manifest.scripts.launch.as_deref(), limits)?;
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
    let sources = contents
        .iter()
        .map(|(path, bytes)| Source {
            path,
            bytes,
            executable: (path == &entrypoint).then(|| Executable {
                facility: Facility::Sidecar,
                target: native_target().into(),
                registrations: Vec::new(),
            }),
        })
        .collect::<Vec<_>>();
    let bundle = bundle::build(identity, &sources, None, limits)?;
    Ok(bundle.bytes().to_vec())
}

pub fn inspect(path: &Path) -> Result<Value> {
    let bundle = open(path)?;
    Ok(serde_json::to_value(bundle.manifest())?)
}

/// Read and Verify a Bundle Against the Current Product Compatibility Policy.
pub fn open(path: &Path) -> Result<Bundle> {
    let limits = Limits::default();
    let bytes = read(path, limits.archive_bytes as u64)?;
    let bundle = Bundle::inspect(&bytes, limits)?;
    if let Some(metadata) = bundle.file("dokime.json") {
        let metadata: Value = serde_json::from_slice(metadata)?;
        let requirement = match metadata.get("requires_dokime") {
            None | Some(Value::Null) => None,
            Some(Value::String(value)) => Some(value.as_str()),
            _ => return Err("Inspect Plugin Bundle: Set requires_dokime to a Version Requirement".into()),
        };
        crate::project::check_compatibility(requirement, env!("CARGO_PKG_VERSION"))?;
    }
    Ok(bundle)
}

/* === Private Functions === */

fn excluded(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    matches!(
        name.as_str(),
        ".git"
            | ".gitignore"
            | ".gitattributes"
            | ".gitmodules"
            | ".github"
            | ".gitlab"
            | ".gitlab-ci.yml"
            | ".hg"
            | ".hgignore"
            | ".hgtags"
            | ".svn"
            | "mise.toml"
            | ".mise.toml"
            | "mise.lock"
            | ".tool-versions"
            | "test"
            | "tests"
            | "__tests__"
            | ".pytest_cache"
            | ".mypy_cache"
            | ".ruff_cache"
            | ".cache"
            | ".idea"
            | ".vscode"
            | ".vs"
            | ".ds_store"
            | "thumbs.db"
            | ".venv"
            | ".env"
            | "_input"
            | "_output"
            | "__pycache__"
            | "credentials"
            | "credentials.json"
            | ".netrc"
            | ".pypirc"
            | "id_rsa"
            | "id_ed25519"
    ) || name.starts_with(".env.")
        || name.ends_with(".pair")
        || name.ends_with(".pem")
        || name.ends_with(".key")
}

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
    use super::{DevelopmentInput, inspect};
    use crate::project::PluginProject;
    use harmos_runtime::bundle::{self, Facility, Identity, Limits, Manifest};
    use ignore::WalkBuilder;
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
        fs::write(root.path().join("dokime.toml"), "[plugin]\n[scripts]\nexecutable = 'target/debug/plugin'\n")?;
        fs::remove_file(root.path().join("pyproject.toml"))?;
        assert_eq!(super::development_input(root.path())?, DevelopmentInput::Project);
        fs::write(root.path().join("dokime.toml"), "[plugin]\n[scripts]\nlaunch = ['fixture', 'entry']\n")?;
        assert_eq!(super::development_input(root.path())?, DevelopmentInput::Project);
        let bytes = b"native executable";
        let manifest = serde_json::json!({"schema": 1, "identity": {"name": "example", "version": "1.0.0", "description": ""},
            "files": [{"path": "bin/plugin", "size": bytes.len(), "sha256": digest(bytes)}]});
        let file = bundle(&[("bin/plugin", bytes)], &manifest)?;
        assert_eq!(super::development_input(file.path())?, DevelopmentInput::Bundle);
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
        let bytes = super::create(&executable, &[root.path().to_path_buf()], identity.clone(), Some(b"declarations"))?;
        let bundle = bundle::Bundle::inspect(&bytes, Limits::default())?;
        assert_eq!(bundle.file("bin/plugin"), Some([2, 3].as_slice()));
        assert_eq!(bundle.file("bin/runtime.dll"), Some([5, 7].as_slice()));
        assert_eq!(bundle.file("bin/assets/data.bin"), Some([11, 13].as_slice()));
        assert_eq!(bundle.manifest().identity, identity);
        assert!(bundle.manifest().launch.is_none());
        assert_eq!(bundle.file("description.pb"), Some(b"declarations".as_slice()));
        let entrypoint = bundle.manifest().files.iter().find(|file| file.executable.is_some()).unwrap();
        assert_eq!(entrypoint.path, "bin/plugin");
        let executable = entrypoint.executable.as_ref().unwrap();
        assert_eq!(executable.facility, Facility::Sidecar);
        assert_eq!(executable.target, super::native_target());
        assert!(executable.registrations.is_empty());
        Ok(())
    }
    #[test]
    fn launch_assets_are_complete_while_private_inputs_are_excluded() -> crate::Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(
            root.path().join("dokime.toml"),
            "[plugin]\nversion='2.3.4'\nlabel='A Label'\npublisher='your-organization'\nicon='assets/icon.svg'\nreadme='README.md'\n[compatibility]\ndokime='>=0.1.0'\n[scripts]\nlaunch=['fixture','entry.bin']\n",
        )?;
        fs::write(root.path().join("entry.bin"), b"entry")?;
        fs::write(root.path().join("README.md"), b"Readme")?;
        fs::write(root.path().join(".gitignore"), "assets/\ndokime.toml\nREADME.md\nignored.bin\n")?;
        fs::write(root.path().join("ignored.bin"), b"ignored")?;
        fs::create_dir_all(root.path().join("assets/nested"))?;
        fs::write(root.path().join("assets/icon.svg"), b"icon")?;
        fs::write(root.path().join("assets/nested/ignored.bin"), b"asset")?;
        for name in [
            ".git",
            ".venv",
            "_input",
            "_output",
            "credentials",
            "Tests",
            "test",
            "__tests__",
            ".github",
            ".gitlab",
            ".hg",
            ".svn",
            ".idea",
            ".vscode",
            ".vs",
            ".pytest_cache",
            ".mypy_cache",
            ".ruff_cache",
            ".cache",
            "__pycache__",
        ] {
            fs::create_dir(root.path().join(name))?;
            fs::write(root.path().join(name).join("private"), b"private")?;
        }
        fs::write(root.path().join(".env"), b"private")?;
        for name in [
            "Mise.toml",
            ".mise.toml",
            "mise.lock",
            ".tool-versions",
            ".ENV.example",
            ".env.local",
            ".gitattributes",
            ".gitmodules",
            ".gitlab-ci.yml",
            ".hgignore",
            ".hgtags",
            ".DS_Store",
            "Thumbs.db",
        ] {
            fs::write(root.path().join(name), b"development")?;
        }
        fs::write(root.path().join("operator.pair"), b"private")?;
        let project = PluginProject::read(root.path())?;
        let identity = Identity {
            name: "class-identifier".into(),
            version: "1.0.0".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = super::create_project(&project, identity, None)?;
        let bundle = bundle::Bundle::inspect(&bytes, Limits::default())?;
        let inventory = bundle.manifest().files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>();
        assert_eq!(inventory, ["README.md", "assets/icon.svg", "assets/nested/ignored.bin", "dokime.json", "dokime.toml", "entry.bin"]);
        assert_eq!(bundle.manifest().identity.name, "class-identifier");
        assert_eq!(bundle.manifest().identity.version.to_string(), "2.3.4");
        let metadata: Value = serde_json::from_slice(bundle.file("dokime.json").unwrap())?;
        assert_eq!(metadata["requires_dokime"], ">=0.1.0");
        assert_eq!(metadata["plugin"]["label"], "A Label");
        assert_eq!(metadata["plugin"]["publisher"], "your-organization");
        Ok(())
    }

    #[test]
    fn managed_launch_keeps_staged_runtime_companions() -> crate::Result<()> {
        let root = tempfile::tempdir()?;
        fs::create_dir_all(root.path().join("assets/plugin"))?;
        fs::write(root.path().join(".gitignore"), "assets/\n")?;
        fs::write(root.path().join("dokime.toml"), "[plugin]\n[scripts]\nlaunch=['dotnet','assets/plugin/example.dll']\n")?;
        for name in ["example.dll", "example.deps.json", "example.runtimeconfig.json", "dependency.dll"] {
            fs::write(root.path().join("assets/plugin").join(name), name.as_bytes())?;
        }
        let project = PluginProject::read(root.path())?;
        let identity = Identity {
            name: "managed".into(),
            version: "1.2.3".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = super::create_project(&project, identity, None)?;
        let bundle = bundle::Bundle::inspect(&bytes, Limits::default())?;
        assert_eq!(bundle.manifest().launch.as_deref(), project.manifest.scripts.launch.as_deref());
        for name in ["example.dll", "example.deps.json", "example.runtimeconfig.json", "dependency.dll"] {
            assert_eq!(bundle.file(&format!("assets/plugin/{name}")), Some(name.as_bytes()));
        }
        Ok(())
    }

    #[test]
    fn native_inventory_contains_only_executable_assets_and_referenced_metadata() -> crate::Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(root.path().join("dokime.toml"), "[plugin]\nversion='1.0.0'\nreadme='README.md'\n[scripts]\nexecutable='plugin'\n")?;
        fs::write(root.path().join("plugin"), b"executable")?;
        fs::write(root.path().join("unrelated.dll"), b"unrelated")?;
        fs::write(root.path().join("README.md"), b"readme")?;
        fs::create_dir_all(root.path().join("assets/runtime"))?;
        fs::write(root.path().join("assets/runtime/companion.dll"), b"companion")?;
        let project = PluginProject::read(root.path())?;
        let identity = Identity {
            name: "native".into(),
            version: "1.0.0".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = super::create_project(&project, identity, None)?;
        let bundle = bundle::Bundle::inspect(&bytes, Limits::default())?;
        assert_eq!(
            bundle.manifest().files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>(),
            ["README.md", "assets/runtime/companion.dll", "bin/plugin", "dokime.json"]
        );
        assert!(bundle.manifest().files.iter().find(|file| file.path == "bin/plugin").unwrap().executable.is_some());
        Ok(())
    }

    #[test]
    fn ignored_required_launch_inputs_are_refused_and_missing_assets_are_valid() -> crate::Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(root.path().join("dokime.toml"), "[plugin]\n[scripts]\nlaunch=['fixture','entry.bin']\n")?;
        fs::write(root.path().join("entry.bin"), b"entry")?;
        fs::write(root.path().join(".gitignore"), "entry.bin\n")?;
        let project = PluginProject::read(root.path())?;
        let identity = Identity {
            name: "launch".into(),
            version: "1.0.0".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        assert!(super::create_project(&project, identity.clone(), None).is_err());
        fs::write(root.path().join(".gitignore"), "")?;
        super::create_project(&project, identity, None)?;
        Ok(())
    }

    #[test]
    fn source_lock_is_preserved_and_included_even_when_ignored() -> crate::Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(root.path().join("dokime.toml"), "[plugin]\n[scripts]\nlaunch=['uv','run','python','plugin.py']\n")?;
        fs::write(root.path().join("pyproject.toml"), "[project]\nname='fixture'\nversion='1.0.0'")?;
        fs::write(root.path().join("plugin.py"), b"entry")?;
        fs::write(root.path().join("uv.lock"), b"user-owned-lock")?;
        fs::write(root.path().join(".gitignore"), "uv.lock\npyproject.toml\n")?;
        let project = PluginProject::read(root.path())?;
        let identity = Identity {
            name: "launch".into(),
            version: "1.0.0".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = super::create_project(&project, identity, None)?;
        let bundle = bundle::Bundle::inspect(&bytes, Limits::default())?;
        assert_eq!(bundle.file("uv.lock"), Some(b"user-owned-lock".as_slice()));
        assert_eq!(fs::read(root.path().join("uv.lock"))?, b"user-owned-lock");
        assert!(bundle.file("pyproject.toml").is_some());
        Ok(())
    }

    #[test]
    fn excluded_required_script_inputs_are_refused() -> crate::Result<()> {
        let root = tempfile::tempdir()?;
        fs::create_dir_all(root.path().join("assets/tests"))?;
        fs::write(root.path().join("assets/tests/entry.py"), b"entry")?;
        fs::write(root.path().join("entry.py"), b"entry")?;
        fs::create_dir_all(root.path().join("tests"))?;
        fs::write(root.path().join("tests/__main__.py"), b"entry")?;
        let identity = Identity {
            name: "bench".into(),
            version: "1.0.0".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        for scripts in [
            "launch=['python','assets/tests/entry.py']",
            "launch=['python','entry.py']\nbuild=['python','assets/tests/entry.py']",
            "launch=['python','-m','tests']",
            "launch=['fixture']\nbuild=['python','-m','tests']",
        ] {
            fs::write(root.path().join("dokime.toml"), format!("[plugin]\n[scripts]\n{scripts}\n"))?;
            let project = PluginProject::read(root.path())?;
            assert!(super::create_project(&project, identity.clone(), None).is_err());
        }
        Ok(())
    }

    #[test]
    fn starter_bundle_inventories_exclude_development_inputs() -> crate::Result<()> {
        let sdk = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sdk");
        for language in ["rust", "python", "csharp", "cpp"] {
            let folder = if language == "csharp" { "Starter" } else { "starter" };
            let source = sdk.join(language).join("examples").join(folder);
            let root = tempfile::tempdir()?;
            for entry in WalkBuilder::new(&source).hidden(false).git_global(false).git_exclude(false).build() {
                let entry = entry?;
                if !entry.file_type().is_some_and(|kind| kind.is_file()) {
                    continue;
                }
                let path = root.path().join(entry.path().strip_prefix(&source)?);
                fs::create_dir_all(path.parent().unwrap())?;
                fs::copy(entry.path(), path)?;
            }
            let project = PluginProject::read(root.path())?;
            let executable = project.manifest.scripts.executable.clone().or_else(|| {
                project.manifest.scripts.launch.as_ref()?.iter().find(|argument| argument.ends_with(".dll")).map(PathBuf::from)
            });
            if let Some(executable) = executable {
                let path = root.path().join(executable);
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(path, b"runtime fixture")?;
            }
            let identity = Identity {
                name: "bench".into(),
                version: "0.1.0".parse()?,
                description: String::new(),
                package: Default::default(),
            };
            let bytes = super::create_project(&project, identity, None)?;
            let bundle = bundle::Bundle::inspect(&bytes, Limits::default())?;
            let inventory = bundle.manifest().files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>();
            assert!(!inventory.iter().any(|name| {
                name.split('/').any(|part| matches!(part, "mise.toml" | ".env.example" | ".gitignore" | ".gitattributes" | "tests"))
            }));
            eprintln!("{language}: {}", inventory.join(", "));
        }
        Ok(())
    }

    #[test]
    fn inspect_refuses_unmet_compatibility_before_install_or_dev() -> crate::Result<()> {
        let metadata = br#"{"schema":1,"requires_dokime":">=999.0.0"}"#;
        let manifest = serde_json::json!({"schema": 1, "identity": {"name": "example", "version": "1.0.0", "description": ""},
            "files": [{"path":"dokime.json","size":metadata.len(),"sha256":digest(metadata)}]});
        let file = bundle(&[("dokime.json", metadata)], &manifest)?;
        assert!(inspect(file.path()).is_err());
        let error = super::development_input(file.path()).unwrap_err().to_string();
        assert!(error.contains(">=999.0.0"), "{error}");
        Ok(())
    }
}
