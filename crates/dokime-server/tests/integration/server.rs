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

// Third Party Imports

// Local Imports
use super::*;

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
async fn discovery_shares_independently() {
    use harmos::operations::OperationDeclaration;
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let node_id = handle.identity().node.node_id.to_string();
    let epoch = handle.process_epoch().to_string();
    let service = dokime_server::grpc::Service::new(handle.clone(), Config::default())
        .await
        .unwrap();
    let server = service.listen().await.unwrap();
    let address = server.address();
    let client = connect(&server).await;
    let identity = client
        .unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {})
        .await
        .unwrap()
        .into_inner();
    assert_eq!(identity.backend_id, node_id);
    assert_eq!(identity.epoch, epoch);
    assert_eq!(identity.display_name, "gRPC fixture");
    assert!(identity.started_at.is_none());
    assert_eq!(identity.api.unwrap().descriptor_sha256.len(), 32);
    let health = client
        .unary::<contracts::backend::GetHealth>(contracts::backend::GetHealth {})
        .await
        .unwrap()
        .into_inner();
    assert_eq!(health.readiness, contracts::backend::Readiness::Ready);
    assert_eq!(health.epoch, epoch);
    let native = dokime_client::Client::connect(
        dokime_client::Target::local(server.local_endpoint()).unwrap(),
        None,
        Some(&node_id),
    )
    .await
    .unwrap();
    let managed = native.health().await.unwrap();
    let managed_identity = native.identity().await.unwrap();
    assert_eq!(managed_identity.backend_id, node_id);
    assert_eq!(managed_identity.epoch, epoch);
    assert_eq!(managed_identity.display_name, identity.display_name);
    assert_eq!(managed_identity.product_version, identity.product_version);
    let operations = native.operations().await.unwrap();
    assert!(operations.iter().any(|operation| {
        operation.contract.operation().id.as_str() == dokime_runtime::operations::evidence::WatchRecords::ID
            && operation.grpc_available
    }));
    assert!(operations.iter().any(|operation| {
        operation.contract.operation().id.as_str() == dokime_runtime::operations::runtime::GetIdentity::ID
            && operation.grpc_available
    }));
    let response = native
        .call_operation(
            dokime_runtime::operations::runtime::GetIdentity::ID,
            None,
            b"{}",
            vec![],
        )
        .await
        .unwrap();
    let Some(dokime_client::dynamic::Event::Completion(identity_response)) =
        response.message().await.unwrap()
    else {
        panic!("unary identity operation must return a completion");
    };
    assert_eq!(identity_response["backendId"], node_id);
    assert_eq!(identity_response["epoch"], epoch);
    assert!(response.message().await.unwrap().is_none());
    response.close().await;
    assert_eq!(managed.backend_id, node_id);
    assert_eq!(managed.epoch, epoch);
    assert_eq!(
        managed.readiness,
        dokime_runtime::operations::runtime::Readiness::Ready
    );
    assert!(managed.observed_at_nanoseconds < 1_000_000_000);
    native.close_and_wait().await;
    let availability = client
        .unary::<contracts::backend::GetAvailability>(contracts::backend::GetAvailability {})
        .await
        .unwrap()
        .into_inner();
    use prost::Message;
    let descriptor =
        prost_types::FileDescriptorSet::decode(dokime_server::grpc::FILE_DESCRIPTOR_SET).unwrap();
    let expected = descriptor
        .file
        .into_iter()
        .flat_map(|file| {
            let package = file.package.unwrap_or_default();
            file.service.into_iter().flat_map(move |service| {
                let name = format!("/{package}.{}", service.name.unwrap());
                service
                    .method
                    .into_iter()
                    .map(move |method| format!("{name}/{}", method.name.unwrap()))
            })
        })
        .collect::<std::collections::BTreeSet<_>>();
    let advertised = availability
        .methods
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        advertised.len(),
        availability.methods.len(),
        "duplicate advertised method"
    );
    assert_eq!(advertised, expected);
    assert_eq!(availability.limits.unwrap().max_request_bytes, 1024 * 1024);
    server.shutdown().await
        .unwrap();
    // Keep the generated client/stream alive throughout listener drain.
    drop(client);
    assert!(tokio::net::TcpStream::connect(address).await.is_err());
    assert!(!handle.is_shutting_down());
    let second = service.listen().await.unwrap();
    assert_eq!(
        connect(&second)
            .await
            .unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {})
            .await
            .unwrap()
            .into_inner()
            .epoch,
        epoch
    );
    second.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
    let mut rebooted = runtime(root.path()).await;
    assert_eq!(
        rebooted.handle().identity().node.node_id.to_string(),
        node_id
    );
    assert_ne!(rebooted.handle().process_epoch().to_string(), epoch);
    rebooted.shutdown().await.unwrap();
}

#[tokio::test]
async fn malformed_credentials_wire() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let config = Config::local()
        .with_bearer_auth(vec![grant(Role::Read)])
        .unwrap();
    let server = Server::start(owner.handle(), config).await.unwrap();
    let client = connect(&server).await;
    let mut invalid = Request::new(contracts::backend::GetIdentity {});
    invalid.metadata_mut().insert(
        "authorization",
        "Bearer a-different-long-token-01234567890123456789"
            .parse()
            .unwrap(),
    );
    assert_eq!(
        client
            .unary::<contracts::backend::GetIdentity>(invalid)
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    for authorization in [
        format!("Basic {TOKEN}"),
        "Bearer short".to_owned(),
        format!("Bearer  {TOKEN}"),
    ] {
        let mut malformed = Request::new(contracts::backend::GetIdentity {});
        malformed
            .metadata_mut()
            .insert("authorization", authorization.parse().unwrap());
        let error = client
            .unary::<contracts::backend::GetIdentity>(malformed)
            .await
            .unwrap_err();
        assert_eq!(error.code(), Code::Unauthenticated);
        assert_eq!(error.message(), "invalid credentials");
    }
    let mut duplicate = authenticated(contracts::backend::GetIdentity {});
    duplicate
        .metadata_mut()
        .append("authorization", format!("Bearer {TOKEN}").parse().unwrap());
    assert_eq!(
        client
            .unary::<contracts::backend::GetIdentity>(duplicate)
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    client
        .unary::<contracts::backend::GetIdentity>(authenticated(
            contracts::backend::GetIdentity {},
        ))
        .await
        .unwrap();
    client
        .unary::<contracts::backend::GetHealth>(authenticated(
            contracts::backend::GetHealth {},
        ))
        .await
        .unwrap();
    client
        .unary::<contracts::backend::GetAvailability>(authenticated(
            contracts::backend::GetAvailability {},
        ))
        .await
        .unwrap();
    client
        .unary::<contracts::backend::GetDiagnostics>(authenticated(
            contracts::backend::GetDiagnostics {},
        ))
        .await
        .unwrap();
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn diagnostics_projects_paths() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let epoch = owner.handle().session_context().await.0.to_string();
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let client = connect(&server).await;
    let diagnostics = client
        .unary::<contracts::backend::GetDiagnostics>(contracts::backend::GetDiagnostics {})
        .await
        .unwrap()
        .into_inner();
    let backend = diagnostics.server.expect("server diagnostics");
    assert!(!backend.version.is_empty());
    assert_eq!(backend.process.unwrap().pid, std::process::id());
    let runtime = diagnostics.runtime.expect("runtime diagnostics");
    assert_eq!(runtime.session_id, epoch);
    assert_eq!(runtime.storage_root, root.path().to_string_lossy());
    assert_eq!(
        runtime.startup_doctor.unwrap().status,
        contracts::backend::DiagnosticCheckStatus::Ok
    );
    assert!(
        runtime
            .path_conventions
            .iter()
            .any(|path| path.key == contracts::backend::DiagnosticPathKey::HubLog)
    );
    let counts = runtime.counts.expect("diagnostic counts");
    assert_eq!(counts.cycles, 0);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn remote_requires_binding() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let remote = "0.0.0.0:0".parse().unwrap();
    assert!(
        Server::start(owner.handle(), Config::local().with_bind_addr(remote))
            .await
            .is_err()
    );
    assert!(Config::remote(remote, TlsIdentity::from_pem(vec![], vec![]), vec![]).is_err());
    let invalid = Config::remote(
        remote,
        TlsIdentity::from_pem(vec![], vec![]),
        vec![grant(Role::Admin)],
    )
    .unwrap();
    assert!(Server::start(owner.handle(), invalid).await.is_err());
    assert!(!owner.handle().is_shutting_down());
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn tls_listener_clients() {
    // A throwaway localhost identity generated for this run; nothing is committed.
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let certificate = generated.cert.pem().into_bytes();
    let key = generated.signing_key.serialize_pem().into_bytes();
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let config = Config::remote(
        "0.0.0.0:0".parse().unwrap(),
        TlsIdentity::from_pem(certificate.clone(), key),
        vec![grant(Role::Read)],
    )
    .unwrap();
    assert!(!format!("{config:?}").contains(TOKEN));
    assert!(!format!("{config:?}").contains("PRIVATE KEY"));
    let server = Server::start(owner.handle(), config).await.unwrap();
    let endpoint = Endpoint::from_shared(server.local_endpoint()).unwrap();
    assert!(
        endpoint.clone().connect().await.is_err(),
        "untrusted certificate must be rejected"
    );
    let stream = endpoint
        .tls_config(
            ClientTlsConfig::new()
                .domain_name("localhost")
                .ca_certificate(Certificate::from_pem(&certificate)),
        )
        .unwrap()
        .connect()
        .await
        .unwrap();
    let client = WireClient::new(stream);
    assert_eq!(
        client
            .unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {})
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    client
        .unary::<contracts::backend::GetIdentity>(authenticated(
            contracts::backend::GetIdentity {},
        ))
        .await
        .unwrap();
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
