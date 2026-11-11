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

// Local Imports
use dokime_server::host::{Args, NetworkArgs, Session};
use futures_util::FutureExt;
use std::path::Path;

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

fn arguments(root: &Path) -> Args {
    Args {
        shutdown_stdin: false,
        dev_control: false,
        network: NetworkArgs {
            host: std::net::Ipv4Addr::LOCALHOST.into(),
            port: Some(0),
            message_limit_mib: Some(2),
        },
        title: None,
        label: None,
        output_root: Some(root.join("store")),
        application_root: Some(root.join("application")),
        env_file: None,
        plugins: Vec::new(),
        plugins_root: None,
    }
}

async fn reacquire(args: &Args) -> Session {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            match Session::start(args, args.configuration().unwrap(), args.network.config().unwrap(), Vec::new(), |_| {}).await {
                Ok(session) => break session,
                Err(_) => tokio::time::sleep(std::time::Duration::from_millis(10)).await,
            }
        }
    })
    .await
    .expect("cancelled startup releases its execution store")
}

#[tokio::test]
async fn startup_unwind_drains() {
    let root = tempfile::tempdir().unwrap();
    let args = arguments(root.path());
    let startup = Session::start(&args, args.configuration().unwrap(), args.network.config().unwrap(), Vec::new(), |phase| {
        if phase == "plugins" {
            panic!("deliberate startup cancellation");
        }
    });
    assert!(std::panic::AssertUnwindSafe(startup).catch_unwind().await.is_err());
    reacquire(&args).await.shutdown().await.unwrap();
}

#[tokio::test]
async fn session_drop_drains() {
    let root = tempfile::tempdir().unwrap();
    let args = arguments(root.path());
    let session = Session::start(&args, args.configuration().unwrap(), args.network.config().unwrap(), Vec::new(), |_| {}).await.unwrap();
    let address = session.address();
    drop(session);
    let mut replacement = reacquire(&args).await;
    assert!(tokio::net::TcpStream::connect(address).await.is_err(), "the dropped session releases its listener");
    replacement.shutdown().await.unwrap();
}
