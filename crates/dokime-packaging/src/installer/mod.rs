/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod artwork;
mod distribution;
mod error;
mod license;
mod manifest;
mod native;
mod observer;
mod pack;
mod plugins;
mod policy;
mod release;
mod signing;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(unix)]
mod unix;

#[cfg(test)]
mod fixtures;

/* === Re-Exports === */

pub use error::Error;

#[cfg(target_os = "windows")]
pub(super) use windows::{scratch, stage_executable};

#[cfg(all(test, target_os = "windows"))]
pub(super) use windows::executable;

#[cfg(unix)]
pub(super) use unix::{scratch, stage_executable};

pub use manifest::{Format, Manifest};
pub use observer::{Observer, Stage, Step};
#[cfg(all(test, unix))]
pub(super) use unix::executable;

/* === Imports === */

// Standard Library Imports
use std::path::{Path, PathBuf};
use std::{env, fs};
use std::{ffi::OsStr, process::Command};

// Third Party Imports
use tauri_bundler::Settings;
use tempfile::TempDir;

// Local Imports
use crate::installation;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct PackageOptions {
    /// Constructs Tool Commands with the Product's Process Policy.
    pub command: fn(&OsStr) -> Command,

    pub artifacts: Option<PathBuf>,
    pub plugins: Vec<PathBuf>,
    pub output: PathBuf,
    pub sign: bool,
    pub formats: Vec<Format>,
    pub target: Option<String>,
}

/// Verified Native Packaging Inputs, Prepared Without Invoking an Installer Tool.
pub struct PreparedInstaller {
    settings: Settings,
    manifest: Manifest,
    distribution: distribution::Distribution,
    scratch: TempDir,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PreparedInstaller {
    /// Native Packaging Settings Borrow Their Prepared Resources Until This Owner Is Dropped.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

/* === Public Functions === */

pub async fn prepare(path: &Path, options: &PackageOptions, observer: &mut Observer<'_>) -> Result<PreparedInstaller, Error> {
    let result = prepare_inputs(path, options, observer).await;
    if let Err(error) = &result {
        observer.failed(error);
    }
    result
}

async fn prepare_inputs(path: &Path, options: &PackageOptions, observer: &mut Observer<'_>) -> Result<PreparedInstaller, Error> {
    observer.begin(Stage::Manifest);
    let mut manifest = Manifest::read(path)?;
    if !options.formats.is_empty() {
        manifest.installer.formats = options.formats.clone();
    }
    if let Some(target) = &options.target {
        manifest.dokime.target = target.clone();
    }
    manifest.validate()?;
    native::validate_signing(&manifest, options.sign)?;
    let formats = manifest.installer.formats.iter().map(|format| format!("{format:?}").to_lowercase()).collect::<Vec<_>>().join(", ");
    let identity = manifest.identity(&distribution::Identity::compiled()?)?;
    let identifier = identity.identifier;
    observer.complete(vec![format!("{} · {identifier} {} · {formats}", path.display(), manifest.installer.version)]);
    observer.begin(Stage::Kit);
    let scratch = scratch()?;
    let downloaded = scratch.path().join("distribution");
    let release = if options.artifacts.is_none() { Some(release::Release::resolve(manifest.dokime.version()).await?) } else { None };
    let mut downloaded_bytes = 0;
    let source = if let Some(source) = &options.artifacts {
        source.canonicalize()?
    } else {
        fs::create_dir_all(&downloaded)?;
        let release = release.as_ref().ok_or_else(|| Error::Invalid("missing resolved release".into()))?;
        let (version, bytes) = release.distribution(manifest.dokime.version(), &manifest.dokime.target, &downloaded).await?;
        downloaded_bytes = bytes;
        manifest.dokime.version = Some(version);
        downloaded
    };
    let distribution = distribution::validate(&source, &manifest.dokime)?;
    manifest.dokime.version = Some(distribution.version.clone());
    native::validate_signing(&manifest, options.sign)?;
    let acquisition = if options.artifacts.is_some() {
        "local".to_owned()
    } else {
        format!("downloaded {:.1} MB", downloaded_bytes as f64 / 1_000_000.0)
    };
    let revision: String = distribution.source_revision.chars().take(8).collect();
    observer.complete(vec![format!(
        "{} ({revision}, {}) · {} · {acquisition} · verified {} files",
        distribution.version,
        distribution.channel,
        distribution.target,
        distribution.files.len()
    )]);
    observer.begin(Stage::Appearance);
    let pack = native::resolve_organization(&manifest, &source)?;
    let appearance = manifest.installer.organization.as_ref().map(|path| path.display().to_string()).unwrap_or_else(|| "default".into());
    observer.complete(vec![appearance]);
    observer.begin(Stage::Plugins);
    let names = manifest.plugin_names(&distribution.plugins)?;
    let mut selected = options.plugins.clone();
    if options.artifacts.is_some() && !names.is_empty() {
        let directory = source.join(installation::PLUGINS);
        let mut paths = Vec::new();
        for entry in
            fs::read_dir(&directory).map_err(|error| Error::Invalid(format!("Add Requested .dokime Bundles to {directory:?}: {error}")))?
        {
            let path = entry?.path();
            if path.extension().is_some_and(|extension| extension == "dokime") {
                paths.push(path);
            }
        }
        selected.extend(plugins::select(&paths, &names)?);
    }
    selected.extend(manifest.plugin_paths(&source.join("organization"))?);
    if let Some(release) = release {
        let paths = release.plugins(&names, &distribution.target, &distribution.version, &scratch.path().join("release-plugins")).await?;
        selected.extend(paths);
    }
    let plugins = plugins::stage(&selected, scratch.path(), &distribution.target, &distribution.version)?;
    let mut details = Vec::new();
    for plugin in plugins.values() {
        let identity = &plugin.identity;
        let origin = if distribution.plugins.contains(&identity.name) && !manifest.installer.exclude_plugins.contains(&identity.name) {
            "default".to_owned()
        } else {
            plugin.source.display().to_string()
        };
        details.push(format!("{} {} ({origin})", identity.name, identity.version));
    }
    observer.complete(if details.is_empty() { vec!["none".into()] } else { details });
    observer.begin(Stage::Bundle);
    let provider = env::var("DOKIME_AUTHENTICODE_PROVIDER").ok();
    let authenticode = signing::authenticode_enabled(provider.as_deref())?;
    let settings = native::prepare(&manifest, &distribution, &plugins, scratch.path(), options, authenticode, pack)?;
    Ok(PreparedInstaller {
        settings,
        manifest,
        distribution,
        scratch,
    })
}

pub async fn package(path: &Path, options: &PackageOptions, observer: &mut Observer<'_>) -> Result<Vec<PathBuf>, Error> {
    let prepared = prepare(path, options, observer).await?;
    let result = publish(prepared, options, observer);
    if let Err(error) = &result {
        observer.failed(error);
    }
    result
}

fn publish(prepared: PreparedInstaller, options: &PackageOptions, observer: &mut Observer<'_>) -> Result<Vec<PathBuf>, Error> {
    let staged = prepared.scratch.path().join("installers");
    fs::create_dir_all(&staged)?;
    let outputs = native::assemble(&prepared.manifest, &prepared.distribution, &prepared.settings, &staged)?;
    observer.complete(vec![
        prepared.manifest.installer.formats.iter().map(|format| format!("{format:?}").to_lowercase()).collect::<Vec<_>>().join(", "),
    ]);
    observer.begin(Stage::Sign);
    for path in &outputs {
        native::sign(&prepared.manifest, &prepared.distribution, path, &staged, options)?;
    }
    let provider = signing::authenticode_enabled(env::var("DOKIME_AUTHENTICODE_PROVIDER").ok().as_deref())?;
    observer.complete(vec![if options.sign || provider {
        "signed and verified".into()
    } else {
        "skipped: no signing provider configured".into()
    }]);
    observer.begin(Stage::Output);
    fs::create_dir_all(&options.output)?;
    let output = options.output.canonicalize()?;
    let mut published = Vec::new();
    for path in outputs {
        let destination = output.join(path.file_name().ok_or_else(|| Error::Invalid("invalid installer output".into()))?);
        fs::copy(&path, &destination)?;
        let signature = path.with_extension("exe.sig");
        if signature.is_file() {
            fs::copy(&signature, destination.with_extension("exe.sig"))?;
        } else if destination.extension().is_some_and(|extension| extension == "exe") {
            let stale = destination.with_extension("exe.sig");
            if stale.exists() {
                fs::remove_file(stale)?;
            }
            let metadata = output.join("release.json");
            if metadata.is_file() {
                let value: serde_json::Value = serde_json::from_slice(&fs::read(&metadata)?)?;
                let named = value
                    .pointer("/platforms/windows-x86_64/url")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|url| url.rsplit('/').next());
                if named == destination.file_name().and_then(|name| name.to_str()) {
                    fs::remove_file(metadata)?;
                }
            }
        }
        published.push(destination);
    }
    if staged.join("release.json").is_file() {
        fs::copy(staged.join("release.json"), output.join("release.json"))?;
    }
    prepared.scratch.close()?;
    let details = published
        .iter()
        .map(|path| Ok(format!("{} · {:.1} MB", path.display(), fs::metadata(path)?.len() as f64 / 1_000_000.0)))
        .collect::<Result<Vec<_>, Error>>()?;
    observer.complete(details);
    Ok(published)
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use harmos_runtime::bundle::Identity;
    use std::{
        future::Future,
        pin::pin,
        task::{Context, Poll, Waker},
    };

    #[test]
    fn fabricated_kit_reports_steps_and_stages_defaults_with_exclusions() -> Result<(), Error> {
        let source = tempfile::tempdir()?;
        fixtures::kit(source.path())?;
        let executable = source.path().join("bench");
        fs::write(&executable, b"plugin")?;
        let project = source.path().join("project");
        fs::create_dir_all(&project)?;
        fs::write(project.join("dokime.toml"), "[plugin]\n[scripts]\nlaunch=['fixture']\n")?;
        let project = crate::project::PluginProject::read(&project).map_err(|error| Error::Invalid(error.to_string()))?;
        let identity = Identity {
            name: "bench".into(),
            version: "1.2.3".parse().unwrap(),
            description: String::new(),
            package: Default::default(),
        };
        let bytes = crate::bundle::create_project(&project, identity, None).map_err(|error| Error::Invalid(error.to_string()))?;
        fs::create_dir_all(source.path().join("plugins"))?;
        fs::write(source.path().join("plugins/bench.dokime"), bytes)?;
        let metadata = source.path().join("distribution.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&metadata)?)?;
        value["plugins"] = serde_json::json!(["bench"]);
        fs::write(metadata, serde_json::to_vec(&value)?)?;
        let path = source.path().join("installer.toml");
        let format = if cfg!(windows) { "nsis" } else { "deb" };
        let mut options = fixtures::options();
        options.artifacts = Some(source.path().to_owned());
        for excluded in [false, true] {
            let exclusion = if excluded { "exclude_plugins=['bench']\n" } else { "" };
            fs::write(&path, format!("[installer]\nversion='1.2.3'\nformats=['{format}']\n{exclusion}[dokime]\nversion='1.2.3'\n"))?;
            let mut steps = Vec::new();
            let mut report = |step: Step| steps.push((step.stage, step.failed));
            let mut observer = Observer::new(&mut report);
            let result = pin!(prepare(&path, &options, &mut observer)).poll(&mut Context::from_waker(Waker::noop()));
            let Poll::Ready(result) = result else { panic!("Local Preparation Must Not Wait for Network or Native Tools") };
            let prepared = result?;
            assert_eq!(prepared.scratch.path().join("plugins/bench.dokime").is_file(), !excluded);
            assert_eq!(steps, [(Stage::Manifest, false), (Stage::Kit, false), (Stage::Appearance, false), (Stage::Plugins, false)]);
        }
        fs::write(source.path().join("LICENSE"), b"changed")?;
        let mut steps = Vec::new();
        let mut report = |step: Step| steps.push((step.stage, step.failed));
        let mut observer = Observer::new(&mut report);
        let result = pin!(prepare(&path, &options, &mut observer)).poll(&mut Context::from_waker(Waker::noop()));
        assert!(matches!(result, Poll::Ready(Err(Error::DistributionDigest { .. }))));
        assert_eq!(steps, [(Stage::Manifest, false), (Stage::Kit, true)]);
        Ok(())
    }
}
