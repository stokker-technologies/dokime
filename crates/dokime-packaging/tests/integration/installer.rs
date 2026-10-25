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
            "deep_link_scheme":"dokime-test-hub",
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
