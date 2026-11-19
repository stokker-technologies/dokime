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
use std::env;
use std::fs;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

// Third Party Imports
use base64::{Engine, engine::general_purpose::STANDARD};

// Local Imports
use super::Error;
use super::{distribution::Distribution, manifest::Manifest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn authenticode(path: &Path) -> Result<bool, Error> {
    let provider = env::var("DOKIME_AUTHENTICODE_PROVIDER").ok();
    authenticode_with_provider(path, provider.as_deref())
}

pub(crate) fn authenticode_enabled(provider: Option<&str>) -> Result<bool, Error> {
    match provider {
        None | Some("" | "none") => Ok(false),
        Some("azure" | "pfx") => Ok(true),
        Some(_) => Err(Error::Invalid("DOKIME_AUTHENTICODE_PROVIDER must be azure, pfx or none".into())),
    }
}

pub fn updater(manifest: &Manifest, distribution: &Distribution, package: &Path, output: &Path) -> Result<(), Error> {
    let Some(settings) = manifest.signing.as_ref().and_then(|s| s.updater.as_ref()) else {
        return Ok(());
    };
    let scratch = tempfile::tempdir()?;
    let key = scratch.path().join("updater.key");
    let mut sources = 0;
    let mut material = None;
    if let Some(path) = &settings.private_key_file {
        sources += 1;
        material = Some(fs::read(path)?);
    }
    if let Some(name) = &settings.private_key_path_env {
        sources += 1;
        let path = env::var_os(name).ok_or_else(|| Error::Invalid("updater key path environment is missing".into()))?;
        material = Some(fs::read(path)?);
    }
    if let Some(name) = &settings.private_key_env {
        sources += 1;
        let encoded = env::var(name).map_err(|_| Error::Invalid("updater private key environment is missing".into()))?;
        material = Some(STANDARD.decode(encoded.trim()).map_err(|_| Error::Invalid("updater private key must be base64".into()))?);
    }
    if sources != 1 {
        return Err(Error::Invalid("updater signing requires exactly one private key file/environment reference".into()));
    }
    let material = material.ok_or_else(|| Error::Invalid("updater key is missing".into()))?;
    let material = if material.starts_with(b"untrusted comment:") {
        material
    } else {
        STANDARD
            .decode(material.iter().copied().filter(|byte| !byte.is_ascii_whitespace()).collect::<Vec<_>>())
            .map_err(|_| Error::Invalid("updater key file must contain a minisign secret key or its base64 encoding".into()))?
    };
    fs::write(&key, material)?;
    let signature = scratch.path().join("installer.minisig");
    let password = env::var(&settings.password_env).map_err(|_| Error::Invalid("updater password environment is missing".into()))?;
    let mut child = Command::new("minisign")
        .args(["-S", "-q", "-s"])
        .arg(&key)
        .arg("-m")
        .arg(package)
        .arg("-x")
        .arg(&signature)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| Error::Invalid("updater signing requires minisign".into()))?;
    let mut input = child.stdin.take().ok_or_else(|| Error::Invalid("updater signing input unavailable".into()))?;
    input.write_all(password.as_bytes())?;
    input.write_all(b"\n")?;
    drop(input);
    if !child.wait()?.success() {
        return Err(Error::Invalid("updater signing failed".into()));
    }
    let public_key = scratch.path().join("updater.pub");
    fs::write(
        &public_key,
        STANDARD
            .decode(distribution.identity.updater_public_key.trim())
            .map_err(|_| Error::Invalid("compiled updater public key is invalid".into()))?,
    )?;
    run(
        Command::new("minisign").args(["-V", "-q", "-m"]).arg(package).arg("-p").arg(&public_key).arg("-x").arg(&signature),
        "updater signature does not match compiled application trust",
    )?;
    let signature = STANDARD.encode(fs::read(signature)?);
    fs::write(package.with_extension("exe.sig"), format!("{signature}\n"))?;
    let filename =
        package.file_name().and_then(|s| s.to_str()).ok_or_else(|| Error::Invalid("installer output filename is invalid".into()))?;
    let published = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| Error::Invalid(e.to_string()))?;
    let url = format!("{}/{}", settings.artifact_base_url.trim_end_matches('/'), filename);
    let metadata = serde_json::json!({
        "version":manifest.installer.version,
        "pub_date":published,
        "platforms":{
            "windows-x86_64":{
                "signature":signature,
                "url":url,
            },
        },
    });
    fs::write(output.join("release.json"), serde_json::to_vec_pretty(&metadata)?)?;
    scratch.close()?;
    Ok(())
}

pub(crate) fn run(command: &mut Command, requirement: &str) -> Result<(), Error> {
    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| Error::Invalid(requirement.into()))?;
    if !status.success() {
        return Err(Error::Invalid(format!("{requirement}; tool failed with {status}")));
    }
    Ok(())
}

pub(crate) fn verify_rpm_signature(package: &Path) -> Result<(), Error> {
    let result = Command::new("rpmkeys")
        .arg("--checksig")
        .arg(package)
        .env("LC_ALL", "C")
        .output()
        .map_err(|_| Error::Invalid("RPM signature verification requires rpmkeys".into()))?;
    let output = String::from_utf8_lossy(&result.stdout).to_ascii_lowercase();
    if !result.status.success() || !output.contains("signatures ok") {
        return Err(Error::Invalid("RPM has no trusted valid signature".into()));
    }
    Ok(())
}

/* === Private Functions === */

fn authenticode_with_provider(path: &Path, provider: Option<&str>) -> Result<bool, Error> {
    if !authenticode_enabled(provider)? {
        eprintln!("Authenticode skipped: no Windows signing provider configured.");
        return Ok(false);
    }
    if !cfg!(target_os = "windows") {
        return Err(Error::Invalid("Authenticode requires native Windows".into()));
    }
    let scratch = tempfile::tempdir()?;
    let script = scratch.path().join("authenticode.ps1");
    fs::write(&script, include_str!("authenticode.ps1"))?;
    let status = Command::new("pwsh")
        .args(["-NoProfile", "-NonInteractive", "-File"])
        .arg(&script)
        .args(["-Operation", "Sign", "-Path"])
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(Error::Invalid(format!("Authenticode signing or verification failed: {status}")));
    }
    scratch.close()?;
    Ok(true)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skipped_authenticode_never_opens_the_input() {
        for provider in [None, Some(""), Some("none")] {
            assert!(!authenticode_with_provider(Path::new("absent-installer.exe"), provider).unwrap());
        }
        assert!(authenticode_with_provider(Path::new("absent-installer.exe"), Some("unknown")).is_err());
    }

    #[test]
    fn authenticode_provider_selection() {
        for provider in ["azure", "pfx"] {
            assert!(authenticode_enabled(Some(provider)).unwrap());
        }
    }
}
