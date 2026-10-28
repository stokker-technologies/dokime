/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, Serialize};

use super::payload::{self, Identity};
use crate::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DistributionMetadata {
    pub schema: u32,
    pub version: String,
    pub dokime_version: String,
    pub target: String,
    pub label: String,
    pub identity: Identity,
    pub files: BTreeMap<String, String>,
    pub installer_signed: bool,
    pub updater_signed: bool,
}

#[derive(Debug, Serialize)]
pub struct Inspection {
    pub package: PathBuf,
    pub sha256: String,
    pub distribution: DistributionMetadata,
    pub verified_files: usize,
    pub installer_signature: String,
    pub updater_signature: String,
}

pub fn inspect(package: &Path) -> Result<Inspection, Error> {
    let original = package.canonicalize()?;
    let extension =
        original.extension().and_then(|value| value.to_str()).ok_or_else(|| Error::Invalid("installer package has no extension".into()))?;
    if !["exe", "deb", "rpm"].contains(&extension) {
        return Err(Error::Invalid("inspection accepts finished NSIS .exe, .deb or .rpm packages".into()));
    }
    let scratch = tempfile::tempdir()?;
    let package = scratch.path().join(format!("input.{extension}"));
    std::fs::copy(&original, &package)?;
    let original_signature = original.with_extension("exe.sig");
    if extension == "exe" && original_signature.is_file() {
        std::fs::copy(original_signature, package.with_extension("exe.sig"))?;
    }
    let mut header = [0u8; 8];
    std::fs::File::open(&package)?.read_exact(&mut header)?;
    validate_native_magic(extension, &header)?;
    validate_archive(&package)?;
    let contents = scratch.path().join("contents");
    std::fs::create_dir(&contents)?;
    match package.extension().and_then(|s| s.to_str()) {
        Some("exe") => {
            run(
                Command::new("7z").arg("x").arg("-y").arg(format!("-o{}", contents.display())).arg(&package),
                "NSIS inspection requires 7z",
            )?;
        }
        Some("deb") => {
            run(Command::new("dpkg-deb").arg("--extract").arg(&package).arg(&contents), "DEB inspection requires dpkg-deb")?;
        }
        Some("rpm") => {
            run(Command::new("bsdtar").arg("-xf").arg(&package).arg("-C").arg(&contents), "RPM inspection requires bsdtar (libarchive)")?;
        }
        _ => return Err(Error::Invalid("inspection accepts finished NSIS .exe, .deb or .rpm packages".into())),
    }
    let mut result = inspect_tree(&package, &contents)?;
    result.package = original;
    scratch.close()?;
    Ok(result)
}

pub fn inspect_tree(package: &Path, root: &Path) -> Result<Inspection, Error> {
    let candidates = [root.join("distribution.json"), root.join("usr/lib/dokime/distribution.json")];
    let found: Vec<_> = candidates.iter().filter(|p| p.is_file()).collect();
    if found.len() != 1 {
        return Err(Error::Invalid("finished installer must contain exactly one distribution manifest".into()));
    }
    let metadata: DistributionMetadata = serde_json::from_slice(&std::fs::read(found[0])?)?;
    validate_metadata(&metadata)?;
    let windows = metadata.target.contains("windows");
    let package_windows = package.extension().is_some_and(|extension| extension == "exe");
    if windows != package_windows {
        return Err(Error::Invalid("distribution target does not match the native package format".into()));
    }
    super::native::validate_label(&metadata.label)?;
    let organization = if windows { root.join("organization") } else { root.join("usr/lib/dokime/organization") };
    let pack = dokime::config::organization::resolve_with_base(Some(&organization), None).map_err(Error::Invalid)?;
    if pack.label() != metadata.label {
        return Err(Error::Invalid("embedded organization label differs from distribution metadata".into()));
    }
    for (name, expected) in &metadata.files {
        let path = root.join(payload::relative(name)?);
        if !path.is_file()
            || std::fs::symlink_metadata(&path)?.file_type().is_symlink()
            || payload::digest(&std::fs::read(path)?) != *expected
        {
            return Err(Error::Invalid(format!("finished installer content mismatch: {name}")));
        }
    }
    #[cfg(windows)]
    if windows {
        let (_, expected_icon) =
            pack.asset("application").ok_or_else(|| Error::Invalid("organization application artwork is missing".into()))?;
        let image = image::load_from_memory(expected_icon).map_err(|error| Error::Invalid(error.to_string()))?;
        let icon = super::artwork::icon(&image)?;
        let executable = root.join("dokime-gui.exe");
        super::windows::verify(&executable, &icon)?;
        if metadata.installer_signed {
            run(Command::new("signtool").args(["verify", "/pa"]).arg(&executable), "GUI executable Authenticode verification failed")?;
        }
    }
    if !windows {
        let launcher = root.join("usr/share/applications").join(format!("{}.desktop", metadata.label));
        let contents = std::fs::read_to_string(launcher)?;
        let name = contents.lines().find_map(|line| line.strip_prefix("Name="));
        let executable = contents.lines().find_map(|line| line.strip_prefix("Exec="));
        if name != Some(metadata.label.as_str()) || !executable.is_some_and(|value| value == "dokime-gui") {
            return Err(Error::Invalid("desktop launcher name or executable differs from the packaged application".into()));
        }
        let icon = contents
            .lines()
            .find_map(|line| line.strip_prefix("Icon="))
            .ok_or_else(|| Error::Invalid("desktop launcher has no icon".into()))?;
        if icon.is_empty() || !icon.bytes().all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c)) {
            return Err(Error::Invalid("desktop launcher icon is invalid".into()));
        }
        let (_, expected_icon) =
            pack.asset("application").ok_or_else(|| Error::Invalid("organization application artwork is missing".into()))?;
        let expected_icon = image::load_from_memory(expected_icon)
            .map_err(|_| Error::Invalid("organization application artwork is invalid".into()))?
            .to_rgba8();
        let icons = root.join("usr/share/icons/hicolor");
        let mut found = false;
        for dimension in std::fs::read_dir(icons)? {
            let path = dimension?.path().join("apps").join(format!("{icon}.png"));
            if path.is_file() {
                let actual_icon = image::load_from_memory(&std::fs::read(path)?)
                    .map_err(|_| Error::Invalid("packaged desktop icon is invalid".into()))?
                    .to_rgba8();
                if actual_icon != expected_icon {
                    return Err(Error::Invalid("packaged desktop icon differs from organization application artwork".into()));
                }
                found = true;
            }
        }
        if !found {
            return Err(Error::Invalid("desktop launcher icon is absent from the finished package".into()));
        }
    }
    verify_inventory(root, &metadata)?;
    let installer_signature = if metadata.installer_signed {
        match package.extension().and_then(|value| value.to_str()) {
            Some("exe") => {
                run(Command::new("signtool").args(["verify", "/pa"]).arg(package), "installer Authenticode verification failed")?
            }
            Some("deb") => {
                run(Command::new("debsig-verify").arg(package), "DEB signature verification requires trusted debsig-verify policy")?
            }
            Some("rpm") => verify_rpm_signature(package)?,
            _ => return Err(Error::Invalid("unsupported signed package".into())),
        }
        "verified".into()
    } else {
        "not-requested".into()
    };
    let updater_signature = if metadata.updater_signed {
        super::signing::verify_updater(package, &metadata.identity.updater_public_key)?;
        "verified".into()
    } else {
        "not-requested".into()
    };
    Ok(Inspection {
        package: package.into(),
        sha256: payload::digest(&std::fs::read(package)?),
        verified_files: metadata.files.len(),
        installer_signature,
        updater_signature,
        distribution: metadata,
    })
}

pub(crate) fn run(command: &mut Command, requirement: &str) -> Result<(), Error> {
    let status = command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
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

fn verify_inventory(root: &Path, metadata: &DistributionMetadata) -> Result<(), Error> {
    let directory = if metadata.target.contains("windows") { root.join("organization") } else { root.join("usr/lib/dokime/organization") };
    let mut pending = vec![directory];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                return Err(Error::Invalid("finished organization contains a symbolic link".into()));
            }
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            let name = path
                .strip_prefix(root)
                .map_err(|_| Error::Invalid("installed path escapes package root".into()))?
                .to_string_lossy()
                .replace('\\', "/");
            if !kind.is_file() || !metadata.files.contains_key(&name) {
                return Err(Error::Invalid(format!("unverified organization file in finished installer: {name}")));
            }
        }
    }
    Ok(())
}

fn validate_native_magic(extension: &str, header: &[u8]) -> Result<(), Error> {
    let valid = match extension {
        "exe" => header.starts_with(b"MZ"),
        "rpm" => header.starts_with(&[0xed, 0xab, 0xee, 0xdb]),
        "deb" => header.starts_with(b"!<arch>\n"),
        _ => false,
    };
    if !valid {
        return Err(Error::Invalid("package bytes do not match their declared native installer format".into()));
    }
    Ok(())
}

fn validate_nsis_inventory(text: &str) -> Result<(), Error> {
    let mut members = false;
    let mut nsis = false;
    for line in text.lines() {
        if line == "----------" {
            members = true;
            continue;
        }
        if !members {
            if line == "Type = Nsis" {
                nsis = true;
            }
            continue;
        }
        if line.starts_with("Symbolic Link =") || line.starts_with("Hard Link =") {
            return Err(Error::Invalid("installer contains a linked member".into()));
        }
        if let Some(name) = line.strip_prefix("Path = ") {
            payload::relative(&name.replace('\\', "/"))?;
        }
    }
    if !members || !nsis {
        return Err(Error::Invalid("package is not an NSIS installer with a valid member inventory".into()));
    }
    Ok(())
}

fn validate_archive(package: &Path) -> Result<(), Error> {
    let extension = package.extension().and_then(|value| value.to_str());
    if extension == Some("exe") {
        let listing = Command::new("7z")
            .args(["l", "-slt", "-sccUTF-8"])
            .arg(package)
            .output()
            .map_err(|_| Error::Invalid("NSIS inspection requires 7z".into()))?;
        if !listing.status.success() {
            return Err(Error::Invalid("could not list NSIS contents".into()));
        }
        let text = String::from_utf8(listing.stdout).map_err(|_| Error::Invalid("NSIS member listing is not UTF-8".into()))?;
        validate_nsis_inventory(&text)?;
        return Ok(());
    }
    let archive = tempfile::NamedTempFile::new()?;
    let source = if extension == Some("deb") {
        let file = archive.reopen()?;
        let status = Command::new("dpkg-deb")
            .arg("--fsys-tarfile")
            .arg(package)
            .stdout(file)
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|_| Error::Invalid("DEB inspection requires dpkg-deb".into()))?;
        if !status.success() {
            return Err(Error::Invalid("could not read DEB file inventory".into()));
        }
        archive.path()
    } else {
        package
    };
    let listing = Command::new("bsdtar")
        .arg("-tvf")
        .arg(source)
        .output()
        .map_err(|_| Error::Invalid("installer inspection requires bsdtar (libarchive)".into()))?;
    if !listing.status.success() {
        return Err(Error::Invalid("could not list installer contents".into()));
    }
    let text = String::from_utf8(listing.stdout).map_err(|_| Error::Invalid("installer inventory is not UTF-8".into()))?;
    if text.lines().any(|line| !matches!(line.as_bytes().first(), Some(b'-' | b'd'))) {
        return Err(Error::Invalid("installer contains a link or special file".into()));
    }
    let listing = Command::new("bsdtar")
        .arg("-tf")
        .arg(source)
        .output()
        .map_err(|_| Error::Invalid("installer inspection requires bsdtar".into()))?;
    if !listing.status.success() {
        return Err(Error::Invalid("could not list installer paths".into()));
    }
    let text = String::from_utf8(listing.stdout).map_err(|_| Error::Invalid("installer path inventory is not UTF-8".into()))?;
    for name in text.lines() {
        let name = name.strip_prefix("./").unwrap_or(name).trim_end_matches('/');
        if !name.is_empty() {
            payload::relative(name)?;
        }
    }
    Ok(())
}

fn validate_metadata(metadata: &DistributionMetadata) -> Result<(), Error> {
    if metadata.schema != 1 || metadata.version.is_empty() || metadata.dokime_version.is_empty() || metadata.label.is_empty() {
        return Err(Error::Invalid("invalid embedded distribution manifest".into()));
    }
    let required = match metadata.target.as_str() {
        "x86_64-pc-windows-msvc" => ["dokime.exe", "dokime-gui.exe", "organization/organization.toml", "LICENSE"],
        "x86_64-unknown-linux-gnu" => {
            ["usr/bin/dokime", "usr/bin/dokime-gui", "usr/lib/dokime/organization/organization.toml", "usr/lib/dokime/LICENSE"]
        }
        _ => return Err(Error::Invalid("invalid embedded distribution target".into())),
    };
    if required.iter().any(|name| !metadata.files.contains_key(*name)) {
        return Err(Error::Invalid("distribution manifest omits required installed files".into()));
    }
    for (name, digest) in &metadata.files {
        payload::relative(name)?;
        if digest.len() != 64 || !digest.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::Invalid("invalid distribution file SHA-256".into()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_magic() {
        assert!(validate_native_magic("exe", b"MZpayload").is_ok());
        assert!(validate_native_magic("rpm", &[0xed, 0xab, 0xee, 0xdb]).is_ok());
        assert!(validate_native_magic("deb", b"!<arch>\n").is_ok());
        for extension in ["exe", "rpm", "deb"] {
            assert!(validate_native_magic(extension, b"PK\x03\x04zip").is_err());
        }
    }

    #[test]
    fn nsis_inventory() {
        assert!(validate_nsis_inventory("Type = Nsis\n----------\nPath = dokime.exe\n").is_ok());
        assert!(validate_nsis_inventory("Type = zip\n----------\nPath = dokime.exe\n").is_err());
        assert!(validate_nsis_inventory("Type = Nsis\n----------\nPath = ../outside\n").is_err());
    }

    #[test]
    fn complete_inventory() {
        let files = ["dokime.exe", "dokime-gui.exe", "LICENSE", "organization/organization.toml"]
            .into_iter()
            .map(|name| (name.into(), "00".repeat(32)))
            .collect();
        let identity = Identity {
            identifier: "com.example.dokime".into(),
            slug: "dokime".into(),
            updater_public_key: "fixture".into(),
        };
        let mut metadata = DistributionMetadata {
            schema: 1,
            version: "1.2.3".into(),
            dokime_version: "1.2.3".into(),
            target: "x86_64-pc-windows-msvc".into(),
            label: "Company".into(),
            identity,
            files,
            installer_signed: false,
            updater_signed: false,
        };
        assert!(validate_metadata(&metadata).is_ok());
        metadata.files.remove("dokime-gui.exe");
        assert!(validate_metadata(&metadata).is_err());
    }
}
