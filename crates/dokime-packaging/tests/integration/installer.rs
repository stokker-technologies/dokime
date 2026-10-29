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

// Third Party Imports
use dokime_packaging::installer::{PackageOptions, package};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[cfg(windows)]
fn gui_resources(path: &std::path::Path) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::System::LibraryLoader::{BeginUpdateResourceW, EndUpdateResourceW, UpdateResourceW};

    let mut filename = path.as_os_str().encode_wide().collect::<Vec<_>>();
    filename.push(0);
    let update = unsafe { BeginUpdateResourceW(filename.as_ptr(), 0) };
    assert!(!update.is_null());
    let bytes = b"GUI fixture resource";
    let kind = std::ptr::without_provenance::<u16>(10);
    let name = std::ptr::without_provenance::<u16>(1);
    let written = unsafe { UpdateResourceW(update, kind, name, 0, bytes.as_ptr().cast(), bytes.len() as u32) };
    let committed = unsafe { EndUpdateResourceW(update, i32::from(written == 0)) };
    assert_ne!(written, 0);
    assert_ne!(committed, 0);
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Tests === */

#[tokio::test]
async fn distribution_identity() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("payload");
    std::fs::create_dir_all(source.join("bin")).unwrap();
    let pack = dokime::config::organization::resolve(None).unwrap();
    pack.write_to(&source.join("organization")).unwrap();
    std::fs::write(source.join("LICENSE"), b"fixture license").unwrap();
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let executable = std::env::current_exe().unwrap();
    let mut files = std::collections::BTreeMap::new();
    for program in ["dokime", "dokime-gui"] {
        let name = format!("bin/{program}{suffix}");
        let destination = source.join(&name);
        std::fs::copy(&executable, &destination).unwrap();
        #[cfg(windows)]
        if program == "dokime-gui" {
            gui_resources(&destination);
        }
        files.insert(name, hash(&std::fs::read(destination).unwrap()));
    }
    let original_gui = std::fs::read(source.join(format!("bin/dokime-gui{suffix}"))).unwrap();
    files.insert("LICENSE".to_owned(), hash(b"fixture license"));
    for (name, bytes) in &pack.files {
        files.insert(format!("organization/{name}"), hash(bytes));
    }
    let target = if cfg!(windows) { "x86_64-pc-windows-msvc" } else { "x86_64-unknown-linux-gnu" };
    let metadata = serde_json::json!({
        "schema": 1, "version": "1.2.3", "target": target,
        "identity": { "identifier": "com.stokker.dokime", "slug": "dokime", "updater_public_key": "fixture" },
        "files": files,
    });
    std::fs::write(source.join("payload.json"), serde_json::to_vec(&metadata).unwrap()).unwrap();
    let manifest = root.path().join("installer.toml");
    let formats = if cfg!(windows) { "'nsis'" } else { "'deb','rpm'" };
    let options = PackageOptions {
        artifacts: Some(source.clone()),
        plugins: vec![],
        output: root.path().join("output"),
        sign: false,
        formats: vec![],
        target: None,
    };
    for (identifier, slug) in [(None, "dokime"), (Some("com.hilti.dokime"), "com-hilti-dokime")] {
        let declaration = identifier.map_or_else(String::new, |identifier| format!("identifier='{identifier}'\n"));
        let text =
            format!("[installer]\nversion='1.2.3'\n{declaration}formats=[{formats}]\n[dokime]\nversion='1.2.3'\ntarget='{target}'\n");
        std::fs::write(&manifest, text).unwrap();
        let packages = match package(&manifest, &options).await {
            Ok(packages) => packages,
            Err(error) => {
                let retained = root.keep();
                panic!("{error}; installer fixture retained at {}", retained.display());
            }
        };
        assert_eq!(packages.len(), if cfg!(windows) { 1 } else { 2 });
        for package in packages {
            let expected = match package.extension().unwrap().to_str().unwrap() {
                "exe" => format!("{slug}-1.2.3-{target}-setup.exe"),
                "deb" => format!("{slug}_1.2.3_amd64.deb"),
                "rpm" => format!("{slug}-1.2.3-1.x86_64.rpm"),
                extension => panic!("unexpected package extension: {extension}"),
            };
            assert_eq!(package.file_name().unwrap().to_str().unwrap(), expected);
            let inspection = dokime_packaging::installer::inspect(&package).unwrap();
            let identity = &inspection.distribution.identity;
            assert_eq!(identity.identifier, identifier.unwrap_or("com.stokker.dokime"));
            assert_eq!(identity.slug, slug);
            assert_eq!(identity.updater_public_key, "fixture");
            let prefix = if cfg!(windows) { String::new() } else { format!("usr/lib/{slug}/") };
            let name = format!("{prefix}distribution.json");
            let bytes = serde_json::to_vec_pretty(&dokime::paths::DistributionIdentity::new(identity.identifier.clone()).unwrap()).unwrap();
            assert_eq!(inspection.distribution.files[&name], hash(&bytes));
            if !cfg!(windows) {
                assert!(inspection.distribution.files.contains_key(&format!("usr/bin/{slug}")));
                assert!(inspection.distribution.files.contains_key(&format!("usr/bin/{slug}-gui")));
            }
        }
    }
    assert_eq!(std::fs::read(source.join(format!("bin/dokime-gui{suffix}"))).unwrap(), original_gui);
}

#[tokio::test]
async fn installer() {
    // Payload Integrity Refusals Preserve The Unpublished Output.
    let root = tempfile::tempdir().unwrap();
    let artifacts = root.path().join("payload");
    std::fs::create_dir_all(artifacts.join("bin")).unwrap();
    std::fs::create_dir_all(artifacts.join("organization")).unwrap();
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let target = if cfg!(windows) { "x86_64-pc-windows-msvc" } else { "x86_64-unknown-linux-gnu" };
    let mut files = serde_json::Map::new();
    for name in
        [format!("bin/dokime{suffix}"), format!("bin/dokime-gui{suffix}"), "LICENSE".into(), "organization/organization.toml".into()]
    {
        std::fs::write(artifacts.join(&name), b"tampered").unwrap();
        files.insert(name, serde_json::Value::String("00".repeat(32)));
    }
    let mut metadata = serde_json::json!({
        "schema":1,
        "version":"1.2.3",
        "target":target,
        "identity":{
            "identifier":"com.example.dokime",
            "slug":"dokime",
            "updater_public_key":"fixture",
        },
        "files":files,
    });
    std::fs::write(artifacts.join("payload.json"), serde_json::to_vec(&metadata).unwrap()).unwrap();
    let manifest = root.path().join("installer.toml");
    let format = if cfg!(windows) { "nsis" } else { "deb" };
    std::fs::write(&manifest, format!("[installer]\nversion='1.2.3'\nformats=['{format}']\n[dokime]\nversion='1.2.3'\ntarget='{target}'"))
        .unwrap();
    let mut options = PackageOptions {
        plugins: Vec::new(),
        artifacts: Some(artifacts),
        output: root.path().join("output"),
        sign: false,
        formats: vec![],
        target: None,
    };
    let error = package(&manifest, &options).await.unwrap_err();
    assert!(matches!(error, dokime_packaging::Error::PayloadDigest { member } if metadata["files"].get(&member).is_some()));
    assert!(!options.output.exists());
    let digest: String = Sha256::digest(b"tampered").iter().map(|byte| format!("{byte:02x}")).collect();
    for value in metadata["files"].as_object_mut().unwrap().values_mut() {
        *value = serde_json::Value::String(digest.clone());
    }
    std::fs::write(options.artifacts.as_ref().unwrap().join("payload.json"), serde_json::to_vec(&metadata).unwrap()).unwrap();
    std::fs::write(options.artifacts.as_ref().unwrap().join("organization/injected.css"), b"unverified injected appearance").unwrap();
    let error = package(&manifest, &options).await.unwrap_err();
    assert!(matches!(error, dokime_packaging::Error::UnindexedPayload { member } if member == "organization/injected.css"));
    assert!(!options.output.exists());

    // Signing Refusal Precedes Payload Acquisition.
    options.sign = true;
    options.artifacts = None;
    let error = package(&manifest, &options).await.unwrap_err();
    assert!(matches!(error, dokime_packaging::Error::SigningConfiguration));
    assert!(!options.output.exists());

    // The Same Manifest Then Exercises Native TLS Before Any Installer Assembly.

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("https://{}", listener.local_addr().unwrap());
    let mut serving = tokio::task::JoinSet::new();
    serving.spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut record = [0];
        socket.read_exact(&mut record).await.unwrap();
        socket.shutdown().await.unwrap();
        record[0]
    });

    std::fs::write(&manifest, format!(
        "[installer]\nversion='1.2.3'\nformats=['{format}']\n[dokime]\nversion='1.2.3'\ntarget='{target}'\nrelease_url='{url}'\nsha256='{}'\n",
        "00".repeat(32)
    )).unwrap();
    options.sign = false;
    let result = package(&manifest, &options).await;
    assert!(result.is_err());
    assert_eq!(serving.join_next().await.unwrap().unwrap(), 0x16);
    assert!(!options.output.exists());
}
