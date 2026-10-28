/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Response Publication Failure After Operation Effects

use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use clap::Parser;
use dokime::harmos::RegistrationExt;
use tokio::sync::Notify;

use super::{Config, Server, runtime};

const POLICY: dokime::harmos::operations::OperationPolicy = dokime::harmos::operations::OperationPolicy {
    timeout: std::time::Duration::from_secs(120),
    attempt_timeout: std::time::Duration::from_secs(120),
    retries: dokime::harmos::operations::RetryPolicy::Never,
};

#[dokime::harmos::message(harmos_path = dokime::harmos)]
struct Receipt {
    calls: u32,
}

// The SDK Path Keeps The Declared Resource Ungated In This Feature-Less Test Crate.
#[dokime::harmos::operation(harmos_path = harmos_sdk, response = Receipt, output = Receipt,
    id = "fixture.output.mutate", policy = POLICY, resources = [Gate],
    visibility = dokime::harmos::operations::Visibility::Host,
    grpc(package = "fixture.output.v1", service = "Device", method = "Mutate"))]
struct Mutate {
    unavailable: bool,
}

#[derive(Default)]
struct Gate {
    calls: AtomicU32,
    acted: Notify,
    release: Notify,
}

impl dokime::harmos::Operation for Mutate {
    type Error = tonic::Status;

    async fn run(self, ctx: dokime::harmos::Context<'_, Self>) -> Result<dokime::harmos::operations::Response<Receipt>, Self::Error> {
        let gate = ctx.resource::<Gate>();
        let calls = gate.calls.fetch_add(1, Ordering::SeqCst) + 1;
        let size = std::mem::size_of::<Receipt>();
        let item = || Ok(Receipt { calls });
        let sent = ctx.streams().send_with(size, item).await;
        sent.map_err(|error| {
            let message = error.to_string();
            tonic::Status::internal(message)
        })?;

        gate.acted.notify_one();
        gate.release.notified().await;
        if self.unavailable {
            return Err(tonic::Status::unavailable("the operation acted but its completion is unavailable"));
        }
        Ok(Receipt { calls }.into())
    }
}

#[tokio::test]
async fn response_publication_faults() {
    for overwrite in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let mut owner = runtime(root.path()).await;
        let handle = owner.handle();
        let resources = Gate::default();
        let gate = Arc::new(resources);
        let mut dependencies = dokime::harmos::Dependencies::new();
        dependencies.insert(gate.clone()).unwrap();
        let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<Mutate>();
        handle.operations().register(batch.lane(dokime::app::Lanes::Open)).await.unwrap();
        let server = Server::start(handle, Config::local()).await.unwrap();
        let endpoint = server.local_endpoint();

        let directory = root.path().join("response");
        std::fs::create_dir(&directory).unwrap();
        let destination = directory.join("response.json");
        if overwrite {
            std::fs::write(&destination, b"original response").unwrap();
        }
        let command =
            ["dokime", "operation", "call", "fixture.output.mutate", "--endpoint", &endpoint, "--output", destination.to_str().unwrap()];
        let arguments = command.into_iter().chain(overwrite.then_some("--overwrite"));
        let args = dokime_cli::Cli::try_parse_from(arguments).unwrap();
        let pending = args.run(dokime::config::Config::local(root.path()));
        tokio::pin!(pending);

        // The Real Handler Has Acted, But Cannot Complete Before The Destination Changes.
        tokio::select! {
            () = gate.acted.notified() => {},
            result = &mut pending => panic!("command returned before the operation acted: {result:?}"),
        }
        assert_eq!(gate.calls.load(Ordering::SeqCst), 1);
        if overwrite {
            std::fs::rename(&destination, directory.join("original.json")).unwrap();
            std::fs::create_dir(&destination).unwrap();
            std::fs::write(destination.join("retained.json"), b"concurrent directory content").unwrap();
        } else {
            std::fs::write(&destination, b"concurrent response").unwrap();
        }

        gate.release.notify_one();
        let error = pending.await.unwrap_err();
        assert!(matches!(error, dokime_cli::Error::OperationOutput { .. }), "{error}");
        assert_eq!(error.exit_code(), std::process::ExitCode::from(3));
        assert_eq!(gate.calls.load(Ordering::SeqCst), 1, "a failed response publication must never replay the operation");

        if overwrite {
            assert_eq!(std::fs::read(directory.join("original.json")).unwrap(), b"original response");
            assert_eq!(std::fs::read(destination.join("retained.json")).unwrap(), b"concurrent directory content");
            assert_eq!(std::fs::read_dir(&destination).unwrap().count(), 1);
        } else {
            assert_eq!(std::fs::read(&destination).unwrap(), b"concurrent response");
        }
        assert_eq!(
            std::fs::read_dir(&directory).unwrap().count(),
            if overwrite { 2 } else { 1 },
            "failed publication must remove staged output"
        );

        server.shutdown().await.unwrap();
        owner.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn response_unavailable_after_effect() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let resources = Gate::default();
    let gate = Arc::new(resources);
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(gate.clone()).unwrap();
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<Mutate>();
    handle.operations().register(batch.lane(dokime::app::Lanes::Open)).await.unwrap();
    let server = Server::start(handle, Config::local()).await.unwrap();
    let endpoint = server.local_endpoint();

    let directory = root.path().join("response");
    std::fs::create_dir(&directory).unwrap();
    let destination = directory.join("response.json");
    std::fs::write(&destination, b"previous complete response").unwrap();
    let command = [
        "dokime",
        "operation",
        "call",
        "fixture.output.mutate",
        "--endpoint",
        &endpoint,
        "--json",
        "{\"unavailable\":true}",
        "--output",
        destination.to_str().unwrap(),
        "--overwrite",
    ];
    let args = dokime_cli::Cli::try_parse_from(command).unwrap();
    let pending = args.run(dokime::config::Config::local(root.path()));
    tokio::pin!(pending);

    tokio::select! {
        () = gate.acted.notified() => {},
        result = &mut pending => panic!("command returned before the operation acted: {result:?}"),
    }
    gate.release.notify_one();
    let error = pending.await.unwrap_err();

    assert!(matches!(error, dokime_cli::Error::OperationUncertain { .. }), "{error}");
    assert_eq!(error.exit_code(), std::process::ExitCode::from(3));
    assert_eq!(gate.calls.load(Ordering::SeqCst), 1, "an unavailable completion must never retry the effect");
    assert_eq!(std::fs::read(&destination).unwrap(), b"previous complete response");
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1, "an incomplete response must leave no staged file");

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
