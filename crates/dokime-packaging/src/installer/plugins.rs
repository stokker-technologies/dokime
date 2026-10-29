/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify And Stage Installer Plugin Bundles

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use harmos_packaging::bundle::{Bundle, Facility, Limits};

use crate::{
    Error,
    bundle::{COMPATIBILITY_FILE, Compatibility},
};

pub(super) fn stage(
    paths: &[PathBuf], base: &Path, organization: Option<&Path>, scratch: &Path, target: &str, version: &str,
) -> Result<BTreeMap<String, PathBuf>, Error> {
    let mut selected = folder(&base.join("plugins"))?;
    if let Some(organization) = organization {
        selected.extend(folder(&organization.join("plugins"))?);
    }
    for path in paths {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::Invalid(format!("invalid plugin bundle path: {}", path.display())))?;
        if selected.insert(name.to_owned(), path.clone()).is_some() {
            return Err(Error::Invalid(format!("duplicate installer plugin filename: {name}")));
        }
    }

    let mut names = BTreeSet::new();
    let mut packages = BTreeSet::new();
    let mut verified = Vec::new();
    for (name, path) in selected {
        super::payload::relative(&name)?;
        let folded = name.to_lowercase();
        if !names.insert(folded) {
            return Err(Error::Invalid(format!("duplicate installer plugin filename: {name}")));
        }
        let inspected = verify(&path, target, version);
        let (bytes, package) = inspected.map_err(|error| Error::Invalid(format!("plugin bundle {}: {error}", path.display())))?;
        if !packages.insert(package.clone()) {
            return Err(Error::Invalid(format!("plugin bundle {}: duplicate package {package}", path.display())));
        }
        verified.push((name, bytes));
    }

    let mut staged = BTreeMap::new();
    if verified.is_empty() {
        return Ok(staged);
    }
    let directory = scratch.join("plugins");
    std::fs::create_dir(&directory)?;
    for (name, bytes) in verified {
        let destination = directory.join(&name);
        std::fs::write(&destination, bytes)?;
        staged.insert(name, destination);
    }
    Ok(staged)
}

fn folder(directory: &Path) -> Result<BTreeMap<String, PathBuf>, Error> {
    let metadata = match std::fs::symlink_metadata(directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_dir() {
        return Err(Error::Invalid(format!("plugin folder must be a directory: {}", directory.display())));
    }
    let mut paths = BTreeMap::new();
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| Error::Invalid(format!("invalid plugin bundle path: {}", path.display())))?;
        paths.insert(name.to_owned(), path.clone());
        if paths.len() > Limits::default().files {
            return Err(Error::Invalid(format!("too many plugin bundles in {}", directory.display())));
        }
    }
    Ok(paths)
}

fn verify(path: &Path, target: &str, version: &str) -> Result<(Vec<u8>, String), Error> {
    if path.extension().is_none_or(|extension| extension != "dokime") {
        return Err(Error::Invalid("installer plugins must be .dokime bundles".into()));
    }
    let bytes = crate::bundle::read(path)?;
    let bundle = Bundle::inspect(&bytes, Limits::default())?;
    let executables: Vec<_> = bundle.manifest().files.iter().filter_map(|file| file.executable.as_ref()).collect();
    if executables.is_empty()
        || executables.iter().any(|executable| executable.target != target || executable.facility != Facility::Sidecar)
    {
        return Err(Error::Invalid(format!("expected only native sidecars for target {target}")));
    }
    let compatibility = bundle.file(COMPATIBILITY_FILE).ok_or_else(|| Error::Invalid("missing Dokime compatibility metadata".into()))?;
    let compatibility: Compatibility = serde_json::from_slice(compatibility)?;
    let version = version.parse().map_err(|error: semver::Error| Error::Invalid(error.to_string()))?;
    if compatibility.schema != 1 || !compatibility.requires_dokime.matches(&version) {
        return Err(Error::Invalid(format!("requires Dokime {}, installer has {version}", compatibility.requires_dokime)));
    }
    Ok((bytes, bundle.manifest().identity.name.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use harmos_packaging::bundle::{Executable, Identity, Source};

    fn bundle(owner: &str, target: &str, requirement: &str) -> Vec<u8> {
        let prefix = std::str::from_utf8(dokime::harmos::artifacts::PREFIX).unwrap();
        let policy = dokime::operations::STOP_SETTLEMENT;
        let lifecycle = serde_json::json!([
            {"action":"initialize", "identity":"harmos.operation.v1", "policy":policy},
            {"action":"shutdown", "identity":"harmos.operation.v1", "policy":policy},
        ]);
        let records = [
            serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"Fixture","config":"()"}),
            serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"provides":[],"requires":[],"lifecycle":lifecycle}),
            serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}}),
            serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]}),
            serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]}),
        ];
        let executable = records.iter().map(|record| format!("{prefix}{record}\0")).collect::<String>();
        let compatibility = Compatibility {
            schema: 1,
            requires_dokime: requirement.parse().unwrap(),
        };
        let bytes = serde_json::to_vec(&compatibility).unwrap();
        Bundle::build(
            Identity {
                name: owner.into(),
                version: "1.0.0".parse().unwrap(),
                description: "Fixture".into(),
                package: Default::default(),
            },
            &[
                Source {
                    path: "bin/fixture",
                    bytes: executable.as_bytes(),
                    executable: Some(Executable {
                        facility: Facility::Sidecar,
                        target: target.into(),
                        registrations: vec![],
                    }),
                },
                Source {
                    path: COMPATIBILITY_FILE,
                    bytes: &bytes,
                    executable: None,
                },
                Source {
                    path: "assets/probe.txt",
                    bytes: b"verified resource",
                    executable: None,
                },
            ],
            Limits::default(),
        )
        .unwrap()
        .bytes()
        .to_vec()
    }

    fn selected(paths: &[PathBuf], scratch: &Path) -> Result<BTreeMap<String, PathBuf>, Error> {
        stage(paths, &scratch.join("base"), None, scratch, dokime::plugins::NATIVE_TARGET, "0.1.1")
    }

    #[test]
    fn explicit_plugins_staged() {
        let input = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let host = input.path().join("host.dokime");
        let bench = input.path().join("bench.dokime");
        std::fs::write(&host, bundle("host", dokime::plugins::NATIVE_TARGET, "^0.1")).unwrap();
        std::fs::write(&bench, bundle("bench", dokime::plugins::NATIVE_TARGET, "^0.1")).unwrap();

        assert!(selected(&[], scratch.path()).unwrap().is_empty());
        assert!(!scratch.path().join("plugins").exists());
        let staged = selected(std::slice::from_ref(&host), scratch.path()).unwrap();
        assert_eq!(staged.len(), 1);
        assert_eq!(std::fs::read(&staged["host.dokime"]).unwrap(), std::fs::read(host).unwrap());
        assert!(!scratch.path().join("plugins/bench.dokime").exists());

        let repeated = tempfile::tempdir().unwrap();
        let staged = selected(&[bench.clone(), bench.with_file_name("host.dokime")], repeated.path()).unwrap();
        assert_eq!(staged.len(), 2);
        assert_eq!(std::fs::read(&staged["bench.dokime"]).unwrap(), std::fs::read(bench).unwrap());
    }

    #[test]
    fn invalid_plugins_refused() {
        let input = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let host = input.path().join("host.dokime");
        std::fs::write(&host, b"bundle").unwrap();

        assert!(selected(&[host.clone(), host.clone()], scratch.path()).is_err());
        assert!(selected(&[host.clone(), host.with_file_name("HOST.dokime")], scratch.path()).is_err());
        assert!(selected(&[input.path().to_owned()], scratch.path()).is_err());
        assert!(selected(&[host.with_extension("zip")], scratch.path()).is_err());
        assert!(selected(&[host.with_file_name("missing.dokime")], scratch.path()).is_err());
        assert!(!scratch.path().join("plugins").exists());
    }

    #[test]
    fn organization_plugins_verified() {
        let base = tempfile::tempdir().unwrap();
        let organization = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        std::fs::create_dir(base.path().join("plugins")).unwrap();
        std::fs::create_dir(organization.path().join("plugins")).unwrap();
        let host = bundle("host", dokime::plugins::NATIVE_TARGET, "^0.1");
        std::fs::write(base.path().join("plugins/host.dokime"), b"replaced by overlay").unwrap();
        std::fs::write(organization.path().join("plugins/host.dokime"), &host).unwrap();
        let staged = stage(&[], base.path(), Some(organization.path()), scratch.path(), dokime::plugins::NATIVE_TARGET, "0.1.1").unwrap();
        assert_eq!(std::fs::read(&staged["host.dokime"]).unwrap(), host);
    }

    #[test]
    fn mismatches_name_bundle() {
        let input = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let path = input.path().join("host.dokime");
        for bytes in
            [bundle("host", "foreign-target", "^0.1"), bundle("host", dokime::plugins::NATIVE_TARGET, ">=99"), b"damaged archive".to_vec()]
        {
            std::fs::write(&path, bytes).unwrap();
            let error = selected(std::slice::from_ref(&path), scratch.path()).unwrap_err();
            assert!(error.to_string().contains("host.dokime"));
            assert!(!scratch.path().join("plugins").exists());
        }
        // A Valid Archive With An Unchanged Manifest Cannot Hide Altered Member Bytes.
        let mut bytes = bundle("host", dokime::plugins::NATIVE_TARGET, "^0.1");
        let offset = {
            let mut archive = tar::Archive::new(bytes.as_slice());
            let member = archive
                .entries()
                .unwrap()
                .map(Result::unwrap)
                .find(|entry| entry.path().unwrap() == Path::new("assets/probe.txt"))
                .unwrap();
            usize::try_from(member.raw_file_position()).unwrap()
        };
        bytes[offset] ^= 1;
        std::fs::write(&path, bytes).unwrap();
        let error = selected(&[path], scratch.path()).unwrap_err();
        assert!(error.to_string().contains("host.dokime"));
        assert!(!scratch.path().join("plugins").exists());
    }
}
