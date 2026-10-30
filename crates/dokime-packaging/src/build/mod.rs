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
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

// Third Party Imports
use cargo_metadata::{MetadataCommand, Package};
use dokime::harmos::plugin::inventory::Declaration;
use dokime::harmos::sidecar::CATALOG_EXPORT_LIMIT;
use harmos_packaging::bundle::{Bundle, Executable, Facility, Identity, Limits, PackageMetadata, Source};

// Local Imports
use crate::{
    Error,
    bundle::{COMPATIBILITY_FILE, Compatibility, files},
    process,
};

/* === Constants === */

const PROBE_TIMEOUT_MS: u64 = 30_000;
const PROBE_OUTPUT_BYTES: u64 = 65_536;
const STRIP_TIMEOUT_MS: u64 = 120_000;
const CATALOG_EXPORT_TIMEOUT_MS: u64 = 10_000;
const CATALOG_STDERR_BYTES: u64 = 262_144;

/* === Macros === */

/* === Type Aliases === */

type Executables = BTreeMap<String, (PathBuf, Option<PathBuf>)>;

/* === Structs === */

// One plugin's executables as an earlier Cargo build produced them, ready to bundle without compiling again.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Collected {
    pub package: Package,
    pub target: String,
    pub executables: BTreeMap<String, (PathBuf, Option<PathBuf>)>,
    // The package opted out of debug info, so packaging strips the shared build's DWARF.
    pub strip: bool,
}

// Packaging Selects Symbols Already Emitted By Cargo, Without Changing Its Profiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DebugInfo {
    enabled: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl DebugInfo {
    fn of(package: &Package) -> Self {
        let disabled =
            package.metadata.get("dokime").and_then(|dokime| dokime.get("debuginfo")).and_then(serde_json::Value::as_bool) == Some(false);
        Self { enabled: !disabled }
    }

    fn summary(self) -> &'static str {
        if !self.enabled {
            return "Cargo defaults; omit debug symbols via [package.metadata.dokime] debuginfo = false";
        }
        "Cargo defaults; include emitted debug symbols"
    }
}

/* === Public Functions === */

#[tracing::instrument(level = "debug", skip_all)]
pub async fn run(args: Options) -> Result<PathBuf, Error> {
    let metadata = metadata(&args, false).await?;
    build_resolved(args, metadata).await
}

// The selected executables of the build whose Cargo messages and metadata `args` names; Cargo is not run.
pub async fn collect(args: &Options) -> Result<Collected, Error> {
    let (Some(messages), Some(metadata)) = (&args.from_cargo_json, &args.metadata_json) else {
        return Err(Error::Invalid("collection requires Cargo messages and metadata".into()));
    };
    let metadata: cargo_metadata::Metadata = serde_json::from_slice(&std::fs::read(metadata)?)?;
    let package = select_package(&metadata, args.package.as_deref())?;
    let selected = selected_binaries(package, &args.binaries)?;
    let target = host_target(package).await?;
    if args.target.as_deref().is_some_and(|requested| requested != target) {
        return Err(Error::Invalid(format!("native binaries require the native host target {target}")));
    }
    let debuginfo = DebugInfo::of(package);
    let reader = std::io::BufReader::new(std::fs::File::open(messages)?);
    let executables = selected_executables(reader, &package.id.repr, &package.name, &selected, debuginfo)?;
    Ok(Collected {
        package: package.clone(),
        strip: !debuginfo.enabled && !target.contains("windows-msvc"),
        target,
        executables,
    })
}

// Bundle collected executables into `output/<package>.dokime`.
pub async fn package(collected: Collected, output: &Path) -> Result<PathBuf, Error> {
    let destination = output.join(format!("{}.dokime", collected.package.name));
    bundle(&collected.package, &collected.target, collected.executables, destination, collected.strip).await
}

/* === Private Functions === */

async fn metadata(args: &Options, dependencies: bool) -> Result<cargo_metadata::Metadata, Error> {
    let cargo_timeout = args.cargo_timeout;
    let cargo_options = args.cargo_options();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut metadata = MetadataCommand::new();
    metadata.cargo_path(&cargo).other_options(cargo_options.clone());
    if !dependencies {
        metadata.no_deps();
    }
    if let Some(path) = &args.manifest_path {
        metadata.manifest_path(path);
    }
    let metadata = process::cargo(
        metadata.cargo_command(),
        if dependencies { "Resolving dependencies" } else { "Resolving package metadata" },
        cargo_timeout,
        false,
    )
    .await?;
    Ok(serde_json::from_slice(&metadata.stdout)?)
}

async fn build_resolved(args: Options, metadata: cargo_metadata::Metadata) -> Result<PathBuf, Error> {
    let cargo_timeout = args.cargo_timeout;
    let cargo_options = args.cargo_options();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let binaries = args.binaries.as_slice();
    let package = select_package(&metadata, args.package.as_deref())?;
    let root = package.manifest_path.parent().expect("Cargo manifest has a parent");
    let selected = selected_binaries(package, binaries)?;
    let host = host_target(package).await?;
    let target = args.target.as_deref().unwrap_or(&host);
    if target != host {
        return Err(Error::Invalid(format!("native binaries require the native host target {host}")));
    }
    let target_dir = args.target_dir.unwrap_or_else(|| metadata.target_directory.as_std_path().to_owned());
    std::fs::create_dir_all(&target_dir)?;
    // Lexical absolutization only: canonicalize yields a verbatim `\\?\` path
    // on Windows, which the MSVC linker rejects for build-script objects.
    let target_dir = std::path::absolute(target_dir)?;
    let mut command = dokime::domain::process::command(&cargo);
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
    if let Some(profile) = &args.profile {
        command.args(["--profile", profile]);
    }
    let debuginfo = DebugInfo::of(package);
    let terminal = dokime::terminal::Terminal::default();
    terminal.status(format!("Building {} ({}) for {target}", package.name, selected.join(", ")));
    terminal.field("Debug info", debuginfo.summary());
    let output = process::cargo(command, "Cargo compilation", cargo_timeout, true).await?;
    let executables = selected_executables(output.stdout.as_slice(), &package.id.repr, &package.name, &selected, debuginfo)?;
    let output = args.output.unwrap_or_else(|| target_dir.join("dokime").join(format!("{}-{}.dokime", package.name, package.version)));
    bundle(package, target, executables, output, !debuginfo.enabled && !target.contains("windows-msvc")).await
}

fn selected_binaries(package: &Package, binaries: &[String]) -> Result<Vec<String>, Error> {
    let bins = package.targets.iter().filter(|target| target.kind.contains(&cargo_metadata::TargetKind::Bin)).collect::<Vec<_>>();
    let selected = if binaries.is_empty() {
        vec![
            package
                .default_run
                .as_deref()
                .or_else(|| (bins.len() == 1).then(|| bins[0].name.as_str()))
                .ok_or_else(|| {
                    Error::Invalid("select one native binary with --bin (repeat for multiple files), or set package.default-run".into())
                })?
                .to_owned(),
        ]
    } else {
        binaries.to_vec()
    };
    if selected.len() > Limits::default().files {
        return Err(Error::Invalid("too many selected executable files".into()));
    }
    let mut requested = std::collections::BTreeSet::new();
    for name in &selected {
        if !requested.insert(name) {
            return Err(Error::Invalid(format!("binary {name} was selected twice")));
        }
        if !bins.iter().any(|bin| bin.name == *name) {
            return Err(Error::Invalid(format!("package {} has no binary {name}", package.name)));
        }
    }
    Ok(selected)
}

fn selected_executables(
    reader: impl std::io::BufRead, package_id: &str, producer: &str, names: &[String], debug: DebugInfo,
) -> Result<Executables, Error> {
    let selected = BTreeMap::from([(package_id.to_owned(), producer.to_owned())]);
    let mut executables = harmos_packaging::cargo::collect(reader, &selected)
        .and_then(|outputs| outputs.executables(producer, names))
        .map_err(|error| Error::Invalid(error.to_string()))?;
    if !debug.enabled {
        for (_, symbols) in executables.values_mut() {
            *symbols = None;
        }
    }
    Ok(executables)
}

async fn native<T: Send + 'static>(
    name: &'static str, work: impl FnOnce(harmos_packaging::BuildCancellation) -> Result<T, harmos_packaging::Error> + Send + 'static,
) -> Result<T, Error> {
    struct Cancel(harmos_packaging::BuildCancellation);
    impl Drop for Cancel {
        fn drop(&mut self) {
            self.0.cancel();
        }
    }
    let cancellation = harmos_packaging::BuildCancellation::default();
    let guard = Cancel(cancellation.clone());
    let mut task = tokio::task::spawn_blocking(move || work(cancellation));
    let result = tokio::select! {
        result = &mut task => result,
        _ = tokio::signal::ctrl_c() => {
            guard.0.cancel();
            let _ = task.await;
            return Err(Error::Cancelled);
        }
    }
    .map_err(|error| Error::Io(std::io::Error::other(error)))?;
    result.map_err(|error| match error {
        harmos_packaging::Error::Cancelled => Error::Cancelled,
        harmos_packaging::Error::Timeout => Error::Timeout(name.into()),
        error => Error::Invalid(error.to_string()),
    })
}

fn tool_settings(timeout_ms: u64, stdout_bytes: usize, stderr_bytes: usize) -> harmos_packaging::ProcessSettings {
    harmos_packaging::ProcessSettings {
        timeout: Some(Duration::from_millis(timeout_ms)),
        stdout_bytes,
        stderr_bytes,
    }
}

async fn host_target(package: &Package) -> Result<String, Error> {
    let root = package.manifest_path.parent().expect("Cargo manifest has a parent").as_std_path().to_owned();
    native("rustc host probe", move |cancel| {
        harmos_packaging::native::host_target(
            Some(&root),
            tool_settings(PROBE_TIMEOUT_MS, PROBE_OUTPUT_BYTES as usize, PROBE_OUTPUT_BYTES as usize),
            &cancel,
        )
    })
    .await
}

async fn strip(executable: &Path, directory: &Path, target: &str) -> Result<PathBuf, Error> {
    let path = executable.to_owned();
    let directory = directory.to_owned();
    let target = target.to_owned();
    native("strip debug sections", move |cancel| {
        harmos_packaging::native::strip_debug(
            &path,
            &directory,
            &target,
            tool_settings(STRIP_TIMEOUT_MS, PROBE_OUTPUT_BYTES as usize, PROBE_OUTPUT_BYTES as usize),
            &cancel,
        )
    })
    .await
}

async fn bundle(package: &Package, target: &str, executables: Executables, output: PathBuf, strip_debug: bool) -> Result<PathBuf, Error> {
    let stripped = tempfile::tempdir()?;
    let limits = Limits::default();
    let root = package.manifest_path.parent().expect("Cargo manifest has a parent");
    let terminal = dokime::terminal::Terminal::default();
    terminal.status("Collecting plugin declarations and assets");
    let collection_started = Instant::now();
    let mut inputs = Vec::new();
    let mut total = 0;
    let mut catalog_remaining = limits.manifest_bytes;
    let requirements = package
        .dependencies
        .iter()
        .filter(|dependency| dependency.name == "dokime")
        .map(|dependency| dependency.req.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    if requirements.len() != 1 {
        return Err(Error::Invalid("plugin must declare one Dokime dependency requirement".into()));
    }
    let requirement = requirements.first().expect("one requirement");
    let requires_dokime = requirement.parse().map_err(|error: semver::Error| Error::Invalid(error.to_string()))?;
    let compatibility = Compatibility { schema: 1, requires_dokime };
    let bytes = serde_json::to_vec(&compatibility)?;
    total += bytes.len();
    inputs.push((COMPATIBILITY_FILE.to_owned(), bytes, None));

    for (path, pdb) in executables.values() {
        let path = if strip_debug { strip(path, stripped.path(), target).await? } else { path.clone() };
        let (name, binary, executable) = collect_executable(
            &path,
            package,
            target,
            limits.file_bytes.min(limits.total_file_bytes.saturating_sub(total)),
            &mut catalog_remaining,
        )
        .await?;
        total += binary.len();
        inputs.push((name, binary, Some(executable)));
        let pdb = match pdb {
            Some(pdb) => symbols(pdb, target, limits.file_bytes.min(limits.total_file_bytes.saturating_sub(total)), &terminal)?,
            None => None,
        };
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
    for (name, path) in files::assets(&root.join("assets").into_std_path_buf(), limits.files - inputs.len())? {
        let bytes = files::read(&path, limits.file_bytes.min(limits.total_file_bytes.saturating_sub(total)))?;
        total += bytes.len();
        inputs.push((name, bytes, None));
    }
    terminal.success(format!("Plugin declarations and assets collected in {:.1}s", collection_started.elapsed().as_secs_f64()));
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
    let bundle = Bundle::build_with_settings(package_identity(package), &sources, limits, &dokime::config::schema_defaults())?;
    let parent = output.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
    std::fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    use std::io::Write;
    temporary.write_all(bundle.bytes())?;
    temporary.persist(&output).map_err(|error| Error::Io(error.error))?;
    terminal.success(format!(
        "Plugin bundle ready · {} in {:.1}s",
        dokime::terminal::size_label(bundle.bytes().len() as u64),
        bundle_started.elapsed().as_secs_f64()
    ));
    Ok(output)
}

// Debug info is optional: a symbols file over the remaining budget is left out
// with a warning, and the bundle ships without symbols for that executable.
fn symbols(
    pdb: &std::path::Path, target: &str, limit: usize, terminal: &dokime::terminal::Terminal,
) -> Result<Option<(String, Vec<u8>)>, Error> {
    let pdb_name =
        pdb.file_name().and_then(|name| name.to_str()).ok_or_else(|| Error::Invalid("pdb filename must be UTF-8".into()))?.to_owned();
    match files::read(pdb, limit) {
        Ok(bytes) => Ok(Some((format!("bin/{target}/{pdb_name}"), bytes))),
        Err(Error::Output(..)) => {
            let size = std::fs::metadata(pdb)?.len();
            terminal.warn(format!(
                "{} is {}, over the {} limit; the bundle ships without its debug symbols",
                pdb.display(),
                dokime::terminal::size_label(size),
                dokime::terminal::size_label(limit as u64)
            ));
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

async fn collect_executable(
    path: &std::path::Path, package: &Package, target: &str, max_bytes: usize, catalog_remaining: &mut usize,
) -> Result<(String, Vec<u8>, Executable), Error> {
    let binary = files::read(path, max_bytes)?;
    let declaration = Declaration::inspect_with_settings(
        &binary,
        dokime::harmos::plugin::inventory::Facility::Sidecar,
        &dokime::config::schema_defaults(),
    )
    .map_err(|error| {
        Error::Invalid(format!("plugin `{}` binary `{}` has no complete native declaration: {error}", package.name, path.display()))
    })?;
    if declaration.version.to_string() != package.version.to_string() {
        return Err(Error::Invalid(format!(
            "executable version {} differs from Cargo package version {}; the executable declaration must derive its version from the selected Cargo package",
            declaration.version, package.version
        )));
    }
    let artifact = harmos_packaging::Artifact::read(
        package.name.to_string(),
        path.file_stem().and_then(|name| name.to_str()).unwrap_or("plugin").to_owned(),
        path.to_owned(),
    )
    .map_err(|error| Error::Invalid(error.to_string()))?;
    let root = package.manifest_path.parent().expect("Cargo manifest has a parent").as_std_path().to_owned();
    let stdout_bytes = CATALOG_EXPORT_LIMIT.min(*catalog_remaining);
    let exported = native("plugin catalog export", move |cancel| {
        harmos_packaging::native::catalog(
            &artifact,
            Some(&root),
            tool_settings(CATALOG_EXPORT_TIMEOUT_MS, stdout_bytes, CATALOG_STDERR_BYTES as usize),
            &cancel,
        )
    })
    .await?;
    *catalog_remaining = catalog_remaining.saturating_sub(exported.output_bytes);
    let registrations = exported.registrations;
    if files::read(path, max_bytes)? != binary {
        return Err(Error::Invalid("the executable changed during catalog export".into()));
    }
    let binary_name =
        path.file_name().and_then(|name| name.to_str()).ok_or_else(|| Error::Invalid("executable filename must be UTF-8".into()))?;
    Ok((
        format!("bin/{target}/{binary_name}"),
        binary,
        Executable {
            facility: Facility::Sidecar,
            target: target.to_owned(),
            registrations,
        },
    ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::plugin::inventory::contracts;

    #[test]
    fn opt_out_ships_no_symbols() {
        let messages = format!(
            "{}\n{}",
            serde_json::json!({"reason":"compiler-artifact","package_id":"fixture",
            "target":{"name":"fixture","kind":["bin"]},"profile":{"test":false},"executable":"fixture.exe","filenames":["fixture.exe","deps/fixture.pdb"]}),
            serde_json::json!({"reason":"build-finished","success":true})
        );
        for enabled in [false, true] {
            let outputs =
                selected_executables(messages.as_bytes(), "fixture", "fixture", &["fixture".into()], DebugInfo { enabled }).unwrap();
            assert_eq!(outputs["fixture"].1, enabled.then(|| PathBuf::from("deps/fixture.pdb")));
        }
    }

    #[test]
    fn oversized_symbols_are_skipped() {
        let root = tempfile::tempdir().unwrap();
        let pdb = root.path().join("plugin.pdb");
        std::fs::write(&pdb, vec![0xAB; 2048]).unwrap();
        let terminal = dokime::terminal::Terminal::default();
        let target = "x86_64-pc-windows-msvc";

        // Over The Limit The Symbols Are Left Out; Within It They Ship
        assert_eq!(symbols(&pdb, target, 1024, &terminal).unwrap(), None);
        let kept = symbols(&pdb, target, 4096, &terminal).unwrap().unwrap();
        assert_eq!(kept.0, format!("bin/{target}/plugin.pdb"));

        // The Bundle Still Builds From The Executable Alone
        let executable = inert_executable();
        let sources = [Source {
            path: "bin/x86_64-pc-windows-msvc/fixture.exe",
            bytes: executable.as_bytes(),
            executable: Some(Executable {
                facility: Facility::Sidecar,
                target: target.into(),
                registrations: vec![],
            }),
        }];
        let identity = Identity {
            name: "fixture".into(),
            version: "1.0.0".parse().unwrap(),
            description: "A fixture.".into(),
            package: PackageMetadata::default(),
        };
        assert!(Bundle::build(identity, &sources, Limits::default()).is_ok());
    }

    // A non-running executable carrying the minimal native declaration.
    fn inert_executable() -> String {
        const OWNER: &str = "fixture";
        const POLICY: dokime::harmos::operations::OperationPolicy = dokime::harmos::operations::OperationPolicy {
            timeout: Duration::from_secs(5),
            attempt_timeout: Duration::from_secs(5),
            cancellation_timeout: std::time::Duration::ZERO,
            retries: dokime::harmos::operations::RetryPolicy::Never,
        };
        const SIZE: usize = contracts::encoded_fragments_len("sidecar", OWNER, &[], &[], &[POLICY; 2]);
        const CONTRACTS: [u8; SIZE] = contracts::encode_fragments("sidecar", OWNER, &[], &[], &[POLICY; 2]);
        let owner = OWNER;
        let sections = [
            serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"A fixture.","config":"()"}),
            serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}}),
            serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]}),
            serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]}),
        ];
        let prefix = std::str::from_utf8(dokime::harmos::artifacts::PREFIX).unwrap();

        let executable = String::from_utf8(CONTRACTS.to_vec()).unwrap();
        sections.iter().fold(executable, |executable, section| format!("{executable}{prefix}{section}\0"))
    }
}
