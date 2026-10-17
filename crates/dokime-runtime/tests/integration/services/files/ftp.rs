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
// credential store by reference. FTPS needs a certificate the platform
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
use dokime_runtime::config::RuntimeCredentialConfig;
use dokime_runtime::domains::files::{GapReason, Phase, UNREACHABLE};
use dokime_domain::operations::files::{FileWatchPolicy, FileWatchSourcePayload, FtpWatchSource};

/* === Constants === */
const USER: &str = "bench";
const PASSWORD: &str = "bench-secret";
const CREDENTIAL: &str = "bench-ftp";

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
    async fn authenticate(
        &self,
        username: &str,
        credentials: &Credentials,
    ) -> Result<Principal, AuthenticationError> {
        if username != USER {
            return Err(AuthenticationError::BadUser);
        }
        if credentials.password.as_deref() != Some(PASSWORD) {
            return Err(AuthenticationError::BadPassword);
        }
        Ok(Principal {
            username: username.to_owned(),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
// Serves `root` on a free loopback port until the runtime stops.
async fn serve(root: &Path) -> u16 {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let home = root.to_path_buf();
    let server = ServerBuilder::with_authenticator(
        Box::new(move || Filesystem::new(home.clone()).unwrap()),
        Arc::new(BenchLogin),
    )
    .passive_ports(50_000..=60_000)
    .build()
    .unwrap();
    tokio::spawn(server.listen(format!("127.0.0.1:{port}")));

    // The In-Process Server Publishes No Readiness Signal; Yield Until Its Control Port Accepts.
    while tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .is_err()
    {
        tokio::task::yield_now().await;
    }
    port
}

fn ftp(url: String, credential: Option<&str>) -> FileWatchSourcePayload {
    FileWatchSourcePayload::Ftp(FtpWatchSource {
        url,
        credential_id: credential.map(str::to_owned),
    })
}

/* === Tests === */
#[tokio::test(flavor = "multi_thread")]
async fn stream_ftp_source() {
    let bench = Bench::start(|config, root| {
        let credentials = root.join("credentials.json");
        let stored = BTreeMap::from([(CREDENTIAL, PASSWORD)]);
        std::fs::write(&credentials, serde_json::to_string(&stored).unwrap()).unwrap();
        config
            .with_trusted_bench(true)
            .with_credential_backend(RuntimeCredentialConfig {
                backend: dokime_runtime::CredentialBackendKind::LocalFile,
                local_file: Some(credentials),
            })
    })
    .await;
    let served = tempdir().unwrap();
    let log = served.path().join("bench/logs/charger.log");
    write(&log, b"before the watch\n");
    write(&served.path().join("bench/readme.txt"), b"unmatched\n");
    let port = serve(served.path()).await;
    let url = format!("ftp://{USER}@127.0.0.1:{port}/bench");
    let rules = vec![include("**/*.log")];

    // A Wrong Or Missing Credential Is Refused At Open
    let refused = bench
        .handle
        .open_file_watch(
            &request(
                &bench.execution,
                ftp(url.clone(), None),
                rules.clone(),
                FileWatchPolicy::Stream,
            ),
            None,
        )
        .await
        .err()
        .unwrap();
    assert_eq!(refused.code(), Some(UNREACHABLE), "{refused}");
    let missing = bench
        .handle
        .open_file_watch(
            &request(
                &bench.execution,
                ftp(url.clone(), Some("absent")),
                rules.clone(),
                FileWatchPolicy::Stream,
            ),
            None,
        )
        .await
        .err()
        .unwrap();
    assert_eq!(missing.code(), Some("credential_not_found"), "{missing}");

    // The Stored Credential Opens The Watch; Listing Baselines Existing Files
    let mut watch = bench
        .handle
        .open_file_watch(
            &request(
                &bench.execution,
                ftp(url, Some(CREDENTIAL)),
                rules,
                FileWatchPolicy::Stream,
            ),
            None,
        )
        .await
        .unwrap();
    let first = poll(&mut watch).await;
    assert!(matches!(
        first.as_slice(),
        [WatchEvent::Progress { path, phase: Phase::Baseline, .. }] if path == "logs/charger.log"
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
    assert_artifact(
        &sealed["logs/charger.log"],
        "logs/charger.log",
        b"line 1\nline 2\n",
    );
    assert_artifact(
        &sealed["logs/new/bms.log"],
        "logs/new/bms.log",
        b"born\nx\n",
    );

    drop(watch);
    bench.stop().await;
}
