/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Served Release Fixtures Never Contact A Public Release Service.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    io::{Cursor, Write},
    sync::{Arc, Mutex},
};

// Third Party Imports
use harmos_packaging::bundle::{Bundle, Identity, Limits, Source};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Local Imports
use dokime::{distribution, domain::model::plugins::release::Feed};
use dokime_packaging::installer::release::Release;

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
async fn verified_release_cache() {
    let root = tempfile::tempdir().unwrap();
    let cache = root.path().join("cache");
    let target = dokime::plugins::NATIVE_TARGET;
    let bundle = Bundle::build(
        Identity {
            name: "opaque".into(),
            version: "1.2.3".parse().unwrap(),
            description: String::new(),
            package: Default::default(),
        },
        &[Source {
            path: "resource.txt",
            bytes: b"checked plugin",
            executable: None,
        }],
        Limits::default(),
    )
    .unwrap();
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    archive.start_file("opaque.dokime", zip::write::SimpleFileOptions::default()).unwrap();
    archive.write_all(bundle.bytes()).unwrap();
    archive.start_file("unrequested.dokime", zip::write::SimpleFileOptions::default()).unwrap();
    archive.write_all(b"must not be extracted").unwrap();
    let archive = archive.finish().unwrap().into_inner();
    let archive_name = format!("dokime-sidecars-1.2.3-{target}.zip");
    let feed = serde_json::to_vec(&serde_json::json!({"format_version":2,"plugins":[{
        "name":"opaque", "kind":"sidecar", "version":"1.2.3", "requires_dokime":"^0.1",
        "targets":{target:{"archive":archive_name,"member":"opaque.dokime","sha256":distribution::digest(bundle.bytes())}}
    }]}))
    .unwrap();
    let mut distribution_zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    distribution_zip.start_file("distribution.json", zip::write::SimpleFileOptions::default()).unwrap();
    distribution_zip.write_all(b"distribution metadata").unwrap();
    let distribution_zip = distribution_zip.finish().unwrap().into_inner();
    let distribution_name = format!("dokime-distribution-1.2.3-{target}.zip");
    let mut contents: BTreeMap<String, Vec<u8>> =
        [("plugins.json".into(), feed.clone()), (archive_name.clone(), archive.clone()), (distribution_name.clone(), distribution_zip)]
            .into();
    let sums = contents.iter().map(|(name, bytes)| format!("{}  {name}\n", distribution::digest(bytes))).collect::<String>();
    contents.insert("SHA256SUMS".into(), sums.into_bytes());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}/", listener.local_addr().unwrap());
    let assets = contents.keys().map(|name| (name.clone(), format!("{origin}{name}").parse().unwrap())).collect();
    let contents = Arc::new(Mutex::new(contents));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let server_contents = contents.clone();
    let server_requests = requests.clone();
    let (stop, mut stopped) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        loop {
            let (mut stream, _) = tokio::select! {
                _ = &mut stopped => break,
                accepted = listener.accept() => accepted.unwrap(),
            };
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                request.push(stream.read_u8().await.unwrap());
            }
            let request = String::from_utf8(request).unwrap();
            let path = request.split_whitespace().nth(1).unwrap().trim_start_matches('/');
            server_requests.lock().unwrap().push(path.to_owned());
            let bytes = server_contents.lock().unwrap().get(path).unwrap().clone();
            let header = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bytes.len());
            stream.write_all(header.as_bytes()).await.unwrap();
            stream.write_all(&bytes).await.unwrap();
            stream.shutdown().await.unwrap();
        }
    });

    let _ = rustls::crypto::ring::default_provider().install_default();
    let release = Release::open(reqwest::Client::new(), assets, cache.clone()).await.unwrap();
    let destination = root.path().join("distribution");
    std::fs::create_dir(&destination).unwrap();
    assert_eq!(release.distribution("dev-channel", target, &destination).await.unwrap(), "1.2.3");
    assert_eq!(std::fs::read(destination.join("distribution.json")).unwrap(), b"distribution metadata");
    let selected = release.plugins(&["opaque".into()], target, "0.1.3", &root.path().join("plugins")).await.unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(std::fs::read(&selected[0]).unwrap(), bundle.bytes());
    assert!(!root.path().join("plugins/unrequested.dokime").exists());
    assert_eq!(requests.lock().unwrap().len(), 4);

    release.plugins(&["opaque".into()], target, "0.1.3", &root.path().join("cached")).await.unwrap();
    assert_eq!(requests.lock().unwrap().len(), 4);
    let cached_archive = cache.join(distribution::digest(&archive));
    std::fs::write(&cached_archive, b"corrupted cache").unwrap();
    release.plugins(&["opaque".into()], target, "0.1.3", &root.path().join("repaired")).await.unwrap();
    assert_eq!(std::fs::read(&cached_archive).unwrap(), archive);
    assert_eq!(requests.lock().unwrap().len(), 5);

    std::fs::remove_file(&cached_archive).unwrap();
    contents.lock().unwrap().insert(archive_name, b"corrupted download".to_vec());
    assert!(release.plugins(&["opaque".into()], target, "0.1.3", &root.path().join("refused")).await.is_err());
    assert!(!cached_archive.exists());
    assert!(release.plugins(&["missing".into()], target, "0.1.3", &root.path().join("missing")).await.is_err());
    assert!(release.plugins(&["opaque".into()], target, "0.2.0", &root.path().join("incompatible")).await.is_err());
    assert_eq!(requests.lock().unwrap().len(), 6);

    let mut feed = Feed::parse(&feed).unwrap();
    feed.plugins[0].targets.get_mut(target).unwrap().sha256 = "0".repeat(64);
    let member = feed.plugins[0].select("0.1.3", target).unwrap();
    assert!(distribution::plugin(&archive, member, &feed.plugins[0]).is_err());
    stop.send(()).unwrap();
    server.await.unwrap();
}
