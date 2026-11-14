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

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime_runtime_next::Host;
use dokime_storage::Store;
use sha2::Digest;

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
async fn client_publishes_execution_report() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Prepare Sealed Evidence and Its Selected Document
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let host = Host::open(root.path(), super::support::settings())?;
        let entry = host.add_managed(&super::support::execution_fixture()?).await?;
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
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut reports = dokime_protocol::client::reports_client::ReportsClient::new(channel.clone());
        let mut executions = dokime_protocol::client::executions_client::ExecutionsClient::new(channel);
        let subject = dokime_protocol::client::ReportSubject {
            kind: Some(dokime_protocol::client::report_subject::Kind::Execution(identifier.clone())),
        };
        let preview = reports
            .create(dokime_protocol::client::ReportsCreateRequest {
                subject: Some(subject.clone()),
                document_id: document.metadata.as_ref().map_or(String::new(), |metadata| metadata.identifier.clone()),
                request_id: "preview".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        let file = preview.file.unwrap();
        assert_eq!(preview.report.unwrap().origin, dokime_protocol::client::ReportOrigin::Current as i32);
        let store = Store::open(root.path())?;
        assert!(store.reports(&identifier)?.is_empty());
        assert_eq!(store.read(&identifier)?, before);
        assert!(
            reports
                .create(dokime_protocol::client::ReportsCreateRequest {
                    subject: Some(subject.clone()),
                    document_id: "missing".into(),
                    request_id: "invalid-document".into(),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        assert!(store.reports(&identifier)?.is_empty());
        let current =
            executions.get(dokime_protocol::client::ExecutionsGetRequest { name: identifier.clone(), at: None }).await?.into_inner();
        assert_eq!(current.execution.unwrap().report.unwrap().state, dokime_protocol::client::ReportState::Pending as i32);
        let request = dokime_protocol::client::ReportsPublishRequest {
            name: file.metadata.as_ref().unwrap().id.clone(),
            subject: Some(subject.clone()),
            expected_revision: file.sha256.clone(),
            request_id: "publish".into(),
        };
        let mut stale = request.clone();
        stale.expected_revision[0] ^= 1;
        assert!(reports.publish(stale).await.is_err());
        assert!(store.reports(&identifier)?.is_empty());
        let metadata = reports.publish(request.clone()).await?.into_inner();
        let published = store.reports(&identifier)?;
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].document.as_ref(), Some(&document));
        assert_eq!(published[0].identifier, metadata.report.as_ref().unwrap().identifier);
        assert_eq!(published[0].execution_seal, before.seal);
        assert_eq!(sha2::Sha256::digest(&published[0].html).as_slice(), file.sha256);
        assert!(std::str::from_utf8(&published[0].html)?.contains("<svg"));
        assert_eq!(store.read(&identifier)?, before);
        assert_eq!(reports.publish(request).await?.into_inner().seal, metadata.seal);
        assert_eq!(
            executions
                .get(dokime_protocol::client::ExecutionsGetRequest { name: identifier.clone(), at: None })
                .await?
                .into_inner()
                .execution
                .unwrap()
                .report
                .unwrap()
                .state,
            dokime_protocol::client::ReportState::Ready as i32
        );
        drop(store);
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut reports = dokime_protocol::client::reports_client::ReportsClient::connect(process.url.clone()).await?;
        let fetched = reports
            .get(dokime_protocol::client::ReportsGetRequest {
                subject: Some(subject),
                name: published[0].identifier.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(fetched.report.as_ref().unwrap().identifier, published[0].identifier);
        assert_eq!(fetched.file.unwrap().sha256, file.sha256);
        assert_eq!(Store::open(root.path())?.reports(&identifier)?, published);
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
