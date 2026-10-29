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
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

// Third Party Imports
use nsis::{
    NsisInstaller,
    strings::{NsisString, StringSegment},
};
use serde::{Deserialize, Serialize};

// Local Imports
use super::payload::{self, Identity};
use crate::Error;

/* === Constants === */

// NSIS Addresses File Data With 32-Bit Signed Offsets, So No Valid Solid Stream Exceeds 2 GiB.
const NSIS_PAYLOAD_LIMIT: usize = i32::MAX as usize;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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

struct Members<'a> {
    root: &'a Path,
    files: BTreeSet<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<'a> Members<'a> {
    fn new(root: &'a Path) -> Self {
        Self { root, files: BTreeSet::new() }
    }

    fn directory(&mut self, name: &str) -> Result<(), Error> {
        let name = name.strip_prefix("./").unwrap_or(name).trim_end_matches('/');
        if name.is_empty() || name == "." {
            return Ok(());
        }
        std::fs::create_dir_all(self.root.join(payload::relative(name)?))?;
        Ok(())
    }

    fn file(&mut self, name: &str, bytes: &[u8]) -> Result<(), Error> {
        let name = name.strip_prefix("./").unwrap_or(name);
        let path = self.root.join(payload::relative(name)?);
        if !self.files.insert(name.to_owned()) {
            return Err(Error::Invalid(format!("installer contains a duplicate member: {name}")));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

/* === Public Functions === */

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
    let bytes = std::fs::read(&package)?;
    validate_native_magic(extension, &bytes)?;
    let contents = scratch.path().join("contents");
    std::fs::create_dir(&contents)?;
    read_members(extension, &bytes, &mut Members::new(&contents))?;
    drop(bytes);
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
            run(
                dokime::domain::process::command("signtool").args(["verify", "/pa"]).arg(&executable),
                "GUI executable Authenticode verification failed",
            )?;
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
            Some("exe") => run(
                dokime::domain::process::command("signtool").args(["verify", "/pa"]).arg(package),
                "installer Authenticode verification failed",
            )?,
            Some("deb") => run(
                dokime::domain::process::command("debsig-verify").arg(package),
                "DEB signature verification requires trusted debsig-verify policy",
            )?,
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

/* === Private Functions === */

pub(crate) fn verify_rpm_signature(package: &Path) -> Result<(), Error> {
    let result = dokime::domain::process::command("rpmkeys")
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

fn read_members(extension: &str, bytes: &[u8], members: &mut Members<'_>) -> Result<(), Error> {
    match extension {
        "exe" => read_nsis(bytes, members),
        "deb" => read_deb(bytes, members),
        "rpm" => read_rpm(bytes, members),
        _ => Err(Error::Invalid("inspection accepts finished NSIS .exe, .deb or .rpm packages".into())),
    }
}

fn read_nsis(bytes: &[u8], members: &mut Members<'_>) -> Result<(), Error> {
    let invalid = |error: nsis::Error| Error::Invalid(format!("could not read NSIS installer: {error}"));
    let installer = NsisInstaller::builder(bytes).max_decompressed_size(NSIS_PAYLOAD_LIMIT).parse().map_err(invalid)?;
    for file in installer.files() {
        let file = file.map_err(invalid)?;
        let destination = file.dest_path().map_err(invalid)?;
        if let Some(name) = nsis_installed_name(&destination)? {
            members.file(&name, &file.decompress().map_err(invalid)?)?;
        }
    }
    Ok(())
}

fn nsis_installed_name(destination: &NsisString) -> Result<Option<String>, Error> {
    let rendered = destination.to_install_path();
    let Some((StringSegment::Variable { name: root, .. }, rest)) = destination.segments.split_first() else {
        return Err(Error::Invalid(format!("installer extracts a file outside its installation directory: {rendered}")));
    };
    match root.as_ref() {
        "$INSTDIR" => {}
        "$PLUGINSDIR" | "$TEMP" => return Ok(None),
        _ => return Err(Error::Invalid(format!("installer extracts a file outside its installation directory: {rendered}"))),
    }
    let mut name = String::new();
    for segment in rest {
        let StringSegment::Literal(text) = segment else {
            return Err(Error::Invalid(format!("installer member has an unresolved path: {rendered}")));
        };
        name.push_str(text);
    }
    let name = name.strip_prefix('\\').ok_or_else(|| Error::Invalid(format!("installer member has an invalid path: {rendered}")))?;
    Ok(Some(name.replace('\\', "/")))
}

fn read_deb(bytes: &[u8], members: &mut Members<'_>) -> Result<(), Error> {
    let mut archive = ar::Archive::new(bytes);
    let mut format = false;
    let mut data = false;
    while let Some(entry) = archive.next_entry() {
        let mut entry = entry?;
        let identifier = entry.header().identifier().to_vec();
        match identifier.as_slice() {
            b"debian-binary" if !format && !data => {
                let mut version = Vec::new();
                entry.read_to_end(&mut version)?;
                if version != b"2.0\n" {
                    return Err(Error::Invalid("DEB format version is not 2.0".into()));
                }
                format = true;
            }
            name if format && !data && name.starts_with(b"control.tar") => {}
            b"data.tar.gz" if format && !data => {
                read_tar(flate2::read::GzDecoder::new(&mut entry), members)?;
                data = true;
            }
            _ => return Err(Error::Invalid("DEB members must be debian-binary, control.tar and a gzip data.tar.gz".into())),
        }
    }
    if !data {
        return Err(Error::Invalid("DEB has no data.tar.gz payload".into()));
    }
    Ok(())
}

fn read_tar(reader: impl Read, members: &mut Members<'_>) -> Result<(), Error> {
    let mut archive = tar::Archive::new(reader);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let name =
            String::from_utf8(entry.path_bytes().into_owned()).map_err(|_| Error::Invalid("installer member path is not UTF-8".into()))?;
        match entry.header().entry_type() {
            tar::EntryType::Directory => members.directory(&name)?,
            tar::EntryType::Regular => {
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes)?;
                members.file(&name, &bytes)?;
            }
            _ => return Err(Error::Invalid("installer contains a link or special file".into())),
        }
    }
    Ok(())
}

fn read_rpm(bytes: &[u8], members: &mut Members<'_>) -> Result<(), Error> {
    let invalid = |error: rpm::Error| Error::Invalid(format!("could not read RPM package: {error}"));
    let package = rpm::Package::parse(&mut std::io::Cursor::new(bytes)).map_err(invalid)?;
    package.verify_digests().map_err(invalid)?;
    for file in package.files().map_err(invalid)? {
        let file = file.map_err(invalid)?;
        let entry = &file.metadata;
        let path = format!("{}{}", entry.dirname(), entry.basename());
        let name = path.strip_prefix('/').ok_or_else(|| Error::Invalid(format!("RPM member path is not absolute: {path}")))?;
        match (entry.file_type(), file.content()) {
            (rpm::FileType::Dir, _) => members.directory(name)?,
            (rpm::FileType::Regular, Some(content)) => members.file(name, content)?,
            _ => return Err(Error::Invalid(format!("installer contains a link, ghost or special file: {path}"))),
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

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    fn extract(extension: &str, bytes: &[u8]) -> Result<tempfile::TempDir, Error> {
        let root = tempfile::tempdir()?;
        read_members(extension, bytes, &mut Members::new(root.path()))?;
        Ok(root)
    }

    fn tar_gz(entries: &[(&str, tar::EntryType, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast()));
        for (name, kind, bytes) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(*kind);
            header.set_mode(0o755);
            header.set_size(bytes.len() as u64);
            if *kind == tar::EntryType::Symlink {
                header.set_link_name("dokime").unwrap();
            }
            header.as_gnu_mut().unwrap().name[..name.len()].copy_from_slice(name.as_bytes());
            header.set_cksum();
            builder.append(&header, *bytes).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    fn deb(members: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = ar::Builder::new(Vec::new());
        for (name, bytes) in members {
            builder.append(&ar::Header::new(name.as_bytes().to_vec(), bytes.len() as u64), *bytes).unwrap();
        }
        builder.into_inner().unwrap()
    }

    fn rpm(build: impl FnOnce(&mut rpm::PackageBuilder) -> &mut rpm::PackageBuilder) -> Vec<u8> {
        let mut builder = rpm::PackageBuilder::new("dokime", "1.2.3", "Apache-2.0", "x86_64", "fixture");
        builder.using_config(rpm::BuildConfig::default().compression(rpm::CompressionWithLevel::Gzip(1)));
        let mut bytes = Vec::new();
        build(&mut builder).build().unwrap().write(&mut bytes).unwrap();
        bytes
    }

    fn nsis_path(root: &'static str, rest: &[&str]) -> NsisString {
        let mut segments = vec![StringSegment::Variable { index: 0, name: Cow::Borrowed(root) }];
        segments.extend(rest.iter().map(|text| StringSegment::Literal((*text).into())));
        NsisString { segments }
    }

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
    fn deb_members() {
        let data = tar_gz(&[
            ("./", tar::EntryType::Directory, b""),
            ("./usr/bin/", tar::EntryType::Directory, b""),
            ("./usr/bin/dokime", tar::EntryType::Regular, b"binary"),
            ("./usr/lib/dokime/LICENSE", tar::EntryType::Regular, b"license"),
        ]);
        let package = deb(&[("debian-binary", b"2.0\n"), ("control.tar.gz", b"control"), ("data.tar.gz", &data)]);
        let root = extract("deb", &package).unwrap();
        assert_eq!(std::fs::read(root.path().join("usr/bin/dokime")).unwrap(), b"binary");
        assert_eq!(std::fs::read(root.path().join("usr/lib/dokime/LICENSE")).unwrap(), b"license");

        let linked = tar_gz(&[("./usr/bin/link", tar::EntryType::Symlink, b"")]);
        let escaping = tar_gz(&[("../outside", tar::EntryType::Regular, b"x")]);
        let duplicate = tar_gz(&[("./LICENSE", tar::EntryType::Regular, b"a"), ("LICENSE", tar::EntryType::Regular, b"b")]);
        for data in [linked, escaping, duplicate] {
            let package = deb(&[("debian-binary", b"2.0\n"), ("control.tar.gz", b"control"), ("data.tar.gz", &data)]);
            assert!(extract("deb", &package).is_err());
        }
        assert!(extract("deb", &deb(&[("debian-binary", b"2.0\n"), ("data.tar.xz", b"xz")])).is_err());
        assert!(extract("deb", &deb(&[("data.tar.gz", &data)])).is_err());
        assert!(extract("deb", &deb(&[("debian-binary", b"2.0\n")])).is_err());
    }

    #[test]
    fn rpm_members() {
        let package = rpm(|builder| {
            builder.with_file_contents("binary", rpm::FileOptions::new("/usr/bin/dokime")).unwrap();
            builder.with_file_contents("license", rpm::FileOptions::new("/usr/lib/dokime/LICENSE")).unwrap()
        });
        let root = extract("rpm", &package).unwrap();
        assert_eq!(std::fs::read(root.path().join("usr/bin/dokime")).unwrap(), b"binary");
        assert_eq!(std::fs::read(root.path().join("usr/lib/dokime/LICENSE")).unwrap(), b"license");

        let mut tampered = package.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 0xff;
        assert!(extract("rpm", &tampered).is_err());
        let linked = rpm(|builder| builder.with_symlink(rpm::FileOptions::symlink("/usr/bin/link", "dokime")).unwrap());
        assert!(extract("rpm", &linked).is_err());
    }

    #[test]
    fn nsis_members() {
        let organization = nsis_path("$INSTDIR", &["\\", "organization\\organization.toml"]);
        assert_eq!(nsis_installed_name(&organization).unwrap().as_deref(), Some("organization/organization.toml"));
        assert_eq!(nsis_installed_name(&nsis_path("$INSTDIR", &["\\dokime.exe"])).unwrap().as_deref(), Some("dokime.exe"));
        assert_eq!(nsis_installed_name(&nsis_path("$PLUGINSDIR", &["\\nsDialogs.dll"])).unwrap(), None);
        assert_eq!(nsis_installed_name(&nsis_path("$TEMP", &["\\MicrosoftEdgeWebview2Setup.exe"])).unwrap(), None);
        assert!(nsis_installed_name(&nsis_path("$SYSDIR", &["\\dokime.dll"])).is_err());
        assert!(nsis_installed_name(&nsis_path("$INSTDIR", &["dokime.exe"])).is_err());
        let mut variable = nsis_path("$INSTDIR", &["\\"]);
        variable.segments.push(StringSegment::Variable {
            index: 0,
            name: Cow::Borrowed("$LANGUAGE"),
        });
        assert!(nsis_installed_name(&variable).is_err());
        let relative = NsisString {
            segments: vec![StringSegment::Literal("dokime.exe".into())],
        };
        assert!(nsis_installed_name(&relative).is_err());
        assert!(extract("exe", b"MZ\x90\x00 not an NSIS installer").is_err());
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
