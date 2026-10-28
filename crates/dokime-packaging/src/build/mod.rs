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
    process::Command,
    time::{Duration, Instant},
};

// Third Party Imports
use cargo_metadata::{Message, MetadataCommand, Package};
use dokime::harmos::plugin::inventory::Declaration;
use dokime::harmos::sidecar::{CATALOG_EXPORT_FLAG, CATALOG_EXPORT_LIMIT, Registration};
use harmos_packaging::bundle::{Bundle, Executable, Facility, Identity, Limits, PackageMetadata, Source};

// Local Imports
use crate::{Error, bundle::files, process};

/* === Constants === */

const PROBE_TIMEOUT_MS: u64 = 30_000;
const PROBE_OUTPUT_BYTES: u64 = 65_536;
const STRIP_TIMEOUT_MS: u64 = 120_000;
const CATALOG_EXPORT_TIMEOUT_MS: u64 = 10_000;
const CATALOG_STDERR_BYTES: u64 = 262_144;

/* === Macros === */

/* === Type Aliases === */

type Executables = BTreeMap<String, Vec<(PathBuf, Option<PathBuf>)>>;

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

    // Cargo reports every emitted output for the unit, debug info included: on
    // Windows MSVC that is the `.pdb` beside the hash-named executable in
    // `deps/`, not beside the copy `executable` names. Disabled debug info
    // ships no symbols even when the linker still writes one.
    fn symbols(self, filenames: &[cargo_metadata::camino::Utf8PathBuf]) -> Option<PathBuf> {
        if !self.enabled {
            return None;
        }
        filenames
            .iter()
            .find(|path| path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("pdb")))
            .map(|path| path.clone().into_std_path_buf())
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
    let mut executables = Executables::new();
    for message in Message::parse_stream(reader) {
        if let Message::CompilerArtifact(artifact) = message?
            && artifact.package_id == package.id
            && !artifact.profile.test
            && selected.contains(&artifact.target.name)
            && artifact.target.kind.contains(&cargo_metadata::TargetKind::Bin)
        {
            let pdb = debuginfo.symbols(&artifact.filenames);
            executables
                .entry(artifact.target.name)
                .or_default()
                .extend(artifact.executable.into_iter().map(|path| (path.into_std_path_buf(), pdb.clone())));
        }
    }
    let executables = single(executables, selected.len())?;
    Ok(Collected {
        package: package.clone(),
        strip: !debuginfo.enabled && !target.contains("windows-msvc"),
        target,
        executables: executables.into_iter().map(|(name, mut paths)| (name, paths.remove(0))).collect(),
    })
}

// Bundle collected executables into `output/<package>.dokime`.
pub async fn package(collected: Collected, output: &Path) -> Result<PathBuf, Error> {
    let destination = output.join(format!("{}.dokime", collected.package.name));
    let mut executables = Executables::new();
    for (name, (path, symbols)) in collected.executables {
        executables.insert(name, vec![(path, symbols)]);
    }
    bundle(&collected.package, &collected.target, executables, destination, collected.strip).await
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
    let requested = selected.iter().collect::<std::collections::BTreeSet<_>>();
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
    if let Some(profile) = &args.profile {
        command.args(["--profile", profile]);
    }
    let debuginfo = DebugInfo::of(package);
    let terminal = dokime::terminal::Terminal::default();
    terminal.status(format!("Building {} ({}) for {target}", package.name, selected.join(", ")));
    terminal.field("Debug info", debuginfo.summary());
    let output = process::cargo(command, "Cargo compilation", cargo_timeout, true).await?;
    let mut executables = std::collections::BTreeMap::<_, Vec<(PathBuf, Option<PathBuf>)>>::new();
    let mut finished = false;
    for message in Message::parse_stream(output.stdout.as_slice()) {
        match message? {
            Message::CompilerArtifact(artifact)
                if artifact.package_id == package.id
                    && requested.contains(&artifact.target.name)
                    && artifact.target.kind.contains(&cargo_metadata::TargetKind::Bin) =>
            {
                let pdb = debuginfo.symbols(&artifact.filenames);
                executables
                    .entry(artifact.target.name)
                    .or_default()
                    .extend(artifact.executable.into_iter().map(|path| (path.into_std_path_buf(), pdb.clone())));
            }
            Message::BuildFinished(result) => finished = result.success,
            _ => {}
        }
    }
    if !finished {
        return Err(Error::Invalid("Cargo did not report exactly one successful executable for each selected binary".into()));
    }
    let executables = single(executables, selected.len())?;
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

async fn host_target(package: &Package) -> Result<String, Error> {
    let root = package.manifest_path.parent().expect("Cargo manifest has a parent");
    let mut probe = Command::new("rustc");
    probe.current_dir(root).arg("-vV");
    let timeout = Some(Duration::from_millis(PROBE_TIMEOUT_MS));
    let output_bytes = PROBE_OUTPUT_BYTES as usize;
    let host = process::run(probe, "rustc host probe", timeout, output_bytes, output_bytes).await?;
    String::from_utf8_lossy(&host.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_owned)
        .ok_or_else(|| Error::Invalid("rustc did not report a host target".into()))
}

// A copy of `executable` without debug sections, written with the toolchain's own llvm-objcopy.
async fn strip(executable: &Path, directory: &Path, target: &str) -> Result<PathBuf, Error> {
    let mut sysroot = Command::new("rustc");
    sysroot.arg("--print").arg("sysroot");
    let timeout = Some(Duration::from_millis(PROBE_TIMEOUT_MS));
    let output_bytes = PROBE_OUTPUT_BYTES as usize;
    let sysroot = process::run(sysroot, "rustc sysroot probe", timeout, output_bytes, output_bytes).await?;
    let sysroot = PathBuf::from(String::from_utf8_lossy(&sysroot.stdout).trim());
    let objcopy =
        sysroot.join("lib/rustlib").join(target).join("bin").join(if cfg!(windows) { "llvm-objcopy.exe" } else { "llvm-objcopy" });
    if !objcopy.is_file() {
        return Err(Error::Invalid(format!("{} is missing; install the llvm-tools-preview component", objcopy.display())));
    }
    let name = executable.file_name().ok_or_else(|| Error::Invalid("executable has no file name".into()))?;
    let stripped = directory.join(name);
    let mut command = Command::new(objcopy);
    command.arg("--strip-debug").arg(executable).arg(&stripped);
    let timeout = Some(Duration::from_millis(STRIP_TIMEOUT_MS));
    process::run(command, "strip debug sections", timeout, output_bytes, output_bytes).await?;
    Ok(stripped)
}

fn single(mut executables: Executables, selected: usize) -> Result<Executables, Error> {
    for paths in executables.values_mut() {
        paths.sort();
        paths.dedup();
    }
    if executables.len() != selected || executables.values().any(|paths| paths.len() != 1) {
        return Err(Error::Invalid("Cargo did not report exactly one successful executable for each selected binary".into()));
    }
    Ok(executables)
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
    for paths in executables.values() {
        let (path, pdb) = &paths[0];
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
    let bundle = Bundle::build(package_identity(package), &sources, limits)?;
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
    let declaration = Declaration::inspect(&binary, dokime::harmos::plugin::inventory::Facility::Sidecar)
        .map_err(|error| Error::Invalid(format!("selected binary has no complete native declaration: {error}")))?;
    if declaration.version.to_string() != package.version.to_string() {
        return Err(Error::Invalid(format!(
            "executable version {} differs from Cargo package version {}; the executable declaration must derive its version from the selected Cargo package",
            declaration.version, package.version
        )));
    }
    let mut command = Command::new(path);
    command.current_dir(package.manifest_path.parent().expect("Cargo manifest has a parent")).arg(CATALOG_EXPORT_FLAG);
    let export = process::run(
        command,
        "plugin catalog export",
        Some(Duration::from_millis(CATALOG_EXPORT_TIMEOUT_MS)),
        CATALOG_EXPORT_LIMIT.min(*catalog_remaining),
        CATALOG_STDERR_BYTES as usize,
    )
    .await?;
    *catalog_remaining = catalog_remaining.saturating_sub(export.stdout.len());
    let registrations: Vec<Registration> = serde_json::from_slice(&export.stdout)?;
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

    #[test]
    fn opt_out_ships_no_symbols() {
        let disabled = DebugInfo { enabled: false };
        let outputs = vec![cargo_metadata::camino::Utf8PathBuf::from("deps/plugin.pdb")];

        assert_eq!(disabled.symbols(&outputs), None);
        assert_eq!(DebugInfo { enabled: true }.symbols(&outputs), Some(PathBuf::from("deps/plugin.pdb")));
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
        let owner = "fixture";
        let policy = dokime::harmos::operations::OperationPolicy {
            timeout: Duration::from_secs(5),
            attempt_timeout: Duration::from_secs(5),
            retries: dokime::harmos::operations::RetryPolicy::Never,
            restart: dokime::harmos::operations::RestartPolicy::Never,
        };
        let lifecycle =
            [dokime::harmos::operations::lifecycle::Action::Initialize, dokime::harmos::operations::lifecycle::Action::Shutdown]
                .map(|action| dokime::harmos::operations::lifecycle::Descriptor::new(action, policy).unwrap());
        let sections = [
            serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"A fixture.","config":"()"}),
            serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"provides":[],"requires":[],"lifecycle":lifecycle}),
            serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}}),
            serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]}),
            serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]}),
        ];
        let prefix = std::str::from_utf8(dokime::harmos::artifacts::PREFIX).unwrap();

        sections.iter().fold("inert".to_owned(), |executable, section| format!("{executable}{prefix}{section}\0"))
    }
}
