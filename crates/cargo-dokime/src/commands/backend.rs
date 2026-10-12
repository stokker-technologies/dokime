/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
// Third Party Imports
use semver::Version;
// Local Imports
use super::installation::{Installation, Origin};
use crate::error::Error;
use dokime_packaging::process;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(serde::Deserialize)]
pub(super) struct Identity {
    pub version: Version,
    pub sdk_revision: Option<String>,
    pub dev_protocol: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn resolve(explicit: Option<&Path>) -> Result<Installation, Error> {
    let Some(path) = explicit else {
        return super::installation::discover();
    };
    let search =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect::<Vec<_>>();
    let executable = override_from(path, &search)?;
    Ok(Installation::new(executable, Origin::Explicit))
}

pub(super) async fn inspect(executable: &Path) -> Result<Identity, Error> {
    let mut command = Command::new(executable);
    command.args(["version", "--json"]);
    let output = process::run(
        command,
        "Dokime version probe",
        Some(Duration::from_secs(10)),
        16 * 1024,
        64 * 1024,
    )
    .await
    .map_err(|error| {
        Error::Invalid(format!(
            "cannot inspect host {}: {error}. Update Dokime or choose a matching --dokime PATH",
            executable.display()
        ))
    })?;
    let identity: Identity = serde_json::from_slice(&output.stdout).map_err(|error| {
        Error::Invalid(format!(
            "host {} returned invalid version information: {error}; update Dokime",
            executable.display()
        ))
    })?;
    if identity.dev_protocol != dokime_protocol::DEV_PROTOCOL {
        return Err(Error::Invalid(format!(
            "host {} uses development protocol {}; this cargo-dokime requires {}. Update the host and Cargo extension together",
            executable.display(),
            identity.dev_protocol,
            dokime_protocol::DEV_PROTOCOL
        )));
    }
    compatible(
        &identity,
        &Version::parse(env!("CARGO_PKG_VERSION")).expect("Cargo version"),
    )?;
    Ok(identity)
}

pub(super) fn compatible(host: &Identity, sdk: &Version) -> Result<(), Error> {
    let version = &host.version;
    let same_api = version.major == sdk.major
        && (version.major != 0 || version.minor == sdk.minor)
        && (version.major != 0 || version.minor != 0 || version.patch == sdk.patch)
        && version.pre == sdk.pre;
    if !same_api {
        return Err(Error::Invalid(format!(
            "Dokime SDK/tool version {sdk} is incompatible with host {version}. Update the host or select an SDK release in the same compatibility series"
        )));
    }
    Ok(())
}

/* === Private Functions === */
fn override_from(path: &Path, search: &[PathBuf]) -> Result<PathBuf, Error> {
    file(&override_candidate(path, search, Path::is_file)?)
}

fn override_candidate(
    path: &Path,
    search: &[PathBuf],
    mut is_file: impl FnMut(&Path) -> bool,
) -> Result<PathBuf, Error> {
    if path.is_absolute() || path.components().count() > 1 || is_file(path) {
        return Ok(path.to_owned());
    }
    for directory in search {
        let candidate = directory.join(path);
        if is_file(&candidate) {
            return Ok(candidate);
        }
        #[cfg(windows)]
        if path.extension().is_none() {
            let candidate = candidate.with_extension("exe");
            if is_file(&candidate) {
                return Ok(candidate);
            }
        }
    }
    Err(Error::Invalid(format!(
        "host override {} was not found; use --dokime with its executable path",
        path.display()
    )))
}

fn file(path: &Path) -> Result<PathBuf, Error> {
    if !path.is_file() {
        return Err(Error::Invalid(format!(
            "host executable {} is not a file",
            path.display()
        )));
    }
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        // Keep the selected path spelling, including a package-manager shim
        // or multicall symlink. Canonicalizing here can change argv[0] and
        // therefore the executable's dispatch behavior.
        std::path::absolute(path)?
    };
    let metadata = fs::metadata(&path)?;
    if !metadata.is_file() {
        return Err(Error::Invalid(format!(
            "host executable {} is not a regular file",
            path.display()
        )));
    }
    Ok(path)
}

/* === Tests === */
