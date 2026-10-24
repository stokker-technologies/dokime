/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A Stream watch over an in-process FTP server: listing discovery, ranged
// retrieval from the committed offset, and the password read from the host
// secret store by reference. FTPS needs a certificate the platform
// verifier trusts, so it is checked manually against a real server.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use libunftp::ServerBuilder;
use unftp_core::auth::{AuthenticationError, Authenticator, Credentials, Principal};
use unftp_sbe_fs::Filesystem;

// Local Imports
use super::*;
use dokime_domain::model;
use dokime_runtime::domains::files::{GapReason, Phase, UNREACHABLE};
use {model::files::wire::FileWatchPolicy, model::files::wire::FileWatchSourcePayload, model::files::wire::FtpWatchSource};

/* === Constants === */
const USER: &str = "bench";
const PASSWORD: &str = "bench-secret";
const SECRET: &str = "bench-ftp";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug)]
struct BenchLogin;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[async_trait::async_trait]
impl Authenticator for BenchLogin {
    async fn authenticate(&self, username: &str, secrets: &Credentials) -> Result<Principal, AuthenticationError> {
        if username != USER {
            return Err(AuthenticationError::BadUser);
        }
        if secrets.password.as_deref() != Some(PASSWORD) {
            return Err(AuthenticationError::BadPassword);
        }
        Ok(Principal { username: username.to_owned() })
    }
}

/* === Public Functions === */

/* === Private Functions === */
// Serves `root` on a free loopback port until the runtime stops.
async fn serve(root: &Path) -> u16 {
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let home = root.to_path_buf();
    let server = ServerBuilder::with_authenticator(Box::new(move || Filesystem::new(home.clone()).unwrap()), Arc::new(BenchLogin))
        .passive_ports(50_000..=60_000)
        .build()
        .unwrap();
    tokio::spawn(server.listen(format!("127.0.0.1:{port}")));

    // The In-Process Server Publishes No Readiness Signal; Yield Until Its Control Port Accepts.
    while tokio::net::TcpStream::connect(("127.0.0.1", port)).await.is_err() {
        tokio::task::yield_now().await;
    }
    port
}

fn ftp(url: String, secret: Option<&str>) -> FileWatchSourcePayload {
    FileWatchSourcePayload::Ftp(FtpWatchSource { url, secret_id: secret.map(str::to_owned) })
}

async fn store_ftp_secret(bench: &Bench) {
    let request = dokime_domain::operations::secrets::Put {
        metadata: dokime_domain::model::secrets::catalog::Metadata { name: SECRET.into(), label: "FTP Password".into(), description: None },
        value: Some(PASSWORD.into()),
        generation: 0,
    };
    bench.handle.operations().call(request).await.unwrap();
}

// Preserve The Real Server's Data Plane While Refusing REST On Its Control Plane
async fn refusing_resume(port: u16) -> (u16, tokio::task::JoinHandle<()>) {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_port = listener.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        while let Ok((client, _)) = listener.accept().await {
            tokio::spawn(async move {
                let server = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.unwrap();
                let (client_read, client_write) = client.into_split();
                let (server_read, mut server_write) = server.into_split();
                let client_write = Arc::new(tokio::sync::Mutex::new(client_write));
                let reply = client_write.clone();
                let incoming = async move {
                    let mut commands = BufReader::new(client_read).lines();
                    while let Ok(Some(command)) = commands.next_line().await {
                        if command.starts_with("REST ") {
                            reply.lock().await.write_all(b"502 REST unavailable\r\n").await.unwrap();
                            continue;
                        }
                        server_write.write_all(format!("{command}\r\n").as_bytes()).await.unwrap();
                    }
                };
                let outgoing = async move {
                    let mut responses = BufReader::new(server_read).lines();
                    while let Ok(Some(response)) = responses.next_line().await {
                        client_write.lock().await.write_all(format!("{response}\r\n").as_bytes()).await.unwrap();
                    }
                };
                tokio::select! { () = incoming => {}, () = outgoing => {} }
            });
        }
    });
    (proxy_port, task)
}

/* === Tests === */

#[tokio::test(flavor = "multi_thread")]
async fn refused_resume_reports_extraction_failure_and_artifact_only_fallback() {
    let bench = Bench::start(|config, _| config.with_trusted_bench(true)).await;
    store_ftp_secret(&bench).await;
    let served = tempdir().unwrap();
    let log = served.path().join("bench/device.log");
    write(&log, b"");
    let port = serve(served.path()).await;
    let (port, proxy) = refusing_resume(port).await;
    let url = format!("ftp://{USER}@127.0.0.1:{port}/bench/device.log");
    let request = dokime_domain::operations::files::WatchFile {
        execution_id: Some(bench.execution.clone()),
        source: Some(model::files::wire::FileWatchSource { payload: Some(ftp(url, Some(SECRET))) }),
        output: OUTPUT.to_owned(),
        artifact: None,
        policy: FileWatchPolicy::Stream,
        selection: model::files::wire::FileWatchSelection::All,
        wait_for_source_seconds: None,
        poll_interval_ms: Some(50),
        maximum_seconds: None,
        maximum_bytes: None,
        matchers: Vec::new(),
        markers: vec![model::files::wire::LineMarker::new("WARN", "tracking.warning")],
    };
    let mut extraction = bench.handle.open_watch_file(&request, None).await.unwrap();
    poll(&mut extraction).await;
    append(&log, b"WARN\n");
    poll(&mut extraction).await;
    append(&log, b"WARN\n");
    let mut events = Vec::new();
    let error = extraction.poll(&mut events).await.unwrap_err();
    assert_eq!(error.code(), Some(dokime_runtime::domains::files::UNSUPPORTED));
    assert!(error.to_string().contains("Enable REST"));
    assert!(events.iter().any(|event| matches!(event, WatchEvent::Gap { reason: GapReason::ResumeUnsupported, .. })));
    seal(&mut extraction).await;
    assert_eq!(extraction.sealed().len(), 1, "captured prefix remains evidence on extraction failure");
    let preserved = bench.handle.read_artifact_content(&extraction.sealed()[0].artifact_id, 100).await.unwrap().unwrap();
    assert_eq!(preserved.bytes, b"WARN\n");

    let artifact_request = dokime_domain::operations::files::WatchFile { markers: Vec::new(), output: "fallback".into(), ..request };
    let mut artifact = bench.handle.open_watch_file(&artifact_request, None).await.unwrap();
    poll(&mut artifact).await;
    append(&log, b"tail\n");
    let fallback = poll(&mut artifact).await;
    assert!(fallback.iter().any(|event| matches!(event, WatchEvent::Gap { reason: GapReason::ResumeUnsupported, .. })));
    seal(&mut artifact).await;
    let record = &artifact.sealed()[0];
    let content = bench.handle.read_artifact_content(&record.artifact_id, 100).await.unwrap().unwrap();
    assert_eq!(content.bytes, b"WARN\nWARN\ntail\n");
    drop(extraction);
    drop(artifact);
    proxy.abort();
    bench.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn stream_ftp_source() {
    let bench = Bench::start(|config, _| config.with_trusted_bench(true)).await;
    store_ftp_secret(&bench).await;
    let served = tempdir().unwrap();
    let log = served.path().join("bench/logs/charger.log");
    write(&log, b"before the watch\n");
    write(&served.path().join("bench/readme.txt"), b"unmatched\n");
    let port = serve(served.path()).await;
    let url = format!("ftp://{USER}@127.0.0.1:{port}/bench");
    let rules = vec![include("**/*.log")];

    // A Wrong Or Missing Secret Is Refused At Open
    let refused = bench
        .handle
        .open_file_watch(&request(&bench.execution, ftp(url.clone(), None), rules.clone(), FileWatchPolicy::Stream), None)
        .await
        .err()
        .unwrap();
    assert_eq!(refused.code(), Some(UNREACHABLE), "{refused}");
    let missing = bench
        .handle
        .open_file_watch(&request(&bench.execution, ftp(url.clone(), Some("absent")), rules.clone(), FileWatchPolicy::Stream), None)
        .await
        .err()
        .unwrap();
    assert_eq!(missing.code(), Some("secret_not_found"), "{missing}");

    // The Stored Secret Opens The Watch; Listing Baselines Existing Files
    let mut watch = bench
        .handle
        .open_file_watch(&request(&bench.execution, ftp(url, Some(SECRET)), rules, FileWatchPolicy::Stream), None)
        .await
        .unwrap();
    let first = poll(&mut watch).await;
    assert!(matches!(
        first.as_slice(),
        [WatchEvent::Armed { .. }, WatchEvent::Progress { path, phase: Phase::Baseline, .. }] if path == "logs/charger.log"
    ));

    // Ranged Retrieval Captures Only The Appended Bytes; New Files Whole
    append(&log, b"line 1\n");
    write(&served.path().join("bench/logs/new/bms.log"), b"born\n");
    poll(&mut watch).await;
    append(&log, b"line 2\n");
    poll(&mut watch).await;

    // Truncation Restarts At The First Byte With A Gap
    write(&served.path().join("bench/logs/new/bms.log"), b"x\n");
    let truncated = poll(&mut watch).await;
    assert!(truncated.iter().any(|event| matches!(
        event,
        WatchEvent::Gap { path, offset: 5, reason: GapReason::Truncated } if path == "logs/new/bms.log"
    )));

    let sealed = sealed(&seal(&mut watch).await);
    assert_artifact(&sealed["logs/charger.log"], "logs/charger.log", b"line 1\nline 2\n");
    assert_artifact(&sealed["logs/new/bms.log"], "logs/new/bms.segment2.log", b"x\n");

    drop(watch);
    bench.stop().await;
}
