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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn client_pairs_and_rotates_remote_access() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        use harmos_protocol::v1::access;

        // Export a Pairing File and Connect Through Its Pinned Certificate
        let root = super::support::tempdir()?;
        let storage = root.path().to_owned();
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&storage, &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let limits = dokime::client::ClientSettings {
            transfer: harmos_protocol::storage::StorageTransferSettings { chunk_bytes: Some(1024) },
            retained_bytes: std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            message_bytes: std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            input_items: std::num::NonZeroUsize::new(8).unwrap(),
            input_bytes: std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        };
        let remote = dokime::Client::from_channel(channel, limits)?;
        let current = remote.access.listener.get(tonic::Request::new(access::ListenerGetRequest {})).await?.into_inner();
        let mut value = current.value.unwrap();
        assert!(!value.enabled);
        value.enabled = true;
        remote
            .access
            .listener
            .set(tonic::Request::new(access::ListenerSetRequest {
                value: Some(value),
                expected_revision: current.revision,
            }))
            .await?;
        let first = remote
            .access
            .pairings
            .create(tonic::Request::new(access::PairingsCreateRequest { label: "Story Desktop".into() }))
            .await?
            .into_inner();
        let tls = tonic::transport::ClientTlsConfig::new()
            .ca_certificate(tonic::transport::Certificate::from_pem(first.pinned_certificate_pem.clone()));
        let channel = tonic::transport::Endpoint::from_shared(first.endpoint.clone())?.tls_config(tls)?.connect().await?;
        let backend = dokime::Client::from_channel(channel.clone(), limits)?;
        assert_eq!(
            backend.status.get(tonic::Request::new(harmos_protocol::v1::StatusGetRequest {})).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(harmos_protocol::v1::StatusGetRequest {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", first.credential).parse()?);
        let identity = backend.status.get(request).await?.into_inner().status.unwrap().identity.unwrap();
        assert_eq!(identity.host_id, std::fs::read_to_string(storage.join("backend-id"))?);

        // The Same Authorization Protects Settings and Plugin Pages
        let mut settings = dokime_protocol::client::settings_client::SettingsClient::new(channel.clone());
        assert_eq!(
            settings.get(dokime_protocol::client::SettingsGetRequest { ..Default::default() }).await.unwrap_err().code(),
            tonic::Code::Unauthenticated,
        );
        let mut request = tonic::Request::new(dokime_protocol::client::SettingsGetRequest { ..Default::default() });
        request.metadata_mut().insert("authorization", format!("Bearer {}", first.credential).parse()?);
        assert!(settings.get(request).await?.into_inner().value.is_some());
        let mut plugins = dokime_protocol::client::plugins_client::PluginsClient::new(channel.clone());
        assert_eq!(plugins.list(dokime_protocol::client::ListPlugins {}).await.unwrap_err().code(), tonic::Code::Unauthenticated);
        let mut request = tonic::Request::new(dokime_protocol::client::ListPlugins {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", first.credential).parse()?);
        assert!(plugins.list(request).await?.into_inner().plugins.is_empty());
        drop(settings);
        drop(plugins);

        // Execution Services Use the Same Authenticated Route Composer
        let key = format!("Bearer {}", first.credential);
        let executions = dokime_protocol::client::executions_client::ExecutionsClient::new(channel.clone());
        assert_eq!(
            dokime_protocol::client::cycles_client::CyclesClient::new(channel.clone())
                .create(dokime_protocol::client::CyclesCreateRequest::default())
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::CyclesCreateRequest {
            request_id: "authorized-cycle".into(),
            ..Default::default()
        });
        request.metadata_mut().insert("authorization", key.parse()?);
        let cycle = dokime_protocol::client::cycles_client::CyclesClient::new(channel.clone()).create(request).await?.into_inner();
        assert!(!cycle.cycle.unwrap().cycle_id.is_empty());

        let mut planning = dokime_protocol::client::cases_client::CasesClient::new(channel.clone());
        assert_eq!(
            planning.create(dokime_protocol::client::CasesCreateRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::CasesCreateRequest {
            plan: Some(dokime_protocol::client::SelectionPlan::default()),
            ..Default::default()
        });
        request.metadata_mut().insert("authorization", key.parse()?);
        assert_eq!(planning.create(request).await.unwrap_err().code(), tonic::Code::InvalidArgument);

        let observation = dokime_protocol::client::backend::overview_client::OverviewClient::new(channel.clone());
        assert_eq!(
            dokime_protocol::client::backend::overview_client::OverviewClient::new(channel.clone())
                .get(dokime_protocol::client::backend::OverviewGetRequest::default())
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::backend::OverviewGetRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        dokime_protocol::client::backend::overview_client::OverviewClient::new(channel.clone()).get(request).await?;

        let prompts = dokime_protocol::client::prompts_client::PromptsClient::new(channel.clone());
        assert_eq!(
            dokime_protocol::client::prompt_definitions_client::PromptDefinitionsClient::new(channel.clone())
                .list(dokime_protocol::client::PromptDefinitionsListRequest::default())
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::PromptDefinitionsListRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        assert!(
            dokime_protocol::client::prompt_definitions_client::PromptDefinitionsClient::new(channel.clone())
                .list(request)
                .await?
                .into_inner()
                .values
                .is_empty()
        );

        let mut history = dokime_protocol::client::cycles_client::CyclesClient::new(channel.clone());
        assert_eq!(
            history.list(dokime_protocol::client::CyclesListRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::CyclesListRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        history.list(request).await?;
        drop((executions, planning, observation, prompts, history));

        // Every Data Service Uses the Same Authenticated Route Composer
        let evidence = dokime_protocol::client::evidence_client::EvidenceClient::new(channel.clone());
        assert_eq!(
            dokime_protocol::client::evidence::records_client::RecordsClient::new(channel.clone())
                .list(dokime_protocol::client::evidence::RecordsListRequest::default())
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::evidence::RecordsListRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        dokime_protocol::client::evidence::records_client::RecordsClient::new(channel.clone()).list(request).await?;
        drop(evidence);

        let telemetry = dokime_protocol::client::evidence::samples_client::SamplesClient::new(channel.clone());
        assert_eq!(
            dokime_protocol::client::evidence::samples_client::SamplesClient::new(channel.clone())
                .list(dokime_protocol::client::evidence::SamplesListRequest::default())
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::evidence::SamplesListRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        assert_eq!(
            dokime_protocol::client::evidence::samples_client::SamplesClient::new(channel.clone()).list(request).await.unwrap_err().code(),
            tonic::Code::InvalidArgument
        );
        drop(telemetry);

        let plots = dokime_protocol::client::plots::snapshots_client::SnapshotsClient::new(channel.clone());
        assert_eq!(
            dokime_protocol::client::analyses_client::AnalysesClient::new(channel.clone())
                .list(dokime_protocol::client::AnalysesListRequest::default())
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::AnalysesListRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        dokime_protocol::client::analyses_client::AnalysesClient::new(channel.clone()).list(request).await?;
        drop(plots);

        let mut documents = dokime_protocol::client::plots::settings_client::SettingsClient::new(channel.clone());
        assert_eq!(
            documents.set(dokime_protocol::client::plots::SettingsSetRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::plots::SettingsSetRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        assert_eq!(documents.set(request).await.unwrap_err().code(), tonic::Code::InvalidArgument);
        drop(documents);

        let mut reports = dokime_protocol::client::reports_client::ReportsClient::new(channel.clone());
        assert_eq!(
            reports.create(dokime_protocol::client::ReportsCreateRequest::default()).await.unwrap_err().code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::ReportsCreateRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        assert_eq!(reports.create(request).await.unwrap_err().code(), tonic::Code::InvalidArgument);
        drop(reports);

        let content = dokime_protocol::client::content_client::ContentClient::new(channel.clone());
        assert_eq!(
            dokime_protocol::client::content::artifacts_client::ArtifactsClient::new(channel.clone())
                .list(dokime_protocol::client::content::ArtifactsListRequest::default())
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::content::ArtifactsListRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        assert!(
            dokime_protocol::client::content::artifacts_client::ArtifactsClient::new(channel.clone())
                .list(request)
                .await?
                .into_inner()
                .artifacts
                .is_empty()
        );
        drop(content);

        let notes = dokime_protocol::client::evidence::notes_client::NotesClient::new(channel.clone());
        assert_eq!(
            dokime_protocol::client::evidence::notes_client::NotesClient::new(channel.clone())
                .list(dokime_protocol::client::evidence::NotesListRequest::default())
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unauthenticated
        );
        let mut request = tonic::Request::new(dokime_protocol::client::evidence::NotesListRequest::default());
        request.metadata_mut().insert("authorization", key.parse()?);
        dokime_protocol::client::evidence::notes_client::NotesClient::new(channel.clone()).list(request).await?;
        drop(notes);

        // Independent Pairings Remain Valid Until Explicit Revocation
        let second = remote
            .access
            .pairings
            .create(tonic::Request::new(access::PairingsCreateRequest { label: "Replacement Desktop".into() }))
            .await?
            .into_inner();
        let listed = remote.access.pairings.list(tonic::Request::new(access::PairingsListRequest { at: None })).await?.into_inner();
        assert_eq!(listed.pairings.len(), 2);
        let mut request = tonic::Request::new(harmos_protocol::v1::StatusGetRequest {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", first.credential).parse()?);
        backend.status.get(request).await?;
        remote
            .access
            .pairings
            .delete(tonic::Request::new(access::PairingsDeleteRequest {
                name: first.pairing.as_ref().unwrap().id.clone(),
            }))
            .await?;
        assert_ne!(second.credential, first.credential);
        let mut old = tonic::Request::new(harmos_protocol::v1::StatusGetRequest {});
        old.metadata_mut().insert("authorization", format!("Bearer {}", first.credential).parse()?);
        assert_eq!(backend.status.get(old).await.unwrap_err().code(), tonic::Code::Unauthenticated);
        let mut current = tonic::Request::new(harmos_protocol::v1::StatusGetRequest {});
        current.metadata_mut().insert("authorization", format!("Bearer {}", second.credential).parse()?);
        assert_eq!(backend.status.get(current).await?.into_inner().status.unwrap().identity.unwrap().host_id, identity.host_id);
        drop(backend);
        drop(channel);
        process.close().await?;

        // Restart Retains the Access Key and Certificate, Then Rotate the Certificate
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&storage, &credentials.environment).await?;
        let management = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let remote = dokime::Client::from_channel(management, limits)?;
        let status = remote.access.listener.get(tonic::Request::new(access::ListenerGetRequest {})).await?.into_inner();
        assert!(status.value.as_ref().unwrap().enabled);
        let address = status.bound_endpoint.clone();
        let tls = tonic::transport::ClientTlsConfig::new()
            .ca_certificate(tonic::transport::Certificate::from_pem(second.pinned_certificate_pem.clone()));
        let channel = tonic::transport::Endpoint::from_shared(address)?.tls_config(tls)?.connect().await?;
        let backend = dokime::Client::from_channel(channel.clone(), limits)?;
        let mut request = tonic::Request::new(harmos_protocol::v1::StatusGetRequest {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", second.credential).parse()?);
        assert_eq!(backend.status.get(request).await?.into_inner().status.unwrap().identity.unwrap().host_id, identity.host_id);

        // Rotate Live Trust; Existing Connections Survive but Revoked Credentials Cannot Authorize
        let rotated = remote.access.certificates.create(tonic::Request::new(access::CertificatesCreateRequest {})).await?.into_inner();
        assert_ne!(rotated.certificate_pem, second.pinned_certificate_pem);
        let mut request = tonic::Request::new(harmos_protocol::v1::StatusGetRequest {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", second.credential).parse()?);
        assert_eq!(backend.status.get(request).await.unwrap_err().code(), tonic::Code::Unauthenticated);
        let fresh = remote
            .access
            .pairings
            .create(tonic::Request::new(access::PairingsCreateRequest { label: "Rotated Desktop".into() }))
            .await?
            .into_inner();
        let mut request = tonic::Request::new(harmos_protocol::v1::StatusGetRequest {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", fresh.credential).parse()?);
        assert_eq!(backend.status.get(request).await?.into_inner().status.unwrap().identity.unwrap().host_id, identity.host_id);
        let tls =
            tonic::transport::ClientTlsConfig::new().ca_certificate(tonic::transport::Certificate::from_pem(fresh.pinned_certificate_pem));
        let fresh_channel = tonic::transport::Endpoint::from_shared(fresh.endpoint)?.tls_config(tls)?.connect().await?;
        let fresh_backend = dokime::Client::from_channel(fresh_channel, limits)?;
        let mut request = tonic::Request::new(harmos_protocol::v1::StatusGetRequest {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", fresh.credential).parse()?);
        assert_eq!(fresh_backend.status.get(request).await?.into_inner().status.unwrap().identity.unwrap().host_id, identity.host_id);
        drop((backend, fresh_backend, channel));
        let mut value = status.value.unwrap();
        value.enabled = false;
        let disabled = remote
            .access
            .listener
            .set(tonic::Request::new(access::ListenerSetRequest {
                value: Some(value),
                expected_revision: status.revision,
            }))
            .await?
            .into_inner();
        assert!(disabled.bound_endpoint.is_empty());
        drop(remote);
        process.close().await?;

        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
