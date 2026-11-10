/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Locate the Pinned Protobuf Compiler for Export Verification.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitStatus,
};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn compiler(root: &Path) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    // Archived Tests Consume the Transferred Compiler, Not the Producer's Registry.
    let (compiler, includes) = if let Ok(profile) = std::env::var("STKR_TEST_PROFILE") {
        let target = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        let vendor = root.join("target/build").join(profile).join(target).join("fixtures/inputs/vendor");
        let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => "linux-x86_64",
            ("linux", "aarch64") => "linux-aarch_64",
            ("macos", "x86_64") => "macos-x86_64",
            ("macos", "aarch64") => "macos-aarch_64",
            ("windows", _) => "win32",
            other => panic!("no vendored protoc for {other:?}"),
        };
        let prefix = format!("protoc-bin-vendored-{platform}-");
        let package = fs::read_dir(vendor)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .find(|path| path.file_name().unwrap().to_string_lossy().starts_with(&prefix))
            .expect("the transferred vendor inputs contain the pinned protobuf compiler");
        let executable = if cfg!(windows) { "protoc.exe" } else { "protoc" };
        (package.join("bin").join(executable), package.join("include"))
    } else {
        (protoc_bin_vendored::protoc_bin_path()?, protoc_bin_vendored::include_path()?)
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&compiler)?.permissions();
        permissions.set_mode(permissions.mode() | 0o100);
        fs::set_permissions(&compiler, permissions)?;
    }

    Ok((compiler, includes))
}

pub fn files(directory: &Path) -> std::io::Result<Vec<String>> {
    let mut pending = vec![directory.to_owned()];
    let mut files = Vec::new();
    while let Some(parent) = pending.pop() {
        for entry in fs::read_dir(parent)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            files.push(path.strip_prefix(directory).expect("descendant path").to_string_lossy().replace('\\', "/"));
        }
    }
    files.sort();
    Ok(files)
}

pub fn parse(directory: &Path, files: &[&str], descriptor: &Path) -> Result<ExitStatus, Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (compiler, includes) = compiler(&root)?;
    let status = dokime::domain::process::command(compiler)
        .arg(format!("--proto_path={}", directory.display()))
        .arg(format!("--proto_path={}", includes.display()))
        .arg(format!("--descriptor_set_out={}", descriptor.display()))
        .arg("--include_imports")
        .args(files)
        .status()?;
    Ok(status)
}

/* === Private Functions === */

/* === Tests === */
