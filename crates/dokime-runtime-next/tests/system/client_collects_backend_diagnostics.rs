/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Collects Backend Diagnostics

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

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
async fn client_collects_backend_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        use dokime::v1;

        // Read Backend Capabilities and Follow Actual Host Logs
        let root = tempfile::tempdir()?;
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut backend = v1::backend_client::BackendClient::connect(process.url.clone()).await?;
        let identity = backend.get_identity(v1::GetBackendIdentity {}).await?.into_inner();
        assert_eq!(
            identity.api.as_ref().unwrap().descriptor_sha256,
            <sha2::Sha256 as sha2::Digest>::digest(dokime_protocol::FILE_DESCRIPTOR_SET).to_vec()
        );
        let availability = backend.get_availability(v1::GetBackendAvailability {}).await?.into_inner();
        assert!(availability.methods.iter().any(|method| method == "/dokime.v1.Backend/CollectDiagnostics"));
        let mut logs = backend
            .watch_logs(v1::WatchBackendLogs {
                replay_lines: 1,
                min_level: v1::LogLevel::Info as i32,
            })
            .await?
            .into_inner();
        assert!(matches!(logs.message().await?.unwrap().value, Some(v1::backend_watch_logs_output::Value::Item(_))));
        let mut plugin =
            dokime_runtime_next::PluginProcess::start(&super::support::thermal_fixture()?, true, &super::support::settings().placement)
                .await?;
        let mut plugins = v1::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        let entry = plugins.add(v1::AddRemotePlugin { url: plugin.endpoint().into() }).await?.into_inner();
        loop {
            let frame = logs.message().await?.ok_or("Log Watch Closed Before Plugin Admission")?;
            if let Some(v1::backend_watch_logs_output::Value::Item(item)) = frame.value
                && item.fields.get("plugin").is_some_and(|identifier| identifier == &entry.description.as_ref().unwrap().id)
            {
                break;
            }
        }
        drop(logs);

        // Page Trace Events with a Process-Bound Resume Sequence
        let scope = Some(v1::TraceScope {
            kind: Some(v1::trace_scope::Kind::Instance(v1::InstanceTraceScope {})),
        });
        let mut trace = backend
            .watch_trace(v1::WatchBackendTrace {
                scope: scope.clone(),
                max_events: 1,
                ..Default::default()
            })
            .await?
            .into_inner();
        let first = trace.message().await?.unwrap();
        let Some(v1::backend_watch_trace_output::Value::Item(batch)) = first.value else {
            panic!("Expected Trace Batch");
        };
        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.epoch, identity.epoch);
        assert!(matches!(trace.message().await?.unwrap().value, Some(v1::backend_watch_trace_output::Value::Response(_))));
        assert!(trace.message().await?.is_none());
        assert!(
            backend
                .watch_trace(v1::WatchBackendTrace {
                    scope,
                    after_sequence: batch.next_sequence,
                    ..Default::default()
                })
                .await
                .is_err()
        );

        // Download and Verify One Support Archive with Contiguous Chunks
        let mut output = backend.collect_diagnostics(v1::CollectBackendDiagnostics { max_log_tail_bytes: None }).await?.into_inner();
        let mut bytes = Vec::new();
        let mut complete = None;
        let mut terminal = false;
        while let Some(frame) = output.message().await? {
            match frame.value.unwrap() {
                v1::backend_collect_diagnostics_output::Value::Item(frame) => match frame.payload.unwrap() {
                    v1::diagnostics_bundle_frame::Payload::Header(header) => {
                        assert_eq!(header.content_type, "application/zip");
                        assert!(!header.entries.is_empty());
                    }
                    v1::diagnostics_bundle_frame::Payload::Chunk(chunk) => {
                        assert_eq!(chunk.offset, bytes.len() as u64);
                        bytes.extend(chunk.data);
                    }
                    v1::diagnostics_bundle_frame::Payload::Complete(value) => complete = Some(value),
                },
                v1::backend_collect_diagnostics_output::Value::Response(_) => terminal = true,
            }
        }
        assert!(terminal);
        let complete = complete.unwrap();
        assert_eq!(complete.size_bytes, bytes.len() as u64);
        use sha2::Digest;
        assert_eq!(complete.sha256, sha2::Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
        assert!(bytes.starts_with(b"PK"));
        process.close().await?;
        plugin.close().await?;

        // Refuse an Incompatible Root While Preserving Every Existing Byte
        let incompatible = tempfile::tempdir()?;
        std::fs::write(incompatible.path().join("existing-data"), b"preserve")?;
        let output = tokio::process::Command::from(dokime_runtime_next::process::command(super::support::host_fixture()?))
            .args(["--endpoint", "127.0.0.1:0", "--storage"])
            .arg(incompatible.path())
            .output()
            .await?;
        assert!(!output.status.success());
        assert_eq!(std::fs::read(incompatible.path().join("existing-data"))?, b"preserve");
        assert!(matches!(dokime_storage::Store::open(incompatible.path()), Err(dokime_storage::Error::Incompatible(_))));
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
