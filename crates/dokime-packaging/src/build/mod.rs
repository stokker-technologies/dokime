/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod options;
mod project;

/* === Re-Exports === */

pub use options::Options;
pub use project::Project;

/* === Imports === */

// Standard Library Imports
use std::path::{Path, PathBuf};
use std::time::Duration;

// Third Party Imports
use cargo_metadata::{Message, Package};
use harmos_packaging::config::{BuildBackend, BuildProfile, BuildSettings};
use harmos_packaging::{Artifact, Build, BuildCancellation, Contents};

// Local Imports
use crate::{
    Error,
    bundle::{COMPATIBILITY_FILE, Compatibility, files},
    process,
};

/* === Constants === */

const PROBE_TIMEOUT_MS: u64 = 30_000;
const PROCESS_OUTPUT_BYTES: usize = 33_554_432;
const PROBE_OUTPUT_BYTES: usize = 65_536;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// One plugin's stamped executable as an earlier Cargo build produced it, ready to bundle without compiling again.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Collected {
    pub package: Package,
    pub build: Build,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

#[tracing::instrument(level = "debug", skip_all)]
pub async fn run(args: Options) -> Result<PathBuf, Error> {
    let metadata = metadata(&args, false).await?;
    build_resolved(args, metadata).await
}

// The selected executable of the build whose Cargo messages and metadata `args` names; Cargo is not run.
pub async fn collect(args: &Options) -> Result<Collected, Error> {
    let (Some(messages), Some(metadata)) = (&args.from_cargo_json, &args.metadata_json) else {
        return Err(Error::Invalid("collection requires Cargo messages and metadata".into()));
    };
    let metadata: cargo_metadata::Metadata = serde_json::from_slice(&std::fs::read(metadata)?)?;
    let package = select_package(&metadata, args.package.as_deref())?;
    let binary = binary(package)?;
    let target = host_target(package).await?;
    let reader = std::io::BufReader::new(std::fs::File::open(messages)?);
    let mut artifacts = Vec::new();
    for message in Message::parse_stream(reader) {
        if let Message::CompilerArtifact(artifact) = message?
            && artifact.package_id == package.id
            && !artifact.profile.test
            && artifact.target.name == binary
            && let Some(executable) = artifact.executable
        {
            let symbols =
                artifact.filenames.iter().find(|path| path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("pdb")));
            artifacts.push(Artifact {
                symbols: symbols.map(|path| path.clone().into_std_path_buf()),
                ..Artifact::read(package.name.to_string(), binary.to_owned(), executable.into_std_path_buf())?
            });
        }
    }
    artifacts.dedup_by(|left, right| left.path == right.path);
    if artifacts.len() != 1 {
        return Err(Error::Invalid(format!("Cargo did not report exactly one executable for binary {binary}")));
    }
    let profile = profile(args)?;
    let build =
        blocking(None, "shipping executable", move |_| harmos_packaging::Build::ship(BuildBackend::Sidecar, profile, target, artifacts))
            .await?;
    Ok(Collected { package: package.clone(), build })
}

// Bundle a collected executable into `output/<package>.dokime`.
pub async fn package(collected: Collected, output: &Path) -> Result<PathBuf, Error> {
    let destination = output.join(format!("{}.dokime", collected.package.name));
    bundle(collected.package, collected.build, destination, None).await
}

/* === Private Functions === */

async fn metadata(args: &Options, dependencies: bool) -> Result<cargo_metadata::Metadata, Error> {
    let mut metadata = cargo_metadata::MetadataCommand::new();
    metadata.cargo_path(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    if !dependencies {
        metadata.no_deps();
    }
    if let Some(path) = &args.manifest_path {
        metadata.manifest_path(path);
    }
    let output =
        process::run(metadata.cargo_command(), "Cargo metadata", args.cargo_timeout, PROCESS_OUTPUT_BYTES, PROBE_OUTPUT_BYTES).await?;
    Ok(serde_json::from_slice(&output.stdout)?)
}

async fn build_resolved(args: Options, metadata: cargo_metadata::Metadata) -> Result<PathBuf, Error> {
    let package = select_package(&metadata, args.package.as_deref())?.clone();
    let settings = settings(&args, &package)?;
    dokime::terminal::Terminal::default().status(format!("Building {}", package.name));
    let build = blocking(args.cargo_timeout, "Cargo compilation", move |cancellation| {
        harmos_packaging::build_with_cancellation(&settings, cancellation)
    })
    .await?;
    let target_dir = args.target_dir.unwrap_or_else(|| metadata.target_directory.into_std_path_buf());
    let output = args.output.unwrap_or_else(|| target_dir.join("dokime").join(format!("{}-{}.dokime", package.name, package.version)));
    bundle(package, build, output, args.cargo_timeout).await
}

fn settings(args: &Options, package: &Package) -> Result<BuildSettings, Error> {
    Ok(BuildSettings {
        backend: BuildBackend::Sidecar,
        manifest: package.manifest_path.clone().into_std_path_buf(),
        packages: vec![package.name.to_string()],
        target: args.target.clone(),
        target_directory: args.target_dir.clone().map(std::path::absolute).transpose()?,
        profile: profile(args)?,
        timings: false,
        features: args.features.clone(),
        default_features: !args.no_default_features,
        locked: args.locked,
    })
}

async fn bundle(package: Package, mut build: Build, output: PathBuf, timeout: Option<Duration>) -> Result<PathBuf, Error> {
    let binary = binary(&package)?.to_owned();
    build.artifacts.retain(|artifact| artifact.name == binary);
    let terminal = dokime::terminal::Terminal::default();
    terminal.status("Building verified plugin bundle");
    let root = package.manifest_path.parent().expect("Cargo manifest has a parent").as_std_path().to_owned();
    let requirements = package
        .dependencies
        .iter()
        .filter(|dependency| dependency.name == "dokime")
        .map(|dependency| dependency.req.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    if requirements.len() != 1 {
        return Err(Error::Invalid("plugin must declare one Dokime dependency requirement".into()));
    }
    let requires_dokime =
        requirements.first().expect("one requirement").parse().map_err(|error: semver::Error| Error::Invalid(error.to_string()))?;
    let compatibility = serde_json::to_vec(&Compatibility { schema: 1, requires_dokime })?;
    // Packaging selects symbols already emitted by Cargo; `[package.metadata.dokime] debuginfo = false` ships none.
    let symbols =
        package.metadata.get("dokime").and_then(|dokime| dokime.get("debuginfo")).and_then(serde_json::Value::as_bool) != Some(false);
    let limits = harmos_packaging::bundle::Limits::default();
    let destination = output.clone();
    blocking(timeout, "plugin packaging", move |cancellation| {
        let shape = |artifact: &Artifact, contents: &mut Contents| {
            let invalid = |error: Error| harmos_packaging::Error::Settings(error.to_string());
            contents.identity.name = package.name.to_string();
            contents.identity.description = package.description.clone().unwrap_or_default();
            contents.identity.package = harmos_packaging::bundle::PackageMetadata {
                authors: package.authors.clone(),
                license: package.license.clone(),
                repository: package.repository.clone(),
                homepage: package.homepage.clone(),
                documentation: package.documentation.clone(),
                keywords: package.keywords.clone(),
                categories: package.categories.clone(),
            };
            contents.files.insert(COMPATIBILITY_FILE.to_owned(), compatibility.clone());
            // Beside the executable, where dbghelp and the Unix loaders look for its debug info first.
            if let Some(pdb) = artifact.symbols.as_ref().filter(|_| symbols)
                && let Some((name, bytes)) = debug_symbols(pdb, limits.file_bytes).map_err(invalid)?
            {
                contents.files.insert(name, bytes);
            }
            for (name, path) in files::assets(&root.join("assets"), limits.files).map_err(invalid)? {
                contents.files.insert(name, files::read(&path, limits.file_bytes).map_err(invalid)?);
            }
            contents.destination = destination.clone();
            Ok(())
        };
        harmos_packaging::package_with(&build, Path::new("."), &dokime::config::schema_defaults(), cancellation, shape)
    })
    .await?;
    terminal.success(format!("Plugin bundle ready · {}", output.display()));
    Ok(output)
}

// Run a blocking Harmos packaging step, cancelling its process tree when `timeout` elapses.
async fn blocking<T: Send + 'static>(
    timeout: Option<Duration>, name: &str, step: impl FnOnce(&BuildCancellation) -> Result<T, harmos_packaging::Error> + Send + 'static,
) -> Result<T, Error> {
    let cancellation = BuildCancellation::default();
    let owned = cancellation.clone();
    let mut task = tokio::task::spawn_blocking(move || step(&owned));
    let deadline = async {
        match timeout {
            Some(duration) => tokio::time::sleep(duration).await,
            None => std::future::pending().await,
        }
    };
    tokio::select! {
        result = &mut task => Ok(result.map_err(|error| Error::Invalid(error.to_string()))??),
        _ = deadline => {
            cancellation.cancel();
            let _ = task.await;
            Err(Error::Timeout(name.to_owned()))
        }
    }
}

// Debug info is optional: a symbols file over the file budget is left out with a warning.
fn debug_symbols(pdb: &Path, limit: usize) -> Result<Option<(String, Vec<u8>)>, Error> {
    let name =
        pdb.file_name().and_then(|name| name.to_str()).ok_or_else(|| Error::Invalid("pdb filename must be UTF-8".into()))?.to_owned();
    match files::read(pdb, limit) {
        Ok(bytes) => Ok(Some((name, bytes))),
        Err(Error::Output(..)) => {
            let size = dokime::terminal::size_label(std::fs::metadata(pdb)?.len());
            let limit = dokime::terminal::size_label(limit as u64);
            dokime::terminal::Terminal::default()
                .warn(format!("{} is {size}, over the {limit} limit; the bundle ships without its debug symbols", pdb.display()));
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn profile(args: &Options) -> Result<BuildProfile, Error> {
    match (&args.profile, args.release) {
        (Some(profile), _) => profile.parse().map_err(|error: &str| Error::Invalid(error.into())),
        (None, true) => Ok(BuildProfile::Release),
        (None, false) => Ok(BuildProfile::Dev),
    }
}

fn binary(package: &Package) -> Result<&str, Error> {
    let bins = package.targets.iter().filter(|target| target.kind.contains(&cargo_metadata::TargetKind::Bin)).collect::<Vec<_>>();
    package
        .default_run
        .as_deref()
        .or_else(|| (bins.len() == 1).then(|| bins[0].name.as_str()))
        .ok_or_else(|| Error::Invalid(format!("package {} needs exactly one binary or package.default-run", package.name)))
}

async fn host_target(package: &Package) -> Result<String, Error> {
    let root = package.manifest_path.parent().expect("Cargo manifest has a parent");
    let mut probe = dokime::domain::process::command("rustc");
    probe.current_dir(root).arg("-vV");
    let timeout = Some(Duration::from_millis(PROBE_TIMEOUT_MS));
    let host = process::run(probe, "rustc host probe", timeout, PROBE_OUTPUT_BYTES, PROBE_OUTPUT_BYTES).await?;
    String::from_utf8_lossy(&host.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_owned)
        .ok_or_else(|| Error::Invalid("rustc did not report a host target".into()))
}

fn select_package<'a>(metadata: &'a cargo_metadata::Metadata, name: Option<&str>) -> Result<&'a Package, Error> {
    if let Some(name) = name {
        return metadata
            .packages
            .iter()
            .find(|package| package.name == name && metadata.workspace_members.contains(&package.id))
            .ok_or_else(|| Error::Invalid(format!("workspace package {name} not found")));
    }
    metadata
        .root_package()
        .or_else(|| {
            if metadata.workspace_default_members.len() == 1 {
                metadata.packages.iter().find(|package| package.id == metadata.workspace_default_members[0])
            } else {
                None
            }
        })
        .ok_or_else(|| Error::Invalid("select a workspace package with --package".into()))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_symbols_are_skipped() {
        let root = tempfile::tempdir().unwrap();
        let pdb = root.path().join("plugin.pdb");
        std::fs::write(&pdb, vec![0xAB; 2048]).unwrap();

        assert_eq!(debug_symbols(&pdb, 1024).unwrap(), None);
        assert_eq!(debug_symbols(&pdb, 4096).unwrap().unwrap().0, "plugin.pdb");
    }
}
