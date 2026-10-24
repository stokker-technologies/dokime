/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Response Publication Failure After Operation Effects

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

// Third Party Imports
use clap::Parser;
use tokio::sync::Notify;

// Local Imports
use super::{Config, Server, runtime};

/* === Constants === */

const POLICY: dokime::harmos::operations::OperationPolicy = dokime::harmos::operations::OperationPolicy {
    timeout: std::time::Duration::from_secs(30),
    attempt_timeout: std::time::Duration::from_secs(30),
    retries: dokime::harmos::operations::RetryPolicy::Never,
    restart: dokime::harmos::operations::RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::harmos::message(harmos_path = dokime::harmos)]
struct Receipt {
    calls: u32,
}

#[dokime::harmos::message(harmos_path = dokime::harmos)]
#[dokime::harmos::operation(harmos_path = dokime::harmos, response = Receipt, output = Receipt,
    id = "fixture.output.mutate", version = 1, policy = POLICY,
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

struct Handler {
    gate: Arc<Gate>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl dokime::harmos::operations::Operation for Handler {
    type Declaration = Mutate;
    type Resources = Gate;
    type Error = tonic::Status;
    type Context<'a> = dokime::harmos::operations::Context<'a>;
    type Endpoints = dokime::harmos::operations::StreamOutput<Receipt>;

    async fn initialize<'ctx>(gate: Arc<Self::Resources>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { gate })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut Mutate, output: &mut Self::Endpoints,
    ) -> Result<dokime::harmos::operations::Response<Receipt>, Self::Error> {
        let calls = self.gate.calls.fetch_add(1, Ordering::SeqCst) + 1;
        let size = std::mem::size_of::<Receipt>();
        let item = || Ok(Receipt { calls });
        let sent = output.send_with(size, item).await;
        sent.map_err(|error| {
            let message = error.to_string();
            tonic::Status::internal(message)
        })?;

        self.gate.acted.notify_one();
        self.gate.release.notified().await;
        if request.unavailable {
            return Err(tonic::Status::unavailable("the operation acted but its completion is unavailable"));
        }
        Ok(Receipt { calls }.into())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn response_publication_faults() {
    for overwrite in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let mut owner = runtime(root.path()).await;
        let handle = owner.handle();
        let resources = Gate::default();
        let gate = Arc::new(resources);
        handle.operations().register_wire_operation::<Handler>("fixture.output", dokime::app::Lanes::Open, gate.clone()).unwrap();
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
        let pending = args.run();
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
    handle.operations().register_wire_operation::<Handler>("fixture.output", dokime::app::Lanes::Open, gate.clone()).unwrap();
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
    let pending = args.run();
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
