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
use dokime::Client;
use dokime::ClientSettings;
use dokime::client::AccessPairingsListOptions;
use dokime::client::AnalysesListOptions;
use dokime::client::BackendOverviewGetOptions;
use dokime::client::CallOptions;
use dokime::client::CasesCreateOptions;
use dokime::client::ContentArtifactsListOptions;
use dokime::client::CyclesCreateOptions;
use dokime::client::CyclesListOptions;
use dokime::client::EvidenceNotesListOptions;
use dokime::client::EvidenceRecordsListOptions;
use dokime::client::EvidenceSamplesListOptions;
use dokime::client::FilesReadOptions;
use dokime::client::PluginsListOptions;
use dokime::client::PromptDefinitionsListOptions;
use dokime::client::ReportsCreateOptions;
use dokime::client::SelectionPlan;
use dokime::client::SettingsGetOptions;
use dokime::client::SettingsSetOptions;
use dokime::client::StatusGetOptions;
use dokime_protocol::host::AnalysesListRequest;
use dokime_protocol::host::CasesCreateRequest;
use dokime_protocol::host::CyclesCreateRequest;
use dokime_protocol::host::CyclesListRequest;
use dokime_protocol::host::FilesReadRequest;
use dokime_protocol::host::PromptDefinitionsListRequest;
use dokime_protocol::host::ReportsCreateRequest;
use dokime_protocol::host::SettingsGetRequest;
use dokime_protocol::host::SettingsSetRequest;
use dokime_protocol::host::backend::OverviewGetRequest;
use dokime_protocol::host::content::ArtifactsListRequest;
use dokime_protocol::host::evidence::NotesListRequest;
use dokime_protocol::host::evidence::RecordsListRequest;
use dokime_protocol::host::evidence::SamplesListRequest;
use harmos_protocol::host::PluginsListRequest;
use harmos_protocol::model::StorageTransferSettings;
use std::error::Error;
use std::num::NonZeroUsize;
use tonic::Code;
use tonic::transport::Certificate;
use tonic::transport::ClientTlsConfig;
use tonic::transport::Endpoint;

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

#[test]
fn client_pairs_and_rotates_remote_access() -> Result<(), Box<dyn Error>> {
    super::support::run(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Export a Pairing File and Connect Through Its Pinned Certificate
            let root = super::support::tempdir()?;
            let storage = root.path().to_owned();
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let process =
                super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&storage, &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let limits = ClientSettings {
                transfer: StorageTransferSettings { chunk_bytes: Some(1024) },
                retained_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
                message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
                input_items: NonZeroUsize::new(8).unwrap(),
                input_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            };
            let remote = Client::from_channel(channel, limits)?;
            let current = remote.access.listener.get().await?;
            let mut value = current.value.unwrap();
            assert!(!value.enabled);
            value.enabled = true;
            remote.access.listener.set(value, current.revision).await?;
            let first = remote.access.pairings.create("Story Desktop").await?;
            let tls = ClientTlsConfig::new().ca_certificate(Certificate::from_pem(first.pinned_certificate_pem.clone()));
            let channel = Endpoint::from_shared(first.endpoint.clone())?.tls_config(tls)?.connect().await?;
            let backend = Client::from_channel(channel.clone(), limits)?;
            assert_eq!(backend.status.get().await.unwrap_err().code(), Code::Unauthenticated);
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", format!("Bearer {}", first.credential).parse()?);
            let identity = backend.status.get_with(StatusGetOptions { context: context_request }).await?.status.unwrap().identity.unwrap();
            assert_eq!(identity.host_id, std::fs::read_to_string(storage.join("backend-id"))?);

            // The Same Authorization Protects Settings and Plugin Pages
            let settings = Client::from_channel(channel.clone(), super::support::client_settings())?.settings.clone();
            assert_eq!(settings.get().await.unwrap_err().code(), Code::Unauthenticated,);
            let request = SettingsGetRequest { ..Default::default() };
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", format!("Bearer {}", first.credential).parse()?);
            assert!(
                settings
                    .get_with(SettingsGetOptions {
                        at: (request.clone()).at,
                        context: context_request,
                    })
                    .await?
                    .value
                    .is_some()
            );
            let plugins = Client::from_channel(channel.clone(), super::support::client_settings())?.plugins.clone();
            assert_eq!(plugins.list().await.unwrap_err().code(), Code::Unauthenticated);
            let request = PluginsListRequest::default();
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", format!("Bearer {}", first.credential).parse()?);
            assert!(plugins.list_with(PluginsListOptions { at: request.at, context: context_request }).await?.entries.is_empty());
            drop(settings);
            drop(plugins);

            // Execution Services Use the Same Authenticated Route Composer
            let key = format!("Bearer {}", first.credential);
            let executions = Client::from_channel(channel.clone(), super::support::client_settings())?.executions.clone();
            assert_eq!(
                Client::from_channel(channel.clone(), super::support::client_settings())?
                    .cycles
                    .clone()
                    .create(Default::default(), Default::default(), Default::default(), String::new())
                    .await
                    .unwrap_err()
                    .code(),
                Code::Unauthenticated
            );
            let request = CyclesCreateRequest {
                request_id: "authorized-cycle".into(),
                metadata: Some(dokime::Metadata {
                    label: "Authorized Cycle".into(),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            let cycle = Client::from_channel(channel.clone(), super::support::client_settings())?
                .cycles
                .clone()
                .create_with(
                    (request.clone()).origins,
                    ((request.clone()).metadata).unwrap_or_default(),
                    (request.clone()).continue_automatically,
                    (request.clone()).source_cycle,
                    CyclesCreateOptions {
                        request_id: Some((request.clone()).request_id),
                        context: context_request,
                    },
                )
                .await?;
            let cycle = cycle.cycle.unwrap().cycle_id;
            assert!(!cycle.is_empty());

            let planning = Client::from_channel(channel.clone(), super::support::client_settings())?.cases.clone();
            assert_eq!(planning.create(Default::default(), Default::default()).await.unwrap_err().code(), Code::Unauthenticated);
            let request = CasesCreateRequest {
                plan: Some(SelectionPlan::default()),
                ..Default::default()
            };
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            assert_eq!(
                planning
                    .create_with(
                        ((request.clone()).plan).unwrap_or_default(),
                        (request.clone()).validate_only,
                        CasesCreateOptions {
                            request_id: Some((request.clone()).request_id),
                            context: context_request,
                        }
                    )
                    .await
                    .unwrap_err()
                    .code(),
                Code::InvalidArgument
            );

            let observation = Client::from_channel(channel.clone(), super::support::client_settings())?.backend.overview.clone();
            assert_eq!(
                Client::from_channel(channel.clone(), super::support::client_settings())?
                    .backend
                    .overview
                    .clone()
                    .get()
                    .await
                    .unwrap_err()
                    .code(),
                Code::Unauthenticated
            );
            let request = OverviewGetRequest::default();
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            Client::from_channel(channel.clone(), super::support::client_settings())?
                .backend
                .overview
                .clone()
                .get_with(BackendOverviewGetOptions {
                    at: (request.clone()).at,
                    context: context_request,
                })
                .await?;

            let prompts = Client::from_channel(channel.clone(), super::support::client_settings())?.prompts.clone();
            assert_eq!(
                Client::from_channel(channel.clone(), super::support::client_settings())?
                    .prompt_definitions
                    .clone()
                    .list(String::new())
                    .await
                    .unwrap_err()
                    .code(),
                Code::Unauthenticated
            );
            let request = PromptDefinitionsListRequest::default();
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            assert!(
                Client::from_channel(channel.clone(), super::support::client_settings())?
                    .prompt_definitions
                    .clone()
                    .list_with(
                        (request.clone()).parent,
                        PromptDefinitionsListOptions {
                            at: (request.clone()).at,
                            context: context_request,
                        }
                    )
                    .await?
                    .values
                    .is_empty()
            );

            let history = Client::from_channel(channel.clone(), super::support::client_settings())?.cycles.clone();
            assert_eq!(
                history.list(Default::default(), Default::default(), Default::default()).await.unwrap_err().code(),
                Code::Unauthenticated
            );
            let request = CyclesListRequest::default();
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            history
                .list_with(
                    ((request.clone()).selection).unwrap_or_default(),
                    ((request.clone()).filter).unwrap_or_default(),
                    ((request.clone()).order).unwrap_or_default(),
                    CyclesListOptions {
                        at: (request.clone()).at,
                        page: (request.clone()).page,
                        context: context_request,
                    },
                )
                .await?;
            drop((executions, planning, observation, prompts, history));

            // Every Data Service Uses the Same Authenticated Route Composer
            let evidence = Client::from_channel(channel.clone(), super::support::client_settings())?.evidence.clone();
            assert_eq!(
                Client::from_channel(channel.clone(), super::support::client_settings())?
                    .evidence
                    .records
                    .clone()
                    .list_with(
                        Default::default(),
                        String::new(),
                        EvidenceRecordsListOptions {
                            versions: Some(Default::default()),
                            between: Some(Default::default()),
                            step: Some(String::new()),
                            selection: Some(Default::default()),
                            ..Default::default()
                        }
                    )
                    .await
                    .unwrap_err()
                    .code(),
                Code::Unauthenticated
            );
            let request = RecordsListRequest {
                selection: Some(dokime::EvidenceCapturedRecordSelection {
                    scope: Some(dokime::EvidenceScope {
                        cycle_id: Some(cycle.clone()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            Client::from_channel(channel.clone(), super::support::client_settings())?
                .evidence
                .records
                .clone()
                .list_with(
                    ((request.clone()).scope).unwrap_or_default(),
                    (request.clone()).record_type,
                    EvidenceRecordsListOptions {
                        versions: Some((request.clone()).versions),
                        between: Some(((request.clone()).between).unwrap_or_default()),
                        step: Some((request.clone()).step),
                        selection: Some(((request.clone()).selection).unwrap_or_default()),
                        at: (request.clone()).at,
                        page: (request.clone()).page,
                        context: context_request,
                    },
                )
                .await?;
            drop(evidence);

            let telemetry = Client::from_channel(channel.clone(), super::support::client_settings())?.evidence.samples.clone();
            assert_eq!(
                Client::from_channel(channel.clone(), super::support::client_settings())?
                    .evidence
                    .samples
                    .clone()
                    .list_with(
                        Default::default(),
                        Default::default(),
                        EvidenceSamplesListOptions {
                            versions: Some(Default::default()),
                            fields: Some(Default::default()),
                            sources: Some(Default::default()),
                            signals: Some(Default::default()),
                            between: Some(Default::default()),
                            bucket: Some(Default::default()),
                            latest: Some(Default::default()),
                            ..Default::default()
                        }
                    )
                    .await
                    .unwrap_err()
                    .code(),
                Code::Unauthenticated
            );
            let request = SamplesListRequest::default();
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            assert_eq!(
                Client::from_channel(channel.clone(), super::support::client_settings())?
                    .evidence
                    .samples
                    .clone()
                    .list_with(
                        ((request.clone()).scope).unwrap_or_default(),
                        (request.clone()).streams,
                        EvidenceSamplesListOptions {
                            versions: Some((request.clone()).versions),
                            fields: Some((request.clone()).fields),
                            sources: Some((request.clone()).sources),
                            signals: Some((request.clone()).signals),
                            between: Some(((request.clone()).between).unwrap_or_default()),
                            bucket: Some(((request.clone()).bucket).unwrap_or_default()),
                            latest: Some((request.clone()).latest),
                            at: (request.clone()).at,
                            page: (request.clone()).page,
                            context: context_request,
                        }
                    )
                    .await
                    .unwrap_err()
                    .code(),
                Code::InvalidArgument
            );
            drop(telemetry);

            let plots = Client::from_channel(channel.clone(), super::support::client_settings())?.plots.snapshots.clone();
            assert_eq!(
                Client::from_channel(channel.clone(), super::support::client_settings())?.analyses.clone().list().await.unwrap_err().code(),
                Code::Unauthenticated
            );
            let request = AnalysesListRequest::default();
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            Client::from_channel(channel.clone(), super::support::client_settings())?
                .analyses
                .clone()
                .list_with(AnalysesListOptions {
                    at: (request.clone()).at,
                    context: context_request,
                })
                .await?;
            drop(plots);

            let documents = Client::from_channel(channel.clone(), super::support::client_settings())?.settings.clone();
            assert_eq!(documents.set(Default::default(), Default::default()).await.unwrap_err().code(), Code::Unauthenticated);
            let request = SettingsSetRequest::default();
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            assert_eq!(
                documents
                    .set_with(
                        ((request.clone()).value).unwrap_or_default(),
                        (request.clone()).expected_revision,
                        SettingsSetOptions {
                            request_id: Some((request.clone()).request_id),
                            context: context_request,
                        }
                    )
                    .await
                    .unwrap_err()
                    .code(),
                Code::InvalidArgument
            );
            drop(documents);

            let reports = Client::from_channel(channel.clone(), super::support::client_settings())?.reports.clone();
            assert_eq!(reports.create(Default::default(), String::new()).await.unwrap_err().code(), Code::Unauthenticated);
            let request = ReportsCreateRequest::default();
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            assert_eq!(
                reports
                    .create_with(
                        ((request.clone()).subject).unwrap_or_default(),
                        (request.clone()).document_id,
                        ReportsCreateOptions {
                            at: (request.clone()).at,
                            request_id: Some((request.clone()).request_id),
                            context: context_request,
                        }
                    )
                    .await
                    .unwrap_err()
                    .code(),
                Code::InvalidArgument
            );
            drop(reports);

            let file_bytes = Client::from_channel(channel.clone(), super::support::client_settings())?.files.clone();
            assert_eq!(
                file_bytes.read(Default::default(), Default::default(), Default::default()).await.unwrap_err().code(),
                Code::Unauthenticated
            );
            let request = FilesReadRequest::default();
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            assert_eq!(
                file_bytes
                    .read_with(
                        ((request.clone()).file).unwrap_or_default(),
                        (request.clone()).offset,
                        (request.clone()).max_bytes,
                        FilesReadOptions {
                            at: (request.clone()).at,
                            context: context_request,
                        }
                    )
                    .await
                    .unwrap_err()
                    .code(),
                Code::InvalidArgument
            );
            drop(file_bytes);
            assert_eq!(
                Client::from_channel(channel.clone(), super::support::client_settings())?
                    .content
                    .artifacts
                    .clone()
                    .list(Default::default())
                    .await
                    .unwrap_err()
                    .code(),
                Code::Unauthenticated
            );
            let request = ArtifactsListRequest::default();
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            assert!(
                Client::from_channel(channel.clone(), super::support::client_settings())?
                    .content
                    .artifacts
                    .clone()
                    .list_with(
                        ((request.clone()).scope).unwrap_or_default(),
                        ContentArtifactsListOptions {
                            at: (request.clone()).at,
                            page: (request.clone()).page,
                            context: context_request,
                        }
                    )
                    .await?
                    .artifacts
                    .is_empty()
            );

            let notes = Client::from_channel(channel.clone(), super::support::client_settings())?.evidence.notes.clone();
            assert_eq!(
                Client::from_channel(channel.clone(), super::support::client_settings())?
                    .evidence
                    .notes
                    .clone()
                    .list(Default::default())
                    .await
                    .unwrap_err()
                    .code(),
                Code::Unauthenticated
            );
            let request = NotesListRequest::default();
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", key.parse()?);
            Client::from_channel(channel.clone(), super::support::client_settings())?
                .evidence
                .notes
                .clone()
                .list_with(
                    ((request.clone()).selection).unwrap_or_default(),
                    EvidenceNotesListOptions {
                        at: (request.clone()).at,
                        page: (request.clone()).page,
                        context: context_request,
                    },
                )
                .await?;
            drop(notes);

            // Independent Pairings Remain Valid Until Explicit Revocation
            let second = remote.access.pairings.create("Replacement Desktop").await?;
            let listed = remote.access.pairings.list_with(AccessPairingsListOptions { at: None, ..Default::default() }).await?;
            assert_eq!(listed.pairings.len(), 2);
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", format!("Bearer {}", first.credential).parse()?);
            backend.status.get_with(StatusGetOptions { context: context_request }).await?;
            remote.access.pairings.delete(first.pairing.as_ref().unwrap().id.clone()).await?;
            assert_ne!(second.credential, first.credential);
            let mut context_old = CallOptions::default();
            context_old.metadata.insert("authorization", format!("Bearer {}", first.credential).parse()?);
            assert_eq!(backend.status.get_with(StatusGetOptions { context: context_old }).await.unwrap_err().code(), Code::Unauthenticated);
            let mut context_current = CallOptions::default();
            context_current.metadata.insert("authorization", format!("Bearer {}", second.credential).parse()?);
            assert_eq!(
                backend.status.get_with(StatusGetOptions { context: context_current }).await?.status.unwrap().identity.unwrap().host_id,
                identity.host_id
            );
            drop(backend);
            drop(channel);
            process.close().await?;

            // Restart Retains the Access Key and Certificate, Then Rotate the Certificate
            let process =
                super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&storage, &credentials.environment).await?;
            let management = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let remote = Client::from_channel(management, limits)?;
            let status = remote.access.listener.get().await?;
            assert!(status.value.as_ref().unwrap().enabled);
            let address = status.bound_endpoint.clone();
            let tls = ClientTlsConfig::new().ca_certificate(Certificate::from_pem(second.pinned_certificate_pem.clone()));
            let channel = Endpoint::from_shared(address)?.tls_config(tls)?.connect().await?;
            let backend = Client::from_channel(channel.clone(), limits)?;
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", format!("Bearer {}", second.credential).parse()?);
            assert_eq!(
                backend.status.get_with(StatusGetOptions { context: context_request }).await?.status.unwrap().identity.unwrap().host_id,
                identity.host_id
            );

            // Rotate Live Trust; Existing Connections Survive but Revoked Credentials Cannot Authorize
            let rotated = remote.access.certificates.create().await?;
            assert_ne!(rotated, second.pinned_certificate_pem);
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", format!("Bearer {}", second.credential).parse()?);
            assert_eq!(
                backend.status.get_with(StatusGetOptions { context: context_request }).await.unwrap_err().code(),
                Code::Unauthenticated
            );
            let fresh = remote.access.pairings.create("Rotated Desktop").await?;
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", format!("Bearer {}", fresh.credential).parse()?);
            assert_eq!(
                backend.status.get_with(StatusGetOptions { context: context_request }).await?.status.unwrap().identity.unwrap().host_id,
                identity.host_id
            );
            let tls = ClientTlsConfig::new().ca_certificate(Certificate::from_pem(fresh.pinned_certificate_pem));
            let fresh_channel = Endpoint::from_shared(fresh.endpoint)?.tls_config(tls)?.connect().await?;
            let fresh_backend = Client::from_channel(fresh_channel, limits)?;
            let mut context_request = CallOptions::default();
            context_request.metadata.insert("authorization", format!("Bearer {}", fresh.credential).parse()?);
            assert_eq!(
                fresh_backend
                    .status
                    .get_with(StatusGetOptions { context: context_request })
                    .await?
                    .status
                    .unwrap()
                    .identity
                    .unwrap()
                    .host_id,
                identity.host_id
            );
            drop((backend, fresh_backend, channel));
            let mut value = status.value.unwrap();
            value.enabled = false;
            let disabled = remote.access.listener.set(value, status.revision).await?;
            assert!(disabled.bound_endpoint.is_empty());
            drop(remote);
            process.close().await?;

            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    })
}
