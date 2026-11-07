/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verified Binary Report Downloads Through the Shipping CLI

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

// Third Party Imports
use base64::Engine;
use sha2::{Digest, Sha256};

// Local Imports
use crate::backend::{Fixture, FixtureConfig, Host, ID, Report, ReportMode};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

async fn call(endpoint: &str, directory: &Path, arguments: &[&str]) -> std::process::Output {
    let executable = std::env::var("NEXTEST_BIN_EXE_dokime").ok().or_else(|| std::env::var("CARGO_BIN_EXE_dokime").ok()).unwrap();
    let mut command = dokime::domain::process::command(executable);
    command.current_dir(directory).args(arguments).args(["--endpoint", endpoint]).stdin(std::process::Stdio::null());
    tokio::task::spawn_blocking(move || command.output().unwrap()).await.unwrap()
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn report_download() {
    tokio::time::timeout(std::time::Duration::from_secs(180), async {
        let bytes = Arc::new((0..2_700_000).map(|index| (index % 251) as u8).collect::<Vec<_>>());
        let mode = Arc::new(Mutex::new(ReportMode::Current));
        let catalog = dokime::harmos::operations::Contracts::of::<dokime::client::contracts::Operations>().unwrap();
        let artifacts = harmos_packaging::proto::Artifacts::new(&catalog).unwrap();
        let host = Host {
            report: Some(Report { bytes: bytes.clone(), mode: mode.clone() }),
            entry: None,
            bytes: Vec::new(),
            calls: Arc::new(Mutex::new(Vec::new())),
            install_code: Arc::new(Mutex::new(tonic::Code::Ok)),
            descriptor_sha256: Sha256::digest(artifacts.descriptors()).to_vec(),
        };
        let runtime = dokime::harmos::Runtime::<Fixture>::launch((), FixtureConfig { host: Some(Arc::new(host)) }).await.unwrap();
        let attachment = runtime
            .grpc()
            .attach(dokime::harmos::grpc::Settings {
                address: "127.0.0.1:0".parse().unwrap(),
                ..Default::default()
            })
            .await
            .unwrap();
        let endpoint = format!("http://{}", attachment.local_addr());
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("report.html");
        let request = format!(r#"{{"execution_id":"{ID}"}}"#);
        let operation = ["operation", "call", "dokime.reports.download_execution_report", "--json", &request];
        let digest: String = Sha256::digest(bytes.as_slice()).iter().map(|byte| format!("{byte:02x}")).collect();

        let result = call(&endpoint, directory.path(), &operation.into_iter().chain(["--output", "report.html"]).collect::<Vec<_>>()).await;
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        assert_eq!(std::fs::read(&destination).unwrap(), *bytes);
        let summary: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(summary["size_bytes"], bytes.len() as u64);
        assert_eq!(summary["sha256"], digest);
        assert_eq!(summary["metadata"]["testExecutionId"], ID);
        assert_eq!(summary["path"], "report.html");

        // Refuse JSON Expansion And Preserve Prior Output On Every Failed Download.
        let result = call(&endpoint, directory.path(), &operation).await;
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("--output"));
        for failure in [ReportMode::DigestMismatch, ReportMode::TransportFailure, ReportMode::OffsetMismatch, ReportMode::SizeMismatch] {
            *mode.lock().unwrap() = failure;
            let result = call(
                &endpoint,
                directory.path(),
                &operation.into_iter().chain(["--output", "report.html", "--overwrite"]).collect::<Vec<_>>(),
            )
            .await;
            assert!(!result.status.success(), "failure mode {failure:?}");
            assert_eq!(std::fs::read(&destination).unwrap(), *bytes);
            assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
        }
        for origin in [ReportMode::Current, ReportMode::Sealed] {
            *mode.lock().unwrap() = origin;
            for subject in ["execution", "cycle"] {
                let result = call(&endpoint, directory.path(), &[subject, "report", ID, "--output", "report.html", "--overwrite"]).await;
                assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
                assert_eq!(std::fs::read(&destination).unwrap(), *bytes);
            }
        }
        let result = call(&endpoint, directory.path(), &["operation", "call", "dokime.runtime.health", "--output", "health.json"]).await;
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let events: serde_json::Value = serde_json::from_slice(&std::fs::read(directory.path().join("health.json")).unwrap()).unwrap();
        assert_eq!(events.as_array().unwrap().len(), 1);
        assert_eq!(events[0]["event"], "completion");
        assert_eq!(events[0]["value"]["backendId"], ID);

        let result = call(&endpoint, directory.path(), &["operation", "call", "dokime.backend.get_identity"]).await;
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let event: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        let descriptor = event["value"]["api"]["descriptorSha256"].as_str().unwrap();
        assert_eq!(base64::engine::general_purpose::STANDARD.decode(descriptor).unwrap(), Sha256::digest(artifacts.descriptors()).to_vec());
        let result =
            call(&endpoint, directory.path(), &["operation", "call", "dokime.backend.get_identity", "--output", "identity.json"]).await;
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(directory.path().join("identity.json")).unwrap()).unwrap();
        assert_eq!(saved, serde_json::json!([event]));

        attachment.detach().await.unwrap();
        runtime.shutdown().await.unwrap();
    })
    .await
    .expect("report download story exceeded its hang guard");
}
