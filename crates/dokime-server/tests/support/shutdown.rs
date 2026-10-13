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
use std::process::Command;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Host(std::process::Child);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl std::ops::Deref for Host {
    type Target = std::process::Child;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Host {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/* === Public Functions === */
pub async fn measure_shutdown(
    signal: bool,
    connected: bool,
    watching: bool,
    plugin: Option<&std::path::Path>,
    settle: std::time::Duration,
) -> std::time::Duration {
    use std::{
        io::{BufRead, BufReader},
        process::Stdio,
        time::{Duration, Instant},
    };
    let temp = tempfile::tempdir().unwrap();
    let mut command = Command::new(
        std::env::var_os("DOKIME_TEST_HOST").expect("mise selects the exact release host"),
    );
    command
        .args([
            "--dev-control",
            "--shutdown-stdin",
            "--no-seed-validation-artifact",
            "--port",
            "0",
            "--output-root",
        ])
        .arg(temp.path().join("data"))
        .arg("--application-root")
        .arg(temp.path().join("application"))
        .arg("--plugins-root")
        .arg(temp.path().join("plugins"))
        .env("RUST_LOG", "off")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(plugin) = plugin {
        command.arg("--plugin").arg(plugin);
    }

    let mut child = Host(command.spawn().unwrap());
    let stdout = child.stdout.take().unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        // `dokime-server --dev-control` prints one `{"event":"phase",...}` line per
        // startup boundary it reaches before the final `ready` line; collect
        // them and hand back both once `ready` arrives.
        let mut reader = BufReader::new(stdout);
        let mut phases = Vec::new();
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                let _ = send.send(Err(
                    "the backend exited before reporting readiness".to_owned()
                ));
                return;
            }
            let event: serde_json::Value = match serde_json::from_str(&line) {
                Ok(value) => value,
                Err(error) => {
                    let _ = send.send(Err(format!("unparseable readiness line: {error}")));
                    return;
                }
            };
            if event["event"] == "phase" {
                phases.push(event["name"].as_str().unwrap_or_default().to_owned());
                continue;
            }
            let _ = send.send(Ok((phases, event)));
            return;
        }
    });
    let received = receive.recv_timeout(Duration::from_secs(60));
    if received.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let (phases, ready) = received.expect("ready within deadline").unwrap();
    // The dev-control startup phases `serve` reaches, in order, before it
    // starts the gRPC listener and reports ready.
    assert_eq!(phases, vec!["storage", "plugins", "requests"]);
    assert_eq!(ready["event"], "ready");
    assert_eq!(ready["protocol"], 3);
    assert_eq!(ready["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(ready["dev_protocol"], 4);
    assert_eq!(ready["transport"], "grpc");
    let endpoint = ready["endpoint"].as_str().unwrap();
    let client = if connected {
        let client = dokime_client::Client::connect(
            dokime_client::Target::local(endpoint).unwrap(),
            None,
            None,
        )
        .await
        .unwrap();
        let health = client.health().await.unwrap();
        assert_eq!(
            health.readiness,
            dokime_client::contracts::runtime::Readiness::Ready
        );
        Some(client)
    } else {
        None
    };
    let observation = if watching {
        Some(
            dokime_client::Client::connect(
                dokime_client::Target::local(endpoint).unwrap(),
                None,
                None,
            )
            .await
            .unwrap(),
        )
    } else {
        None
    };
    let watch = if let Some(observation) = &observation {
        Some(
            observation
                .observation_watch_runtime(Default::default())
                .await
                .unwrap(),
        )
    } else {
        None
    };

    // Benchmarks preserve the idle settling window; correctness needs readiness only.
    if !settle.is_zero() {
        tokio::time::sleep(settle).await;
    }
    let started = Instant::now();
    if signal {
        assert!(
            Command::new("kill")
                .args(["-INT", &child.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
    } else {
        drop(child.stdin.take());
    }
    let deadline = started + Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("host failed to shut down");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let elapsed = started.elapsed();
    drop(watch);
    drop(observation);
    drop(client);
    elapsed
}

/* === Private Functions === */

/* === Tests === */
