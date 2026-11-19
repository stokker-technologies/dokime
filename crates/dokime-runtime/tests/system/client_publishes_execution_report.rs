/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Report Publication Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;

// Third Party Imports
use sha2::{Digest, Sha256};
use tonic::transport::Endpoint;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime_client_internal::Client;
use dokime_client_internal::ExecutionsGetOptions;
use dokime_client_internal::ReportOrigin;
use dokime_client_internal::ReportState;
use dokime_client_internal::ReportSubject;
use dokime_client_internal::ReportsCreateOptions;
use dokime_client_internal::ReportsPublishOptions;
use dokime_protocol::host::ReportsPublishRequest;
use dokime_protocol::host::report_subject;
use dokime_runtime::Host;
use dokime_storage::Store;

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
fn client_publishes_execution_report() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Prepare Sealed Evidence and Its Selected Document
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
            let entry = host.add_managed("execution", &super::support::execution_fixture()?, &[]).await?;
            let document = host.catalog(&entry)?.suites[0].templates[0].documents[0].clone();
            let origin = host.materialize(&entry, "execution.capture", super::support::parameters(23.0, 0, true)).await?;
            let cycle = host.start_cycle(vec![origin]).await?;
            let identifier = cycle.executions[0].clone();
            let prompt = host.next_prompt(&identifier).await?;
            super::support::answer(&host, &identifier, prompt)?;
            cycle.wait().await?;
            let before = host.execution(&identifier)?;
            host.shutdown().await?;
            drop(host);
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
            let reports = &client.reports;
            let subject = ReportSubject {
                kind: Some(report_subject::Kind::Execution(identifier.clone())),
            };
            let preview = reports
                .create_with(
                    subject.clone(),
                    document.metadata.as_ref().map_or(String::new(), |metadata| metadata.identifier.clone()),
                    ReportsCreateOptions {
                        request_id: Some("preview".into()),
                        ..Default::default()
                    },
                )
                .await?;
            // Replay Returns the Original Metadata, Cut, Receipt and File Without Recollecting.
            let replay = reports
                .create_with(
                    subject.clone(),
                    document.metadata.as_ref().map_or(String::new(), |metadata| metadata.identifier.clone()),
                    ReportsCreateOptions {
                        request_id: Some("preview".into()),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(replay, preview);
            let file = preview.file.clone().unwrap();
            assert_eq!(preview.report.as_ref().unwrap().origin, ReportOrigin::Current as i32);
            let store = Store::open(root.path())?;
            assert!(store.reports(&identifier)?.is_empty());
            assert_eq!(store.read(&identifier)?, before);
            assert!(
                reports
                    .create_with(
                        subject.clone(),
                        "missing",
                        ReportsCreateOptions {
                            request_id: Some("invalid-document".into()),
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            assert!(store.reports(&identifier)?.is_empty());
            // Publish From Durable Metadata and File Bytes After Restart
            process.close().await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
            let reports = &client.reports;
            let executions = &client.executions;
            let replay = reports
                .create_with(
                    subject.clone(),
                    document.metadata.as_ref().map_or(String::new(), |metadata| metadata.identifier.clone()),
                    ReportsCreateOptions {
                        request_id: Some("preview".into()),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(replay, preview);
            let current = executions.get_with(identifier.clone(), ExecutionsGetOptions { at: None, ..Default::default() }).await?;
            assert_eq!(current.execution.unwrap().report.unwrap().state, ReportState::Pending as i32);
            let request = ReportsPublishRequest {
                name: file.metadata.as_ref().unwrap().id.clone(),
                subject: Some(subject.clone()),
                expected_revision: file.sha256.clone(),
                request_id: "publish".into(),
            };
            let mut stale = request.clone();
            stale.expected_revision[0] ^= 1;
            assert!(
                reports
                    .publish_with(
                        stale.name,
                        stale.subject.unwrap_or_default(),
                        stale.expected_revision,
                        ReportsPublishOptions {
                            request_id: Some(stale.request_id),
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            assert!(store.reports(&identifier)?.is_empty());
            let metadata = reports
                .publish_with(
                    request.name.clone(),
                    request.subject.clone().unwrap_or_default(),
                    request.expected_revision.clone(),
                    ReportsPublishOptions {
                        request_id: Some(request.request_id.clone()),
                        ..Default::default()
                    },
                )
                .await?;
            let published = store.reports(&identifier)?;
            assert_eq!(published.len(), 1);
            assert_eq!(published[0].document.as_ref(), Some(&document));
            assert_eq!(published[0].identifier, metadata.report.as_ref().unwrap().identifier);
            assert_eq!(published[0].execution_seal, before.seal);
            assert_eq!(Sha256::digest(&published[0].html).as_slice(), file.sha256);
            let capture = super::documents_and_reports::decode_report(&published[0].html)?;
            assert_eq!(
                capture.executions[0].detail.as_ref().unwrap().execution.as_ref().unwrap().summary.as_ref().unwrap().execution_id,
                identifier
            );
            assert!(!capture.executions[0].plots.as_ref().unwrap().envelopes.is_empty());
            assert_eq!(store.read(&identifier)?, before);
            assert_eq!(
                reports
                    .publish_with(
                        request.name.clone(),
                        request.subject.clone().unwrap_or_default(),
                        request.expected_revision.clone(),
                        ReportsPublishOptions {
                            request_id: Some(request.request_id.clone()),
                            ..Default::default()
                        }
                    )
                    .await?
                    .seal,
                metadata.seal
            );
            assert_eq!(
                executions
                    .get_with(identifier.clone(), ExecutionsGetOptions { at: None, ..Default::default() })
                    .await?
                    .execution
                    .unwrap()
                    .report
                    .unwrap()
                    .state,
                ReportState::Ready as i32
            );
            drop(store);
            process.close().await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let reports =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .reports
                    .clone();
            let fetched = reports.get(subject, published[0].identifier.clone()).await?;
            assert_eq!(fetched.report.as_ref().unwrap().identifier, published[0].identifier);
            assert_eq!(fetched.file.unwrap().sha256, file.sha256);
            assert_eq!(Store::open(root.path())?.reports(&identifier)?, published);
            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
