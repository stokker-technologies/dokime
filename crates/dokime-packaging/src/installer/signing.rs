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
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

// Third Party Imports
use base64::{Engine, engine::general_purpose::STANDARD};

// Local Imports
use super::{inspect, manifest::Manifest, payload::Payload};
use crate::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn updater(manifest: &Manifest, payload: &Payload, package: &Path, output: &Path) -> Result<(), Error> {
    let Some(settings) = manifest.signing.as_ref().and_then(|s| s.updater.as_ref()) else {
        return Ok(());
    };
    let scratch = tempfile::tempdir()?;
    let key = scratch.path().join("updater.key");
    let mut sources = 0;
    let mut material = None;
    if let Some(path) = &settings.private_key_file {
        sources += 1;
        material = Some(std::fs::read(path)?);
    }
    if let Some(name) = &settings.private_key_path_env {
        sources += 1;
        let path = std::env::var_os(name).ok_or_else(|| Error::Invalid("updater key path environment is missing".into()))?;
        material = Some(std::fs::read(path)?);
    }
    if let Some(name) = &settings.private_key_env {
        sources += 1;
        let encoded = std::env::var(name).map_err(|_| Error::Invalid("updater private key environment is missing".into()))?;
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
    std::fs::write(&key, material)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o600))?;
    }
    let signature = scratch.path().join("installer.minisig");
    let password = std::env::var(&settings.password_env).map_err(|_| Error::Invalid("updater password environment is missing".into()))?;
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
    std::fs::write(
        &public_key,
        STANDARD
            .decode(payload.identity.updater_public_key.trim())
            .map_err(|_| Error::Invalid("compiled updater public key is invalid".into()))?,
    )?;
    inspect::run(
        Command::new("minisign").args(["-V", "-q", "-m"]).arg(package).arg("-p").arg(&public_key).arg("-x").arg(&signature),
        "updater signature does not match compiled application trust",
    )?;
    let signature = STANDARD.encode(std::fs::read(signature)?);
    std::fs::write(package.with_extension("exe.sig"), format!("{signature}\n"))?;
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
    std::fs::write(output.join("latest.json"), serde_json::to_vec_pretty(&metadata)?)?;
    scratch.close()?;
    Ok(())
}

pub fn verify_updater(package: &Path, compiled_key: &str) -> Result<(), Error> {
    let scratch = tempfile::tempdir()?;
    let key = scratch.path().join("updater.pub");
    let signature = scratch.path().join("installer.minisig");
    std::fs::write(
        &key,
        STANDARD.decode(compiled_key.trim()).map_err(|_| Error::Invalid("compiled updater public key is invalid".into()))?,
    )?;
    let encoded = std::fs::read_to_string(package.with_extension("exe.sig"))?;
    std::fs::write(
        &signature,
        STANDARD.decode(encoded.trim()).map_err(|_| Error::Invalid("updater signature is not valid base64".into()))?,
    )?;
    inspect::run(
        Command::new("minisign").args(["-V", "-q", "-m"]).arg(package).arg("-p").arg(key).arg("-x").arg(signature),
        "updater signature does not match finished package bytes and compiled trust",
    )
}

/* === Private Functions === */

/* === Tests === */
