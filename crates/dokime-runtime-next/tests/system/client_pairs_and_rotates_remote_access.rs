/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Pairs And Rotates Remote Access

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

#[cfg(not(target_os = "linux"))]
#[derive(Clone, PartialEq, prost::Message)]
struct Cleanup {
    #[prost(string, tag = "3")]
    key_slot: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn client_pairs_and_rotates_remote_access() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        use dokime::v1;

        // Export a Pairing File and Connect Through Its Pinned Certificate
        let root = tempfile::tempdir()?;
        let storage = root.path().join("storage");
        let credentials = super::support::CredentialStore::start(root.path()).await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&storage, &credentials.environment).await?;
        let mut remote = v1::remote_client::RemoteClient::connect(process.url.clone()).await?;
        assert!(!remote.read_listener(v1::RemoteReadListenerRequest {}).await?.into_inner().settings.unwrap().enabled);
        let first =
            remote.create_pairing(v1::RemoteCreatePairingRequest { label: "Story Desktop".into() }).await?.into_inner().pairing.unwrap();
        let tls = tonic::transport::ClientTlsConfig::new()
            .ca_certificate(tonic::transport::Certificate::from_pem(first.certificate_pem.clone().unwrap()));
        let channel = tonic::transport::Endpoint::from_shared(first.endpoints[0].clone())?.tls_config(tls)?.connect().await?;
        let mut backend = v1::backend_client::BackendClient::new(channel.clone());
        assert_eq!(backend.get_identity(v1::GetBackendIdentity {}).await.unwrap_err().code(), tonic::Code::Unauthenticated);
        let mut request = tonic::Request::new(v1::GetBackendIdentity {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", first.access_key.as_ref().unwrap()).parse()?);
        let identity = backend.get_identity(request).await?.into_inner();
        assert_eq!(identity.backend_id, first.backend_identifier);

        // The Same Authorization Protects Settings and Plugin Pages
        let mut settings = v1::settings_client::SettingsClient::new(channel.clone());
        assert_eq!(
            settings.read_application_settings(v1::SettingsReadApplicationSettingsRequest {}).await.unwrap_err().code(),
            tonic::Code::Unauthenticated,
        );
        let mut request = tonic::Request::new(v1::SettingsReadApplicationSettingsRequest {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", first.access_key.as_ref().unwrap()).parse()?);
        assert!(settings.read_application_settings(request).await?.into_inner().settings.is_some());
        let mut plugins = v1::plugins_client::PluginsClient::new(channel.clone());
        assert_eq!(plugins.list(v1::ListPlugins {}).await.unwrap_err().code(), tonic::Code::Unauthenticated);
        let mut request = tonic::Request::new(v1::ListPlugins {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", first.access_key.as_ref().unwrap()).parse()?);
        assert!(plugins.list(request).await?.into_inner().plugins.is_empty());
        drop(settings);
        drop(plugins);

        // Execution Services Use the Same Authenticated Route Composer
        let key = format!("Bearer {}", first.access_key.as_ref().unwrap());
        let mut executions = v1::execution_client::ExecutionClient::new(channel.clone());
        assert_eq!(
            executions.create_cycle(v1::ExecutionCreateCycleRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(v1::ExecutionCreateCycleRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        let cycle = executions.create_cycle(request).await?.into_inner();
        assert!(!cycle.cycle_id.is_empty());

        let mut planning = v1::planning_client::PlanningClient::new(channel.clone());
        assert_eq!(
            planning.preview_selection(v1::PlanningPreviewSelectionRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(v1::PlanningPreviewSelectionRequest { plan: Some(v1::SelectionPlan::default()) });
        request.metadata_mut().insert("authorization", key.parse()?);
        assert_eq!(planning.preview_selection(request).await.unwrap_err().code(), tonic::Code::InvalidArgument);

        let mut observation = v1::observation_client::ObservationClient::new(channel.clone());
        assert_eq!(
            observation.get_runtime_shell(v1::ObservationGetRuntimeShellRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(v1::ObservationGetRuntimeShellRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        observation.get_runtime_shell(request).await?;

        let mut prompts = v1::prompts_client::PromptsClient::new(channel.clone());
        assert_eq!(
            prompts.list_definitions(v1::PromptsListDefinitionsRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(v1::PromptsListDefinitionsRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        assert!(prompts.list_definitions(request).await?.into_inner().definitions.is_empty());

        let mut history = v1::history_client::HistoryClient::new(channel.clone());
        assert_eq!(
            history.browse_cycles(v1::HistoryBrowseCyclesRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(v1::HistoryBrowseCyclesRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        history.browse_cycles(request).await?;
        drop((executions, planning, observation, prompts, history));

        // Every Data Service Uses the Same Authenticated Route Composer
        let mut evidence = v1::evidence_client::EvidenceClient::new(channel.clone());
        assert_eq!(
            evidence.list_records(v1::EvidenceListRecordsRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(v1::EvidenceListRecordsRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        evidence.list_records(request).await?;
        drop(evidence);

        let mut telemetry = v1::telemetry_client::TelemetryClient::new(channel.clone());
        assert_eq!(
            telemetry.list_samples(v1::TelemetryListSamplesRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(v1::TelemetryListSamplesRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        assert_eq!(telemetry.list_samples(request).await.unwrap_err().code(), tonic::Code::InvalidArgument);
        drop(telemetry);

        let mut plots = v1::plots_client::PlotsClient::new(channel.clone());
        assert_eq!(plots.list_analyses(v1::PlotsListAnalysesRequest::default()).await.unwrap_err().code(), tonic::Code::Unauthenticated);
        let mut request = tonic::Request::new(v1::PlotsListAnalysesRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        plots.list_analyses(request).await?;
        drop(plots);

        let mut documents = v1::documents_client::DocumentsClient::new(channel.clone());
        assert_eq!(
            documents.update_plot_settings(v1::DocumentsUpdatePlotSettingsRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(v1::DocumentsUpdatePlotSettingsRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        assert_eq!(documents.update_plot_settings(request).await.unwrap_err().code(), tonic::Code::InvalidArgument);
        drop(documents);

        let mut reports = v1::reports_client::ReportsClient::new(channel.clone());
        assert_eq!(
            reports.materialize_report(v1::ReportsMaterializeReportRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(v1::ReportsMaterializeReportRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        assert_eq!(reports.materialize_report(request).await.unwrap_err().code(), tonic::Code::InvalidArgument);
        drop(reports);

        let mut content = v1::content_client::ContentClient::new(channel.clone());
        assert_eq!(
            content.list_artifacts(v1::ContentListArtifactsRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(v1::ContentListArtifactsRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        assert!(content.list_artifacts(request).await?.into_inner().artifacts.is_empty());
        drop(content);

        let mut notes = v1::notes_client::NotesClient::new(channel.clone());
        assert_eq!(notes.list_notes(v1::NotesListNotesRequest::default()).await.unwrap_err().code(), tonic::Code::Unauthenticated);
        let mut request = tonic::Request::new(v1::NotesListNotesRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        notes.list_notes(request).await?;
        drop(notes);

        // A New Export Invalidates the Old Key on an Already Open Channel
        let second = remote
            .create_pairing(v1::RemoteCreatePairingRequest { label: "Replacement Desktop".into() })
            .await?
            .into_inner()
            .pairing
            .unwrap();
        assert_ne!(second.access_key, first.access_key);
        let mut old = tonic::Request::new(v1::GetBackendIdentity {});
        old.metadata_mut().insert("authorization", format!("Bearer {}", first.access_key.unwrap()).parse()?);
        assert_eq!(backend.get_identity(old).await.unwrap_err().code(), tonic::Code::Unauthenticated);
        let mut current = tonic::Request::new(v1::GetBackendIdentity {});
        current.metadata_mut().insert("authorization", format!("Bearer {}", second.access_key.as_ref().unwrap()).parse()?);
        assert_eq!(backend.get_identity(current).await?.into_inner().backend_id, identity.backend_id);
        drop(backend);
        drop(channel);
        process.close().await?;

        // Restart Retains the Access Key and Certificate, Then Rotate the Certificate
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&storage, &credentials.environment).await?;
        let mut remote = v1::remote_client::RemoteClient::connect(process.url.clone()).await?;
        let status = remote.read_listener(v1::RemoteReadListenerRequest {}).await?.into_inner();
        assert!(status.key_configured);
        assert_eq!(status.certificate_pem, second.certificate_pem);
        let address = format!("https://{}", status.listening_address.as_ref().unwrap());
        let tls = tonic::transport::ClientTlsConfig::new()
            .ca_certificate(tonic::transport::Certificate::from_pem(second.certificate_pem.clone().unwrap()));
        let channel = tonic::transport::Endpoint::from_shared(address)?.tls_config(tls)?.connect().await?;
        let mut backend = v1::backend_client::BackendClient::new(channel);
        let mut request = tonic::Request::new(v1::GetBackendIdentity {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", second.access_key.unwrap()).parse()?);
        assert_eq!(backend.get_identity(request).await?.into_inner().backend_id, identity.backend_id);
        drop(backend);
        let rotated = remote
            .regenerate_certificate(v1::RemoteRegenerateCertificateRequest {
                certificate_names: status.settings.as_ref().unwrap().certificate_names.clone(),
            })
            .await?
            .into_inner();
        assert_ne!(rotated.certificate_pem, status.certificate_pem);
        let disabled = remote
            .update_listener(v1::RemoteUpdateListenerRequest {
                settings: Some(v1::RemoteListenerSettings {
                    enabled: false,
                    ..rotated.settings.unwrap()
                }),
            })
            .await?
            .into_inner();
        assert!(disabled.listening_address.is_none());
        process.close().await?;

        // Remove Only This Disposable Story's Protected TLS Key
        #[cfg(not(target_os = "linux"))]
        {
            let store = dokime_storage::Store::open(&storage)?;
            let security: Cleanup = store.administration("remote-security")?.unwrap();
            keyring::Entry::new("dokime.remote-tls", &format!("{}:{}", identity.backend_id, security.key_slot))?.delete_credential()?;
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
