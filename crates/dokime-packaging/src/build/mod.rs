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
use std::{
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};
// Third Party Imports
use cargo_metadata::{Message, MetadataCommand, Package};
use harmos::plugin::bundle::{
    Bundle, Executable, Facility, Identity, Limits, PackageMetadata, Source,
};
use harmos::plugin::inventory::Declaration;
use harmos::plugin::sidecar::{CATALOG_EXPORT_FLAG, CATALOG_EXPORT_LIMIT, Registration};
// Local Imports
use crate::{Error, bundle::files, process};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[tracing::instrument(level = "debug", skip_all)]
pub async fn run(args: Options) -> Result<PathBuf, Error> {
    let metadata = metadata(&args, false).await?;
    build_resolved(args, metadata).await
}

/* === Private Functions === */
async fn metadata(args: &Options, dependencies: bool) -> Result<cargo_metadata::Metadata, Error> {
    let cargo_timeout = args.cargo_timeout;
    let cargo_options = args.cargo_options();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut metadata = MetadataCommand::new();
    metadata
        .cargo_path(&cargo)
        .other_options(cargo_options.clone());
    if !dependencies {
        metadata.no_deps();
    }
    if let Some(path) = &args.manifest_path {
        metadata.manifest_path(path);
    }
    let metadata = process::cargo(
        metadata.cargo_command(),
        if dependencies {
            "Resolving dependencies"
        } else {
            "Resolving package metadata"
        },
        cargo_timeout,
        false,
    )
    .await?;
    Ok(serde_json::from_slice(&metadata.stdout)?)
}

async fn build_resolved(
    args: Options,
    metadata: cargo_metadata::Metadata,
) -> Result<PathBuf, Error> {
    let cargo_timeout = args.cargo_timeout;
    let cargo_options = args.cargo_options();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let binaries = args.binaries.as_slice();
    let package = select_package(&metadata, args.package.as_deref())?;
    let root = package
        .manifest_path
        .parent()
        .expect("Cargo manifest has a parent");
    let bins = package
        .targets
        .iter()
        .filter(|target| target.kind.contains(&cargo_metadata::TargetKind::Bin))
        .collect::<Vec<_>>();
    let selected = if binaries.is_empty() {
        vec![package
            .default_run
            .as_deref()
            .or_else(|| (bins.len() == 1).then(|| bins[0].name.as_str()))
            .ok_or_else(|| {
                Error::Invalid(
                    "select one native binary with --bin (repeat for multiple files), or set package.default-run".into(),
                )
            })?
            .to_owned()]
    } else {
        binaries.to_vec()
    };
    let limits = Limits::default();
    if selected.len() > limits.files {
        return Err(Error::Invalid("too many selected executable files".into()));
    }
    let mut requested = std::collections::BTreeSet::new();
    for name in &selected {
        if !requested.insert(name) {
            return Err(Error::Invalid(format!("binary {name} was selected twice")));
        }
        if !bins.iter().any(|bin| bin.name == *name) {
            return Err(Error::Invalid(format!(
                "package {} has no binary {name}",
                package.name
            )));
        }
    }

    let mut probe = Command::new("rustc");
    probe.current_dir(root).arg("-vV");
    let host = process::run(
        probe,
        "rustc host probe",
        Some(Duration::from_secs(30)),
        64 * 1024,
        64 * 1024,
    )
    .await?;
    let host = String::from_utf8_lossy(&host.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_owned)
        .ok_or_else(|| Error::Invalid("rustc did not report a host target".into()))?;
    let target = args.target.as_deref().unwrap_or(&host);
    if target != host {
        return Err(Error::Invalid(format!(
            "native binaries require the native host target {host}"
        )));
    }
    let target_dir = args
        .target_dir
        .unwrap_or_else(|| metadata.target_directory.as_std_path().to_owned());
    std::fs::create_dir_all(&target_dir)?;
    // Lexical absolutization only: canonicalize yields a verbatim `\\?\` path
    // on Windows, which the MSVC linker rejects for build-script objects.
    let target_dir = std::path::absolute(target_dir)?;
    let mut command = Command::new(&cargo);
    command
        .current_dir(root)
        .args([
            "build",
            "--manifest-path",
            package.manifest_path.as_str(),
            "--package",
            &package.name,
            "--target",
            target,
            "--message-format=json-render-diagnostics",
            "--target-dir",
        ])
        .arg(&target_dir)
        .args(cargo_options);
    for name in &selected {
        command.args(["--bin", name]);
    }
    if args.release {
        command.arg("--release");
    }
    let profile_name = match &args.profile {
        Some(profile) => {
            command.args(["--profile", profile]);
            profile.clone()
        }
        None if args.release => "release".to_owned(),
        None => "dev".to_owned(),
    };
    let debuginfo = debuginfo_overrides(&mut command, &profile_name, target, package);
    let terminal = dokime_terminal::Terminal::default();
    terminal.status(format!(
        "Building {} ({}) for {target}",
        package.name,
        selected.join(", ")
    ));
    terminal.field("Debug info", &debuginfo);
    let output = process::cargo(command, "Cargo compilation", cargo_timeout, true).await?;
    let mut executables = std::collections::BTreeMap::<_, Vec<(PathBuf, Option<PathBuf>)>>::new();
    let mut finished = false;
    for message in Message::parse_stream(output.stdout.as_slice()) {
        match message? {
            Message::CompilerArtifact(artifact)
                if artifact.package_id == package.id
                    && requested.contains(&artifact.target.name)
                    && artifact
                        .target
                        .kind
                        .contains(&cargo_metadata::TargetKind::Bin) =>
            {
                // Cargo reports every emitted output for the unit here, debug
                // info included: on Windows MSVC that is the `.pdb` beside the
                // hash-named executable in `deps/`, not beside the copy
                // `executable` names.
                let pdb = artifact
                    .filenames
                    .iter()
                    .find(|path| {
                        path.extension()
                            .is_some_and(|extension| extension.eq_ignore_ascii_case("pdb"))
                    })
                    .map(|path| path.clone().into_std_path_buf());
                executables.entry(artifact.target.name).or_default().extend(
                    artifact
                        .executable
                        .into_iter()
                        .map(|path| (path.into_std_path_buf(), pdb.clone())),
                );
            }
            Message::BuildFinished(result) => finished = result.success,
            _ => {}
        }
    }
    for paths in executables.values_mut() {
        paths.sort();
        paths.dedup();
    }
    if !finished
        || executables.len() != selected.len()
        || executables.values().any(|paths| paths.len() != 1)
    {
        return Err(Error::Invalid(
            "Cargo did not report exactly one successful executable for each selected binary"
                .into(),
        ));
    }
    let terminal = dokime_terminal::Terminal::default();
    terminal.status("Collecting plugin declarations and assets");
    let collection_started = Instant::now();
    let mut inputs = Vec::new();
    let mut total = 0;
    let mut catalog_remaining = limits.manifest_bytes;
    for paths in executables.values() {
        let (path, pdb) = &paths[0];
        let (name, binary, executable, pdb) = collect_executable(
            path,
            pdb.as_deref(),
            package,
            target,
            limits
                .file_bytes
                .min(limits.total_file_bytes.saturating_sub(total)),
            &mut catalog_remaining,
        )
        .await?;
        total += binary.len();
        inputs.push((name, binary, Some(executable)));
        // Placed at `bin/<target>/<pdb name>`, the same directory `extract`
        // writes the executable to: dbghelp and the equivalent Unix loaders
        // search a module's own directory for its debug info first. Two
        // selected binaries reporting the identical pdb path is not expected
        // — Cargo hash-suffixes each unit's `deps/` output — but a collision
        // would fail `Bundle::build` outright, so the second copy is skipped
        // rather than turning a coincidence into a build failure.
        if let Some((name, bytes)) = pdb
            && !inputs.iter().any(|(existing, ..)| *existing == name)
        {
            total += bytes.len();
            inputs.push((name, bytes, None));
        }
    }
    for (name, path) in files::assets(
        &root.join("assets").into_std_path_buf(),
        limits.files - inputs.len(),
    )? {
        let bytes = files::read(
            &path,
            limits
                .file_bytes
                .min(limits.total_file_bytes.saturating_sub(total)),
        )?;
        total += bytes.len();
        inputs.push((name, bytes, None));
    }
    terminal.success(format!(
        "Plugin declarations and assets collected in {:.1}s",
        collection_started.elapsed().as_secs_f64()
    ));
    terminal.status("Building verified plugin bundle");
    let bundle_started = Instant::now();
    let sources = inputs
        .iter()
        .map(|(path, bytes, executable)| Source {
            path,
            bytes,
            executable: executable.clone(),
        })
        .collect::<Vec<_>>();
    let bundle = Bundle::build(package_identity(package), &sources, limits)?;
    let output = args.output.unwrap_or_else(|| {
        target_dir
            .join("dokime")
            .join(format!("{}-{}.dokime", package.name, package.version))
    });
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    std::fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    use std::io::Write;
    temporary.write_all(bundle.bytes())?;
    temporary
        .persist(&output)
        .map_err(|error| Error::Io(error.error))?;
    terminal.success(format!(
        "Plugin bundle ready · {} bytes in {:.1}s",
        bundle.bytes().len(),
        bundle_started.elapsed().as_secs_f64()
    ));
    Ok(output)
}

fn debuginfo_overrides(
    command: &mut Command,
    profile: &str,
    target: &str,
    package: &Package,
) -> String {
    let disabled = package
        .metadata
        .get("dokime")
        .and_then(|dokime| dokime.get("debuginfo"))
        .and_then(serde_json::Value::as_bool)
        == Some(false);
    if disabled {
        return "disabled via [package.metadata.dokime] debuginfo = false".to_owned();
    }
    let split_debuginfo = if target.contains("windows-msvc") {
        "packed"
    } else {
        "off"
    };
    command
        .arg("--config")
        .arg(format!("profile.{profile}.debug=\"line-tables-only\""))
        .arg("--config")
        .arg(format!("profile.{profile}.strip=\"none\""))
        .arg("--config")
        .arg(format!(
            "profile.{profile}.split-debuginfo=\"{split_debuginfo}\""
        ));
    format!("line tables only, unstripped, split-debuginfo={split_debuginfo} (profile.{profile})")
}

async fn collect_executable(
    path: &std::path::Path,
    pdb: Option<&std::path::Path>,
    package: &Package,
    target: &str,
    max_bytes: usize,
    catalog_remaining: &mut usize,
) -> Result<(String, Vec<u8>, Executable, Option<(String, Vec<u8>)>), Error> {
    let binary = files::read(path, max_bytes)?;
    let declaration = Declaration::inspect(&binary, harmos::plugin::inventory::Facility::Sidecar)
        .map_err(|error| {
        Error::Invalid(format!(
            "selected binary has no complete native declaration: {error}"
        ))
    })?;
    if declaration.version.to_string() != package.version.to_string() {
        return Err(Error::Invalid(format!(
            "executable version {} differs from Cargo package version {}; the executable declaration must derive its version from the selected Cargo package",
            declaration.version, package.version
        )));
    }
    let mut command = Command::new(path);
    command
        .current_dir(
            package
                .manifest_path
                .parent()
                .expect("Cargo manifest has a parent"),
        )
        .arg(CATALOG_EXPORT_FLAG);
    let export = process::run(
        command,
        "plugin catalog export",
        Some(Duration::from_secs(10)),
        CATALOG_EXPORT_LIMIT.min(*catalog_remaining),
        256 * 1024,
    )
    .await?;
    *catalog_remaining = catalog_remaining.saturating_sub(export.stdout.len());
    let registrations: Vec<Registration> = serde_json::from_slice(&export.stdout)?;
    if files::read(path, max_bytes)? != binary {
        return Err(Error::Invalid(
            "the executable changed during catalog export".into(),
        ));
    }
    let binary_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::Invalid("executable filename must be UTF-8".into()))?;
    let pdb = pdb
        .map(|pdb| {
            let bytes = files::read(pdb, max_bytes)?;
            let pdb_name = pdb
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| Error::Invalid("pdb filename must be UTF-8".into()))?;
            Ok::<_, Error>((format!("bin/{target}/{pdb_name}"), bytes))
        })
        .transpose()?;
    Ok((
        format!("bin/{target}/{binary_name}"),
        binary,
        Executable {
            facility: Facility::Sidecar,
            target: target.to_owned(),
            registrations,
        },
        pdb,
    ))
}

fn select_package<'a>(
    metadata: &'a cargo_metadata::Metadata,
    name: Option<&str>,
) -> Result<&'a Package, Error> {
    if let Some(name) = name {
        return metadata
            .packages
            .iter()
            .find(|package| {
                package.name == name && metadata.workspace_members.contains(&package.id)
            })
            .ok_or_else(|| Error::Invalid(format!("workspace package {name} not found")));
    }
    metadata
        .root_package()
        .or_else(|| {
            if metadata.workspace_default_members.len() == 1 {
                metadata
                    .packages
                    .iter()
                    .find(|package| package.id == metadata.workspace_default_members[0])
            } else {
                None
            }
        })
        .ok_or_else(|| Error::Invalid("select a workspace package with --package".into()))
}

fn package_identity(package: &Package) -> Identity {
    Identity {
        name: package.name.to_string(),
        version: package.version.clone(),
        description: package.description.clone().unwrap_or_default(),
        package: PackageMetadata {
            authors: package.authors.clone(),
            license: package.license.clone(),
            repository: package.repository.clone(),
            homepage: package.homepage.clone(),
            documentation: package.documentation.clone(),
            keywords: package.keywords.clone(),
            categories: package.categories.clone(),
        },
    }
}

/* === Tests === */
