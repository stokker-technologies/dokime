/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read and Validate Plugin Project Manifests

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeSet,
    env::consts::EXE_SUFFIX,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

// Third Party Imports
use harmos_runtime::bundle::{Identity, validate_launch};
use ignore::WalkBuilder;
use reqwest::Url;
use roxmltree::Document;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use spdx::Expression;
use toml::{Value, map::Map};

// Local Imports
use crate::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    pub plugin: PluginMetadata,
    pub compatibility: Option<Compatibility>,
    pub scripts: Scripts,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginMetadata {
    pub label: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Option<Vec<Author>>,
    pub license: Option<String>,
    pub repository: Option<String>,

    #[serde(default)]
    pub dynamic: Vec<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    pub publisher: Option<String>,
    pub icon: Option<PathBuf>,
    pub readme: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Compatibility {
    pub dokime: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scripts {
    pub build: Option<Vec<String>>,
    pub executable: Option<PathBuf>,
    pub launch: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct PluginProject {
    pub root: PathBuf,
    pub manifest: PluginManifest,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PluginProject {
    pub fn read(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let source = fs::read_to_string(root.join("dokime.toml"))
            .map_err(|source| format!("Read Plugin Project: Add dokime.toml in {}: {source}", root.display()))?;
        let manifest =
            toml::from_str::<PluginManifest>(&source).map_err(|source| format!("Read Plugin Project: Correct dokime.toml: {source}"))?;
        if manifest.scripts.executable.is_some() == manifest.scripts.launch.is_some() {
            return Err("Read Plugin Project: Set Exactly One of scripts.executable or scripts.launch in dokime.toml".into());
        }
        if let Some(build) = &manifest.scripts.build {
            validate_launch(build).map_err(|source| format!("Read Plugin Project: Correct scripts.build in dokime.toml: {source}"))?;
        }
        if let Some(launch) = &manifest.scripts.launch {
            validate_launch(launch).map_err(|source| format!("Read Plugin Project: Correct scripts.launch in dokime.toml: {source}"))?;
        }
        for (key, command) in [("build", &manifest.scripts.build), ("launch", &manifest.scripts.launch)] {
            if command.as_ref().is_some_and(|command| {
                command.first().is_none_or(|program| program.trim().is_empty()) || command.iter().any(|argument| argument.contains('\0'))
            }) {
                return Err(format!("Read Plugin Project: Set scripts.{key} to a Nonblank Program and Arguments Without NUL").into());
            }
        }
        let project = Self { root, manifest };
        if let Some(executable) = &project.manifest.scripts.executable {
            project.path(executable)?;
        }
        project.manifest.plugin.validate()?;
        project.check_compatibility(env!("CARGO_PKG_VERSION"))?;
        for relative in [&project.manifest.plugin.icon, &project.manifest.plugin.readme].into_iter().flatten() {
            project.path(relative)?;
        }
        for command in [&project.manifest.scripts.build, &project.manifest.scripts.launch].into_iter().flatten() {
            if command[0].contains(['/', '\\']) {
                project.path(Path::new(&command[0]))?;
            }
        }
        Ok(project)
    }

    pub fn validate_inputs(&self) -> Result<()> {
        let python = self.manifest.scripts.launch.as_ref().is_some_and(|command| {
            command.first().is_some_and(|program| program == "uv")
                && command.iter().any(|argument| argument == "python" || argument == "python3")
        });
        if python && !self.root.join("pyproject.toml").is_file() {
            return Err("Read Python Plugin Project: Add pyproject.toml to the Plugin Root Before Running uv".into());
        }
        for (key, relative) in [("plugin.icon", &self.manifest.plugin.icon), ("plugin.readme", &self.manifest.plugin.readme)] {
            if let Some(relative) = relative {
                let path = self.path(relative)?;
                if !path.is_file() || fs::symlink_metadata(&path)?.file_type().is_symlink() {
                    return Err(format!("Read Plugin Project: Set {key} to an Existing Regular File Inside the Project").into());
                }
                if key == "plugin.readme" {
                    fs::read_to_string(path).map_err(|cause| format!("Read Plugin Project: Save plugin.readme as UTF-8: {cause}"))?;
                }
            }
        }
        self.path(Path::new("assets"))?;
        if let Some(executable) = &self.manifest.scripts.executable {
            self.path(executable)?;
        }
        for command in [&self.manifest.scripts.build, &self.manifest.scripts.launch].into_iter().flatten() {
            if command[0].contains(['/', '\\']) {
                self.path(Path::new(&command[0]))?;
            }
        }
        Ok(())
    }

    pub fn check_compatibility(&self, version: &str) -> Result<()> {
        let requirement = self.manifest.compatibility.as_ref().and_then(|value| value.dokime.as_deref());
        check_compatibility(requirement, version)
    }

    pub fn resolve_metadata(&self, described_version: &str) -> Result<PluginMetadata> {
        let mut metadata = self.manifest.plugin.clone();
        let mut values = Value::try_from(&metadata)?;
        for key in &metadata.dynamic {
            if values.get(key).is_some() {
                continue;
            }
            let value = self.native_field(key)?;
            values.as_table_mut().ok_or("Resolve Plugin Metadata: Expected a Metadata Table")?.insert(key.clone(), value);
        }
        metadata = values
            .try_into()
            .map_err(|cause| format!("Resolve Plugin Metadata: Correct the Native Field Type or Set a Static plugin Value: {cause}"))?;
        if metadata.version.is_none() {
            metadata.version = Some(described_version.into());
        }
        metadata.validate()?;
        Ok(metadata)
    }

    pub fn resolve(&self, mut identity: Identity) -> Result<(Identity, PluginMetadata)> {
        let metadata = self.resolve_metadata(&identity.version.to_string())?;
        identity.version = metadata.version.as_deref().ok_or("Resolve Plugin Metadata: Set plugin.version")?.parse()?;
        if let Some(description) = &metadata.description {
            identity.description = description.clone();
        }
        if let Some(authors) = &metadata.authors {
            identity.package.authors = authors
                .iter()
                .map(|author| match &author.email {
                    Some(email) => format!("{} <{email}>", author.name),
                    None => author.name.clone(),
                })
                .collect();
        }
        identity.package.license = metadata.license.clone();
        identity.package.repository = metadata.repository.clone();
        identity.package.keywords = metadata.tags.clone();
        Ok((identity, metadata))
    }

    pub fn source_metadata(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        for name in ["dokime.toml", "pyproject.toml", "uv.lock"] {
            let path = self.path(Path::new(name))?;
            if path.is_file() {
                paths.push(path);
            }
        }
        for extension in ["toml", "csproj"] {
            for path in project_files(&self.root, extension)? {
                if extension == "csproj" || path.file_name().is_some_and(|name| name == "Cargo.toml") {
                    paths.push(path);
                }
            }
        }
        Ok(paths)
    }

    fn native_field(&self, key: &str) -> Result<Value> {
        let launch = self.manifest.scripts.launch.as_deref().unwrap_or_default();
        if launch.first().is_some_and(|program| program == "uv") && launch.iter().any(|value| value == "python" || value == "python3") {
            let path = self.path(Path::new("pyproject.toml"))?;
            let source: Value = toml::from_str(&fs::read_to_string(&path)?)?;
            let project =
                source.get("project").ok_or("Resolve Plugin Metadata: Add [project] to pyproject.toml or Set Static plugin Values")?;
            let native_key = if key == "repository" { "project.urls.Repository".to_owned() } else { format!("project.{key}") };
            let value = if key == "repository" { project.get("urls").and_then(|urls| urls.get("Repository")) } else { project.get(key) };
            if project.get("dynamic").and_then(Value::as_array).is_some_and(|fields| fields.iter().any(|value| value.as_str() == Some(key)))
            {
                return Err(format!(
                    "Resolve Plugin Metadata: pyproject.toml project.{key} Uses Build-Backend Metadata; Set plugin.{key} Statically"
                )
                .into());
            }
            return value
                .cloned()
                .ok_or_else(|| format!("Resolve Plugin Metadata: Set pyproject.toml {native_key} or plugin.{key} in dokime.toml").into());
        }
        let executable = self.manifest.scripts.executable.as_deref();
        if self.root.join("Cargo.toml").is_file() && executable.is_some() {
            return self.rust_field(key);
        }
        let entry = launch.iter().find(|argument| argument.ends_with(".dll")).map(Path::new).or(executable);
        if let Some(entry) = entry {
            let stem = entry.file_stem().and_then(|stem| stem.to_str()).ok_or("Resolve Plugin Metadata: Use a UTF-8 Plugin Entrypoint")?;
            let mut matches = Vec::new();
            for path in project_files(&self.root, "csproj")? {
                let source = fs::read_to_string(&path)?;
                let document = Document::parse(&source)?;
                let assembly = literal(&document, "AssemblyName")
                    .map_err(|cause| format!("Resolve Plugin Metadata: {}: {cause}; Set plugin.{key} Statically", path.display()))?;
                let assembly = assembly.as_deref().or_else(|| path.file_stem().and_then(|name| name.to_str()));
                if assembly == Some(stem) {
                    matches.push(path);
                }
            }
            if matches.len() != 1 {
                return Err(format!(
                    "Resolve Plugin Metadata: Match {stem} to Exactly One Literal C# AssemblyName or Set plugin.{key} Statically"
                )
                .into());
            }
            let path = &matches[0];
            let source = fs::read_to_string(path)?;
            let document = Document::parse(&source)?;
            let native_key = match key {
                "version" => "Version",
                "description" => "Description",
                "authors" => "Authors",
                "license" => "PackageLicenseExpression",
                "repository" => "RepositoryUrl",
                _ => unreachable!(),
            };
            if document.descendants().any(|node| node.has_tag_name("Import")) {
                return Err(format!("Resolve Plugin Metadata: {} Uses Imports; Set plugin.{key} Statically", path.display()).into());
            }
            let value = literal(&document, native_key)
                .map_err(|cause| format!("Resolve Plugin Metadata: {} {native_key}: {cause}; Set plugin.{key} Statically", path.display()))?
                .ok_or_else(|| format!("Resolve Plugin Metadata: Set {} {native_key} or plugin.{key} in dokime.toml", path.display()))?;
            if key == "authors" {
                return Ok(Value::Array(value.split(';').map(|name| author_value(name.trim(), None)).collect()));
            }
            return Ok(Value::String(value));
        }
        Err(format!("Resolve Plugin Metadata: No Supported Native Source for {key}; Set plugin.{key} Statically in dokime.toml").into())
    }

    fn rust_field(&self, key: &str) -> Result<Value> {
        let entry = self
            .manifest
            .scripts
            .executable
            .as_ref()
            .and_then(|path| path.file_stem())
            .and_then(|stem| stem.to_str())
            .ok_or("Resolve Plugin Metadata: Select the Rust Plugin Executable or Set Static plugin Values")?;
        let root: Value = toml::from_str(&fs::read_to_string(self.path(Path::new("Cargo.toml"))?)?)?;
        let mut matches = Vec::new();
        for path in project_files(&self.root, "toml")?.into_iter().filter(|path| path.file_name().is_some_and(|name| name == "Cargo.toml"))
        {
            let source: Value = toml::from_str(&fs::read_to_string(&path)?)?;
            let Some(package) = source.get("package") else {
                continue;
            };
            let explicit = source
                .get("bin")
                .and_then(Value::as_array)
                .is_some_and(|bins| bins.iter().any(|bin| bin.get("name").and_then(Value::as_str) == Some(entry)));
            let implicit = package.get("autobins").and_then(Value::as_bool).unwrap_or(true)
                && path.parent().is_some_and(|parent| {
                    (package.get("name").and_then(Value::as_str) == Some(entry) && parent.join("src/main.rs").is_file())
                        || parent.join("src/bin").join(format!("{entry}.rs")).is_file()
                        || parent.join("src/bin").join(entry).join("main.rs").is_file()
                });
            if explicit || implicit {
                matches.push((path, source));
            }
        }
        if matches.len() != 1 {
            return Err(format!(
                "Resolve Plugin Metadata: Match {entry} to Exactly One Cargo Package Binary or Set plugin.{key} Statically"
            )
            .into());
        }
        let (path, source) = &matches[0];
        let value = source.get("package").and_then(|package| package.get(key));
        if value.and_then(|value| value.get("workspace")).and_then(Value::as_bool) == Some(true) {
            // A Nested Standalone Workspace Does Not Inherit the Repository's Metadata.
            let mut directory = path.parent();
            let mut workspace = None;
            while let Some(parent) = directory {
                let candidate = parent.join("Cargo.toml");
                if candidate.is_file() {
                    let candidate: Value = toml::from_str(&fs::read_to_string(candidate)?)?;
                    if candidate.get("workspace").is_some() {
                        workspace = Some(candidate);
                        break;
                    }
                }
                directory = parent.parent();
            }
            let workspace = workspace.unwrap_or(root);
            let inherited =
                workspace.get("workspace").and_then(|workspace| workspace.get("package")).and_then(|package| package.get(key)).cloned();
            return rust_value(key, inherited, path);
        }
        rust_value(key, value.cloned(), path)
    }

    pub fn executable(&self) -> Result<PathBuf> {
        let relative = self.manifest.scripts.executable.as_deref().ok_or("Start Native Plugin: Set executable in dokime.toml")?;
        let mut executable = self.path(relative)?;
        if !executable.is_file() && executable.extension().is_none() && !EXE_SUFFIX.is_empty() {
            executable.set_extension(EXE_SUFFIX.trim_start_matches('.'));
        }
        let executable = executable.canonicalize().map_err(|source| {
            format!("Start Native Plugin: Build {} or Correct executable in dokime.toml: {source}", executable.display())
        })?;
        if !executable.starts_with(&self.root) || !executable.is_file() {
            return Err("Start Native Plugin: Set executable to a File Inside the Plugin Root in dokime.toml".into());
        }
        Ok(executable)
    }

    pub fn path(&self, relative: &Path) -> Result<PathBuf> {
        let text = relative.to_str().ok_or("Read Plugin Project: Use UTF-8 Paths in dokime.toml")?;
        if text.is_empty() || text.starts_with(['/', '\\']) || text.contains(':') {
            return Err("Read Plugin Project: Use Relative Paths Inside the Plugin Root in dokime.toml".into());
        }
        let mut path = PathBuf::new();
        for component in text.split(['/', '\\']) {
            match component {
                "" | "." => {}
                ".." => {
                    if !path.pop() {
                        return Err("Read Plugin Project: Keep Paths Inside the Plugin Root in dokime.toml".into());
                    }
                }
                component => path.push(component),
            }
        }
        let path = self.root.join(path);
        let mut ancestor = path.as_path();
        loop {
            match ancestor.canonicalize() {
                Ok(resolved) => {
                    if !resolved.starts_with(&self.root) {
                        return Err("Read Plugin Project: Keep Symbolic Links Inside the Plugin Root".into());
                    }
                    break;
                }
                Err(source) if source.kind() == ErrorKind::NotFound => {
                    ancestor = ancestor.parent().ok_or("Read Plugin Project: Keep Paths Inside the Plugin Root")?;
                }
                Err(source) => return Err(source.into()),
            }
        }
        Ok(path)
    }
}

impl PluginMetadata {
    fn validate(&self) -> Result<()> {
        for (key, value) in
            [("label", &self.label), ("description", &self.description), ("license", &self.license), ("publisher", &self.publisher)]
        {
            if value.as_ref().is_some_and(|value| value.trim().is_empty()) {
                return Err(format!("Read Plugin Project: Set plugin.{key} to a Nonblank Value").into());
            }
        }
        if let Some(version) = &self.version {
            Version::parse(version).map_err(|cause| format!("Read Plugin Project: Correct plugin.version: {cause}"))?;
        }
        if let Some(license) = &self.license {
            Expression::parse(license)
                .map_err(|cause| format!("Read Plugin Project: Set plugin.license to an SPDX Expression: {cause}"))?;
        }
        if let Some(repository) = &self.repository {
            let url = Url::parse(repository)
                .map_err(|cause| format!("Read Plugin Project: Set plugin.repository to an Absolute HTTP(S) URL: {cause}"))?;
            if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
                return Err("Read Plugin Project: Set plugin.repository to an Absolute HTTP(S) URL".into());
            }
        }
        let invalid_authors = self.authors.as_ref().is_some_and(|authors| {
            let empty = authors.is_empty();
            let invalid = authors
                .iter()
                .any(|author| author.name.trim().is_empty() || author.email.as_ref().is_some_and(|email| email.trim().is_empty()));
            empty || invalid
        });
        if invalid_authors {
            return Err("Read Plugin Project: Set plugin.authors to Nonblank Names and Optional Nonblank Emails".into());
        }
        for (key, values) in [("dynamic", &self.dynamic), ("tags", &self.tags)] {
            let mut unique = BTreeSet::new();
            for value in values {
                if value.trim().is_empty() || !unique.insert(value) {
                    return Err(format!("Read Plugin Project: Use Distinct Nonblank plugin.{key} Values").into());
                }
                if key == "dynamic" && !matches!(value.as_str(), "version" | "description" | "authors" | "license" | "repository") {
                    return Err(format!("Read Plugin Project: Remove Unknown plugin.dynamic Field {value}").into());
                }
            }
        }
        Ok(())
    }
}

/* === Public Functions === */

pub fn check_compatibility(requirement: Option<&str>, version: &str) -> Result<()> {
    let Some(requirement) = requirement else {
        return Ok(());
    };
    if requirement.trim().is_empty() {
        return Err("Check Plugin Compatibility: Set compatibility.dokime to a Nonempty Version Requirement".into());
    }
    let parsed = VersionReq::parse(requirement)
        .map_err(|cause| format!("Check Plugin Compatibility: Correct compatibility.dokime {requirement:?}: {cause}"))?;
    let version = Version::parse(version)?;
    if !parsed.matches(&version) {
        return Err(format!("Check Plugin Compatibility: Plugin Requires Dokime {requirement}, Selected Dokime Version {version}; Select a Matching Kit or Correct compatibility.dokime").into());
    }
    Ok(())
}

/* === Private Functions === */

fn project_files(root: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut walker = WalkBuilder::new(root);
    walker.hidden(false).parents(false).ignore(false).git_ignore(false).git_global(false).git_exclude(false).filter_entry(|entry| {
        !matches!(entry.file_name().to_str(), Some("target" | "build" | "bin" | "obj" | ".git" | ".venv" | "assets"))
    });
    for entry in walker.build() {
        let entry = entry?;
        if entry.path().extension().is_some_and(|value| value == extension) && entry.file_type().is_some_and(|kind| kind.is_file()) {
            let path = entry.path().canonicalize()?;
            if !path.starts_with(root) {
                return Err("Resolve Plugin Metadata: Keep Native Project Files Inside the Project".into());
            }
            files.push(path);
        }
    }
    Ok(files)
}

fn literal(document: &Document<'_>, key: &str) -> Result<Option<String>> {
    let mut values = BTreeSet::new();
    for node in document.descendants().filter(|node| node.has_tag_name(key)) {
        if !node.parent().is_some_and(|parent| parent.has_tag_name("PropertyGroup")) {
            continue;
        }
        if node.ancestors().any(|ancestor| ancestor.attribute("Condition").is_some()) || node.children().any(|child| child.is_element()) {
            return Err("Use an Unconditional Literal Property".into());
        }
        let value = node.text().unwrap_or("").trim();
        if value.contains("$(") || value.contains("@(") || value.contains("%(") {
            return Err("Use a Literal Property Without MSBuild Expressions".into());
        }
        values.insert(value.to_owned());
    }
    if values.len() > 1 {
        return Err("Remove Conflicting Property Values".into());
    }
    Ok(values.into_iter().next())
}

fn author_value(name: &str, email: Option<&str>) -> Value {
    let mut author = Map::new();
    author.insert("name".into(), Value::String(name.into()));
    if let Some(email) = email {
        author.insert("email".into(), Value::String(email.into()));
    }
    Value::Table(author)
}

fn rust_value(key: &str, value: Option<Value>, path: &Path) -> Result<Value> {
    let value =
        value.ok_or_else(|| format!("Resolve Plugin Metadata: Set {} package.{key} or plugin.{key} in dokime.toml", path.display()))?;
    if key != "authors" {
        return Ok(value);
    }
    let authors = value.as_array().ok_or("Resolve Plugin Metadata: Set Cargo package.authors to an Array of Strings")?;
    let mut result = Vec::new();
    for author in authors {
        let author = author.as_str().ok_or("Resolve Plugin Metadata: Set Cargo package.authors to Strings")?;
        let parsed = author.strip_suffix('>').and_then(|author| author.rsplit_once('<'));
        result.push(match parsed {
            Some((name, email)) => author_value(name.trim(), Some(email.trim())),
            None => author_value(author.trim(), None),
        });
    }
    Ok(Value::Array(result))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{PluginProject, check_compatibility};
    use crate::Result;
    use std::{fs, path::Path};

    fn write(root: &Path, metadata: &str, scripts: &str) -> Result<PluginProject> {
        fs::write(root.join("dokime.toml"), format!("[plugin]\n{metadata}\n[scripts]\n{scripts}\n"))?;
        PluginProject::read(root)
    }

    #[test]
    fn strict_schema_accepts_metadata_and_preserves_tag_order() -> Result<()> {
        let root = tempfile::tempdir()?;
        let project = write(
            root.path(),
            "label='Example'\nversion='1.2.3'\ndescription='Example Plugin'\nauthors=[{name='Your Name',email='author@example.com'}]\nlicense='MIT OR Apache-2.0'\nrepository='https://example.com/repo'\ndynamic=['version']\ntags=['second','first']\npublisher='your-organization'",
            "build=['cargo','build']\nexecutable='target/debug/plugin'",
        )?;
        let resolved = project.resolve_metadata("")?;
        assert_eq!(resolved.version.as_deref(), Some("1.2.3"));
        assert_eq!(resolved.tags, ["second", "first"]);
        Ok(())
    }

    #[test]
    fn schema_refuses_unknown_keys_at_every_level_and_old_root_keys() -> Result<()> {
        let root = tempfile::tempdir()?;
        for (source, key) in [
            ("build=[]\n[plugin]\n[scripts]\nlaunch=['fixture']", "build"),
            ("launch=['fixture']\n[plugin]\n[scripts]\nlaunch=['fixture']", "launch"),
            ("executable='fixture'\n[plugin]\n[scripts]\nlaunch=['fixture']", "executable"),
            ("assets=[]\n[plugin]\n[scripts]\nlaunch=['fixture']", "assets"),
            ("[unknown]\n[plugin]\n[scripts]\nlaunch=['fixture']", "unknown"),
            ("[plugin]\nidentifier='fixture'\n[scripts]\nlaunch=['fixture']", "identifier"),
            ("[plugin]\nauthors=[{name='Your Name',unknown='value'}]\n[scripts]\nlaunch=['fixture']", "unknown"),
            ("[plugin]\n[compatibility]\nunknown='value'\n[scripts]\nlaunch=['fixture']", "unknown"),
            ("[plugin]\n[scripts]\nlaunch=['fixture']\nrun=['fixture']", "run"),
        ] {
            fs::write(root.path().join("dokime.toml"), source)?;
            let error = PluginProject::read(root.path()).unwrap_err().to_string();
            assert!(error.contains(key), "{error}");
        }
        Ok(())
    }

    #[test]
    fn schema_refuses_wrong_types_duplicates_and_invalid_metadata() -> Result<()> {
        let root = tempfile::tempdir()?;
        for metadata in [
            "label=1",
            "label=' '",
            "description=' '",
            "publisher=' '",
            "version='next'",
            "license='not-a-license'",
            "repository='file:///repo'",
            "repository='/repo'",
            "authors=[]",
            "authors=[{name=' '}]",
            "authors=[{name='Your Name',email=' '}]",
            "tags=['x','x']",
            "tags=[' ']",
            "dynamic=['version','version']",
            "dynamic=['identifier']",
            "label='A'\nlabel='B'",
        ] {
            assert!(write(root.path(), metadata, "launch=['fixture']").is_err(), "{metadata}");
        }
        for scripts in [
            "",
            "launch=['fixture']\nexecutable='plugin'",
            "launch=[]",
            "launch=[' ']",
            "launch='fixture'",
            "launch=['fixture']\nbuild=[]",
            "executable='../plugin'",
            "launch=['../fixture']",
            r#"launch=['fixture']
build=["x\u0000"]"#,
        ] {
            assert!(write(root.path(), "", scripts).is_err(), "{scripts}");
        }
        for source in ["[scripts]\nlaunch=['fixture']", "[plugin]"] {
            fs::write(root.path().join("dokime.toml"), source)?;
            assert!(PluginProject::read(root.path()).is_err());
        }
        Ok(())
    }

    #[test]
    fn python_resolves_only_requested_fields_and_static_overrides() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(
            root.path().join("pyproject.toml"),
            "[project]\nname='not-the-plugin-identity'\nversion='2.3.4'\ndescription='Native Description'\nauthors=[{name='Your Name',email='author@example.com'}]\nlicense='Apache-2.0'\n[project.urls]\nRepository='https://example.com/repo'\n",
        )?;
        let project = write(
            root.path(),
            "dynamic=['version','description','authors','license','repository']\ndescription='Override'",
            "launch=['uv','run','python','plugin.py']",
        )?;
        let metadata = project.resolve_metadata("")?;
        assert_eq!(metadata.version.as_deref(), Some("2.3.4"));
        assert_eq!(metadata.description.as_deref(), Some("Override"));
        assert_eq!(metadata.authors.unwrap()[0].email.as_deref(), Some("author@example.com"));
        assert_eq!(metadata.license.as_deref(), Some("Apache-2.0"));
        assert_eq!(metadata.repository.as_deref(), Some("https://example.com/repo"));
        let project = write(root.path(), "", "launch=['uv','run','python','plugin.py']")?;
        let metadata = project.resolve_metadata("3.0.0")?;
        assert_eq!(metadata.version.as_deref(), Some("3.0.0"));
        assert!(metadata.description.is_none());
        Ok(())
    }

    #[test]
    fn python_refuses_missing_dynamic_backend_and_legacy_license_fields() -> Result<()> {
        let root = tempfile::tempdir()?;
        for native in ["[project]\nname='fixture'", "[project]\nversion='1.0.0'\ndynamic=['version']", "[project]\nlicense={text='MIT'}"] {
            fs::write(root.path().join("pyproject.toml"), native)?;
            let key = if native.contains("license=") { "license" } else { "version" };
            let project = write(root.path(), &format!("dynamic=['{key}']"), "launch=['uv','run','python','plugin.py']")?;
            assert!(project.resolve_metadata("1.0.0").is_err());
        }
        let project = write(root.path(), "version='1.2.3'\ndynamic=['version']", "launch=['uv','run','python','plugin.py']")?;
        assert_eq!(project.resolve_metadata("")?.version.as_deref(), Some("1.2.3"));
        Ok(())
    }

    #[test]
    fn rust_resolves_selected_binary_workspace_fields_and_author_emails() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(
            root.path().join("Cargo.toml"),
            "[workspace]\nmembers=['plugin','client']\n[workspace.package]\nversion='4.5.6'\nauthors=['Your Name <author@example.com>']\nlicense='MIT'\nrepository='https://example.com/repo'\ndescription='Native Plugin'\n",
        )?;
        fs::create_dir(root.path().join("plugin"))?;
        fs::create_dir(root.path().join("client"))?;
        fs::write(
            root.path().join("plugin/Cargo.toml"),
            "[package]\nname='package-name'\nversion.workspace=true\nauthors.workspace=true\nlicense.workspace=true\nrepository.workspace=true\ndescription.workspace=true\n[[bin]]\nname='plugin-binary'\npath='src/plugin.rs'\n",
        )?;
        fs::write(
            root.path().join("client/Cargo.toml"),
            "[package]\nname='client'\nversion='8.0.0'\n[[bin]]\nname='client'\npath='main.rs'",
        )?;
        let project = write(
            root.path(),
            "dynamic=['version','authors','license','repository','description']",
            "executable='target/debug/plugin-binary'",
        )?;
        let metadata = project.resolve_metadata("")?;
        assert_eq!(metadata.version.as_deref(), Some("4.5.6"));
        assert_eq!(metadata.description.as_deref(), Some("Native Plugin"));
        assert_eq!(metadata.authors.unwrap()[0].email.as_deref(), Some("author@example.com"));
        assert_eq!(metadata.license.as_deref(), Some("MIT"));
        assert_eq!(metadata.repository.as_deref(), Some("https://example.com/repo"));
        Ok(())
    }

    #[test]
    fn rust_refuses_ambiguous_binary_and_missing_field() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(root.path().join("Cargo.toml"), "[package]\nname='fixture'\n[[bin]]\nname='fixture'\npath='main.rs'")?;
        let project = write(root.path(), "dynamic=['version']", "executable='target/debug/fixture'")?;
        let error = project.resolve_metadata("1.0.0").unwrap_err().to_string();
        assert!(error.contains("package.version"), "{error}");
        fs::create_dir(root.path().join("other"))?;
        fs::copy(root.path().join("Cargo.toml"), root.path().join("other/Cargo.toml"))?;
        assert!(project.resolve_metadata("1.0.0").is_err());
        Ok(())
    }

    #[test]
    fn rust_resolves_auto_discovered_binary_and_respects_autobins() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::create_dir_all(root.path().join("src/bin"))?;
        fs::write(root.path().join("src/bin/plugin.rs"), "fn main() {}")?;
        fs::write(root.path().join("Cargo.toml"), "[package]\nname='authoring'\nversion='1.2.3'\n")?;
        let project = write(root.path(), "dynamic=['version']", "executable='target/debug/plugin'")?;
        assert_eq!(project.resolve_metadata("")?.version.as_deref(), Some("1.2.3"));
        fs::write(root.path().join("Cargo.toml"), "[package]\nname='authoring'\nversion='1.2.3'\nautobins=false\n")?;
        assert!(project.resolve_metadata("").is_err());
        Ok(())
    }

    #[test]
    fn csharp_resolves_only_matching_assembly_and_all_fields() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(
            root.path().join("Plugin.csproj"),
            "<Project><PropertyGroup><AssemblyName>plugin</AssemblyName><Version>1.2.3</Version><Description>Native Plugin</Description><Authors>Your Name;Another Name</Authors><PackageLicenseExpression>MIT</PackageLicenseExpression><RepositoryUrl>https://example.com/repo</RepositoryUrl></PropertyGroup></Project>",
        )?;
        fs::write(
            root.path().join("Client.csproj"),
            "<Project><PropertyGroup><AssemblyName>client</AssemblyName><Version>9.0.0</Version></PropertyGroup></Project>",
        )?;
        let project = write(
            root.path(),
            "dynamic=['version','description','authors','license','repository']",
            "launch=['dotnet','assets/plugin/plugin.dll']",
        )?;
        let metadata = project.resolve_metadata("")?;
        assert_eq!(metadata.version.as_deref(), Some("1.2.3"));
        assert_eq!(metadata.description.as_deref(), Some("Native Plugin"));
        assert_eq!(metadata.authors.unwrap().iter().map(|author| author.name.as_str()).collect::<Vec<_>>(), ["Your Name", "Another Name"]);
        assert_eq!(metadata.license.as_deref(), Some("MIT"));
        assert_eq!(metadata.repository.as_deref(), Some("https://example.com/repo"));
        Ok(())
    }

    #[test]
    fn csharp_refuses_missing_ambiguous_conditional_imported_and_computed_metadata() -> Result<()> {
        let root = tempfile::tempdir()?;
        let project = write(root.path(), "dynamic=['version']", "launch=['dotnet','Plugin.dll']")?;
        for native in [
            "<Project><PropertyGroup/></Project>",
            r#"<Project><PropertyGroup><Version Condition="'x'=='y'">1.0.0</Version></PropertyGroup></Project>"#,
            r#"<Project><PropertyGroup Condition="'x'=='y'"><Version>1.0.0</Version></PropertyGroup></Project>"#,
            "<Project><PropertyGroup><Version>$(VersionPrefix)</Version></PropertyGroup></Project>",
            "<Project><PropertyGroup><Version>1.0.0</Version><Version>2.0.0</Version></PropertyGroup></Project>",
            "<Project><Import Project='version.props'/><PropertyGroup><Version>1.0.0</Version></PropertyGroup></Project>",
        ] {
            fs::write(root.path().join("Plugin.csproj"), native)?;
            assert!(project.resolve_metadata("1.0.0").is_err(), "{native}");
        }
        fs::write(root.path().join("Plugin.csproj"), "<Project><PropertyGroup><Version>1.2.3</Version></PropertyGroup></Project>")?;
        fs::write(
            root.path().join("Other.csproj"),
            "<Project><PropertyGroup><AssemblyName>Plugin</AssemblyName><Version>1.2.3</Version></PropertyGroup></Project>",
        )?;
        assert!(project.resolve_metadata("1.0.0").is_err());
        Ok(())
    }

    #[test]
    fn cpp_requires_static_metadata_and_allows_dynamic_static_overrides() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(root.path().join("CMakeLists.txt"), "project(fixture VERSION 9.0.0)")?;
        let project = write(root.path(), "dynamic=['version']", "executable='build/plugin'")?;
        assert!(project.resolve_metadata("1.0.0").is_err());
        let project = write(root.path(), "dynamic=['version']\nversion='2.0.0'", "executable='build/plugin'")?;
        assert_eq!(project.resolve_metadata("1.0.0")?.version.as_deref(), Some("2.0.0"));
        Ok(())
    }

    #[test]
    fn compatibility_accepts_matching_and_refuses_invalid_or_unmet_requirements() -> Result<()> {
        check_compatibility(Some(">=1.0, <2"), "1.5.0")?;
        check_compatibility(None, "1.0.0")?;
        for requirement in ["", " ", "next", ">=2.0"] {
            assert!(check_compatibility(Some(requirement), "1.5.0").is_err(), "{requirement}");
        }
        Ok(())
    }

    #[test]
    fn native_executable_suffix_and_metadata_file_validation() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::create_dir_all(root.path().join("target/debug"))?;
        let executable = root.path().join(format!("target/debug/plugin{}", super::EXE_SUFFIX));
        fs::write(&executable, [2, 3, 5])?;
        let project = write(root.path(), "icon='missing.svg'", "executable='target/debug/plugin'")?;
        assert_eq!(project.executable()?, executable.canonicalize()?);
        assert!(project.validate_inputs().is_err());
        fs::write(root.path().join("missing.svg"), b"icon")?;
        project.validate_inputs()?;
        let project = write(root.path(), "readme='README.md'", "executable='target/debug/plugin'")?;
        fs::write(root.path().join("README.md"), [255])?;
        assert!(project.validate_inputs().is_err());
        Ok(())
    }
}
