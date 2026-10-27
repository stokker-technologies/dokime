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

/* === Tests === */

#[tokio::test]
async fn tampered_payload() {
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
    let options =
        PackageOptions { artifacts: Some(artifacts), output: root.path().join("output"), sign: false, formats: vec![], target: None };
    let error = package(&manifest, &options).await.unwrap_err();
    assert!(error.to_string().contains("digest mismatch"));
    assert!(!options.output.exists());
    let digest: String = Sha256::digest(b"tampered").iter().map(|byte| format!("{byte:02x}")).collect();
    for value in metadata["files"].as_object_mut().unwrap().values_mut() {
        *value = serde_json::Value::String(digest.clone());
    }
    std::fs::write(options.artifacts.as_ref().unwrap().join("payload.json"), serde_json::to_vec(&metadata).unwrap()).unwrap();
    std::fs::write(options.artifacts.as_ref().unwrap().join("organization/injected.css"), b"unverified injected appearance").unwrap();
    let error = package(&manifest, &options).await.unwrap_err();
    assert!(error.to_string().contains("not integrity indexed"));
    assert!(!options.output.exists());
}

#[tokio::test]
async fn missing_signing() {
    let root = tempfile::tempdir().unwrap();
    let manifest = root.path().join("installer.toml");
    let target = if cfg!(windows) { "x86_64-pc-windows-msvc" } else { "x86_64-unknown-linux-gnu" };
    let format = if cfg!(windows) { "nsis" } else { "deb" };
    std::fs::write(&manifest, format!("[installer]\nversion='1.2.3'\nformats=['{format}']\n[dokime]\nversion='1.2.3'\ntarget='{target}'"))
        .unwrap();
    let options = PackageOptions { artifacts: None, output: root.path().join("output"), sign: true, formats: vec![], target: None };
    let error = package(&manifest, &options).await.unwrap_err();
    assert!(error.to_string().contains("explicit signing configuration"));
    assert!(!options.output.exists());
}

#[tokio::test]
async fn download_initializes_tls() {
    let root = tempfile::tempdir().unwrap();
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
    let manifest = root.path().join("installer.toml");
    let target = if cfg!(windows) { "x86_64-pc-windows-msvc" } else { "x86_64-unknown-linux-gnu" };
    let format = if cfg!(windows) { "nsis" } else { "deb" };
    std::fs::write(&manifest, format!(
        "[installer]\nversion='1.2.3'\nformats=['{format}']\n[dokime]\nversion='1.2.3'\ntarget='{target}'\nrelease_url='{url}'\nsha256='{}'\n",
        "00".repeat(32)
    )).unwrap();
    let options = PackageOptions { artifacts: None, output: root.path().join("output"), sign: false, formats: vec![], target: None };
    let result = package(&manifest, &options).await;
    assert!(result.is_err());
    assert_eq!(serving.join_next().await.unwrap().unwrap(), 0x16);
    assert!(!options.output.exists());
}

#[cfg(windows)]
#[test]
fn staged_resource_iteration() {
    let root = std::env::current_dir().unwrap();
    let input = tempfile::tempdir().unwrap();
    let staging = tempfile::Builder::new().prefix("dokime-installer-test-").tempdir_in(&root).unwrap();
    let files = [
        ("organization/fonts/font.woff2", b"font payload".as_slice()),
        ("LICENSE", b"license payload".as_slice()),
        ("distribution.json", b"distribution payload".as_slice()),
    ];
    let mut resources = std::collections::HashMap::new();
    for (target, bytes) in files {
        let original = input.path().join(target);
        std::fs::create_dir_all(original.parent().unwrap()).unwrap();
        std::fs::write(&original, bytes).unwrap();
        let staged = staging.path().join(target);
        std::fs::create_dir_all(staged.parent().unwrap()).unwrap();
        std::fs::copy(original, &staged).unwrap();
        let relative = staged.strip_prefix(&root).unwrap().to_str().unwrap().to_owned();
        resources.insert(relative, target.to_owned());
    }
    let actual = tauri_utils::resources::ResourcePaths::from_map(&resources, true)
        .iter()
        .map(|resource| {
            let resource = resource.unwrap();
            assert!(resource.path().is_relative());
            assert_eq!(root.join(resource.path()), staging.path().join(resource.target()));
            (resource.target().to_owned(), std::fs::read(resource.path()).unwrap())
        })
        .collect::<std::collections::HashMap<_, _>>();
    for (target, bytes) in files {
        assert_eq!(actual[std::path::Path::new(target)], bytes);
    }
    let icon = staging.path().join("icon.ico");
    std::fs::write(&icon, b"icon payload").unwrap();
    let icons = vec![icon.strip_prefix(&root).unwrap().to_str().unwrap().to_owned()];
    let selected = tauri_utils::resources::ResourcePaths::new(&icons, false).next().unwrap().unwrap();
    assert!(selected.is_relative());
    assert_eq!(root.join(&selected), icon);
    assert_eq!(std::fs::read(selected).unwrap(), b"icon payload");
    let staged_path = staging.path().to_owned();
    staging.close().unwrap();
    assert!(!staged_path.exists());
}
