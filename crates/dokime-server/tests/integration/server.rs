/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod operations;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use clap::Parser;

// Local Imports
use super::*;
use dokime::domain::model;

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
async fn unsecured_pairing_creates_no_revocable_client() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let server = Server::start(owner.handle(), Config::local()).await.unwrap();
    let native = connect(&server).await;
    let settings = model::remote::RemoteListenerSettings {
        enabled: true,
        address: "127.0.0.1:0".into(),
        tls_enabled: false,
        authentication_enabled: false,
        certificate_names: vec![],
    };
    let status = native
        .unary::<contracts::remote::UpdateListener>(contracts::remote::UpdateListener { settings: Some(settings) })
        .await
        .unwrap()
        .into_inner();
    let endpoint = format!("http://{}", status.listening_address.unwrap());

    let pairing = native
        .unary::<contracts::remote::CreatePairing>(contracts::remote::CreatePairing { label: "Unsecured Lab".into(), endpoint })
        .await
        .unwrap()
        .into_inner();
    assert!(pairing.client.is_none());
    assert!(pairing.pairing.access_key.is_none());
    assert!(pairing.pairing.ca_certificate_pem.is_none());
    let page = native
        .unary::<contracts::remote::ListPairedClients>(contracts::remote::ListPairedClients { cursor: None, limit: 0 })
        .await
        .unwrap()
        .into_inner();
    assert!(page.clients.is_empty());
    assert!(page.next_cursor.is_none());

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn independent_pairings_revoke_and_survive_restart() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let server = Server::start(owner.handle(), Config::local()).await.unwrap();
    let native = connect(&server).await;
    let settings = model::remote::RemoteListenerSettings {
        enabled: true,
        address: "127.0.0.1:0".into(),
        tls_enabled: false,
        authentication_enabled: true,
        certificate_names: vec![],
    };
    let status = native
        .unary::<contracts::remote::UpdateListener>(contracts::remote::UpdateListener { settings: Some(settings) })
        .await
        .unwrap()
        .into_inner();
    let endpoint = format!("http://{}", status.listening_address.unwrap());
    let mut created = Vec::new();
    for label in ["First Client", "Second Client"] {
        let result = native
            .unary::<contracts::remote::CreatePairing>(contracts::remote::CreatePairing {
                label: label.into(),
                endpoint: endpoint.clone(),
            })
            .await
            .unwrap()
            .into_inner();
        assert_eq!(result.pairing.version, 1);
        assert_eq!(result.pairing.endpoint, endpoint);
        assert_eq!(result.pairing.label, label);
        assert!(result.pairing.ca_certificate_pem.is_none());
        assert!(result.client.is_some());
        created.push(result);
    }
    let first_key = created[0].pairing.access_key.as_ref().unwrap();
    let second_key = created[1].pairing.access_key.as_ref().unwrap();
    assert_ne!(first_key, second_key);
    assert_ne!(created[0].client.as_ref().unwrap().identifier, created[1].client.as_ref().unwrap().identifier);
    let security = root.path().join("application/remote/access.json");
    let persisted = std::fs::read_to_string(&security).unwrap();
    assert!(!persisted.contains(first_key));
    assert!(!persisted.contains(second_key));
    let remote = WireClient::connect(endpoint).await.unwrap();
    for key in [first_key, second_key] {
        let mut request = Request::new(contracts::backend::GetIdentity {});
        request.metadata_mut().insert("authorization", format!("Bearer {key}").parse().unwrap());
        remote.unary::<contracts::backend::GetIdentity>(request).await.unwrap();
    }
    let first_id = created[0].client.as_ref().unwrap().identifier.clone();
    for _ in 0..2 {
        native
            .unary::<contracts::remote::RevokePairedClient>(contracts::remote::RevokePairedClient { identifier: first_id.clone() })
            .await
            .unwrap();
    }
    for (key, allowed) in [(first_key, false), (second_key, true)] {
        let mut request = Request::new(contracts::backend::GetIdentity {});
        request.metadata_mut().insert("authorization", format!("Bearer {key}").parse().unwrap());
        let result = remote.unary::<contracts::backend::GetIdentity>(request).await;
        if allowed {
            result.unwrap();
        } else {
            assert_eq!(result.unwrap_err().code(), Code::Unauthenticated);
        }
    }
    let page = native
        .unary::<contracts::remote::ListPairedClients>(contracts::remote::ListPairedClients { cursor: None, limit: 1 })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.clients.len(), 1);
    assert_eq!(page.clients[0].identifier, created[1].client.as_ref().unwrap().identifier);
    assert!(page.next_cursor.is_none());
    let error = native
        .unary::<contracts::remote::ListPairedClients>(contracts::remote::ListPairedClients { cursor: None, limit: 501 })
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::InvalidArgument);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();

    let mut owner = runtime(root.path()).await;
    let server = Server::start(owner.handle(), Config::local()).await.unwrap();
    let native = connect(&server).await;
    let status = native.unary::<contracts::remote::ReadListener>(contracts::remote::ReadListener {}).await.unwrap().into_inner();
    let remote = WireClient::connect(format!("http://{}", status.listening_address.unwrap())).await.unwrap();
    for (key, allowed) in [(first_key, false), (second_key, true)] {
        let mut request = Request::new(contracts::backend::GetIdentity {});
        request.metadata_mut().insert("authorization", format!("Bearer {key}").parse().unwrap());
        let result = remote.unary::<contracts::backend::GetIdentity>(request).await;
        if allowed {
            result.unwrap();
        } else {
            assert_eq!(result.unwrap_err().code(), Code::Unauthenticated);
        }
    }
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn remote_sessions_recorded() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let server = Server::start(owner.handle(), Config::local()).await.unwrap();
    let native = connect(&server).await;
    let settings = model::remote::RemoteListenerSettings {
        enabled: true,
        address: "127.0.0.1:0".into(),
        tls_enabled: false,
        authentication_enabled: true,
        certificate_names: vec![],
    };
    let status = native
        .unary::<contracts::remote::UpdateListener>(contracts::remote::UpdateListener { settings: Some(settings) })
        .await
        .unwrap()
        .into_inner();
    let endpoint = format!("http://{}", status.listening_address.unwrap());
    let created = native
        .unary::<contracts::remote::CreatePairing>(contracts::remote::CreatePairing {
            label: "Session Client".into(),
            endpoint: endpoint.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    let key = created.pairing.access_key.unwrap();
    let principal = created.client.unwrap().identifier;
    let remote = WireClient::connect(endpoint).await.unwrap();
    let mut rejected = Request::new(contracts::backend::GetIdentity {});
    rejected.metadata_mut().insert("authorization", "Bearer a-different-long-token-01234567890123456789".parse().unwrap());
    assert_eq!(remote.unary::<contracts::backend::GetIdentity>(rejected).await.unwrap_err().code(), Code::Unauthenticated);
    for _ in 0..2 {
        let mut admitted = Request::new(contracts::backend::GetIdentity {});
        admitted.metadata_mut().insert("authorization", format!("Bearer {key}").parse().unwrap());
        remote.unary::<contracts::backend::GetIdentity>(admitted).await.unwrap();
    }
    native.unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {}).await.unwrap();

    let page = native
        .unary::<contracts::remote::ListSessions>(contracts::remote::ListSessions { cursor: None, limit: 20 })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.sessions.len(), 2, "native calls are not remote sessions");
    let admitted = &page.sessions[0];
    assert_eq!((admitted.principal.as_str(), admitted.calls), (principal.as_str(), 2));
    assert_eq!(admitted.outcome, model::remote::RemoteSessionOutcome::Admitted);
    let refused = &page.sessions[1];
    assert_eq!((refused.principal.as_str(), refused.calls), ("anonymous", 1));
    assert_eq!(refused.outcome, model::remote::RemoteSessionOutcome::Rejected);
    assert!(refused.rejection.is_some());
    assert!(page.next_cursor.is_none());
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();

    // The Log Survives The Host Restart With Its Sessions Closed.
    let mut owner = runtime(root.path()).await;
    let server = Server::start(owner.handle(), Config::local()).await.unwrap();
    let native = connect(&server).await;
    let restored = native
        .unary::<contracts::remote::ListSessions>(contracts::remote::ListSessions { cursor: None, limit: 1 })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(restored.sessions.len(), 1);
    assert!(restored.sessions[0].disconnected_at.is_some());
    assert_eq!(restored.next_cursor.as_deref(), Some(restored.sessions[0].id.as_str()));
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn listener_policy_isolation() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let cycle = handle
        .create_cycle(
            dokime::transactions::CycleDefinition {
                metadata: support::metadata("remote.listeners", "Listener Isolation"),
                continue_automatically: false,
            },
            Default::default(),
        )
        .await
        .unwrap();
    let service = dokime_server::grpc::Service::new(handle.clone(), Config::local()).await.unwrap();
    let local = service.listen().await.unwrap();
    let local_client = connect(&local).await;
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let certificate = generated.cert.pem().into_bytes();
    let key = generated.signing_key.serialize_pem().into_bytes();
    let identity = TlsIdentity::from_pem(certificate.clone(), key);
    let config = Config::remote("0.0.0.0:0".parse().unwrap(), Some(identity.clone()), Some(TOKEN)).unwrap();
    let remote = service.listen_with(config.clone()).await.unwrap();
    let channel = Endpoint::from_shared(remote.local_endpoint())
        .unwrap()
        .tls_config(ClientTlsConfig::new().domain_name("localhost").ca_certificate(Certificate::from_pem(&certificate)))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let remote_client = WireClient::new(channel);

    assert_eq!(
        remote_client.unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {}).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    remote_client.unary::<contracts::backend::GetIdentity>(authenticated(contracts::backend::GetIdentity {})).await.unwrap();
    let local_methods = local_client
        .unary::<contracts::backend::GetAvailability>(contracts::backend::GetAvailability {})
        .await
        .unwrap()
        .into_inner()
        .methods;
    let remote_methods = remote_client
        .unary::<contracts::backend::GetAvailability>(authenticated(contracts::backend::GetAvailability {}))
        .await
        .unwrap()
        .into_inner()
        .methods;
    assert!(local_methods.iter().any(|path| path == "/dokime.v1.Files/FileWatch"));
    assert!(!remote_methods.iter().any(|path| path == "/dokime.v1.Files/FileWatch"));
    assert_eq!(
        remote_client
            .unary::<contracts::files::FileWatch>(authenticated(contracts::files::FileWatch {
                execution_id: None,
                source: None,
                rules: vec![],
                output: String::new(),
                policy: model::files::wire::FileWatchPolicy::SnapshotAtStart,
                selection: model::files::wire::FileWatchSelection::All,
                wait_for_source_seconds: None,
                poll_interval_ms: None,
                maximum_seconds: None,
                maximum_files: None,
                maximum_bytes: None,
            }))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );

    // Failed Attach And Remote Teardown Cannot Remove Local Policy Or Runtime State.
    assert!(service.listen_with(config.with_bind_addr(local.address())).await.is_err());
    let address = remote.address();
    remote.shutdown().await.unwrap();
    local_client.unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {}).await.unwrap();
    assert_eq!(handle.watch_state().latest().state().cycles[&cycle].phase, model::execution::result::CyclePhase::Active);
    let replacement = "replacement-key-012345678901234567890123456789";
    let config = Config::remote(address, Some(identity), Some(replacement)).unwrap();
    let remote = service.listen_with(config).await.unwrap();
    let channel = Endpoint::from_shared(remote.local_endpoint())
        .unwrap()
        .tls_config(ClientTlsConfig::new().domain_name("localhost").ca_certificate(Certificate::from_pem(&certificate)))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let remote_client = WireClient::new(channel);
    assert_eq!(
        remote_client.unary::<contracts::backend::GetIdentity>(authenticated(contracts::backend::GetIdentity {})).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    let mut request = Request::new(contracts::backend::GetIdentity {});
    request.metadata_mut().insert("authorization", format!("Bearer {replacement}").parse().unwrap());
    remote_client.unary::<contracts::backend::GetIdentity>(request).await.unwrap();
    local_client.unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {}).await.unwrap();

    remote.shutdown().await.unwrap();
    local.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn remote_cycle_seal() {
    use dokime::domain::model::execution::result::CyclePhase;

    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let definition = dokime::transactions::CycleDefinition {
        metadata: support::metadata("remote.seal", "Remote Campaign"),
        continue_automatically: false,
    };
    let cycle = handle.create_cycle(definition.clone(), Default::default()).await.unwrap();
    let other = handle.create_cycle(definition.clone(), Default::default()).await.unwrap();
    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
    let endpoint = server.local_endpoint();
    let target = dokime::client::Target::local(endpoint.clone()).unwrap();
    let client = dokime::client::Client::connect(target, None, None).await.unwrap();
    let reply = client.cycle(cycle.clone()).seal().await.unwrap();
    assert!(reply.observed_cursor.is_some());
    let sealed = handle.watch_state().latest().state().cycles[&cycle].clone();
    assert_eq!(sealed.phase, CyclePhase::Sealed);
    assert!(sealed.saved.is_some());
    assert_eq!(handle.watch_state().latest().state().cycles[&other].phase, CyclePhase::Active);
    client.close_and_wait().await;

    let cycle_id = cycle.to_string();
    let command = ["dokime", "cycle", "seal", &cycle_id, "--endpoint", &endpoint, "--json"];
    dokime_cli::Cli::try_parse_from(command).unwrap().run(dokime::config::Config::local(root.path())).await.unwrap();
    assert_eq!(handle.watch_state().latest().state().cycles[&cycle], sealed);

    // Historical Repeats Remain Safe After The Live Header Retires.
    handle.create_cycle(definition, Default::default()).await.unwrap();
    assert!(!handle.watch_state().latest().state().cycles.contains_key(&cycle));
    dokime_cli::Cli::try_parse_from(command).unwrap().run(dokime::config::Config::local(root.path())).await.unwrap();
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn operation_response_files() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let definition = dokime::transactions::CycleDefinition {
        metadata: support::metadata("remote.operations", "Original"),
        continue_automatically: false,
    };
    let cycle = handle.create_cycle(definition, Default::default()).await.unwrap();
    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
    let endpoint = server.local_endpoint();
    let destination = root.path().join("response.json");
    let output = destination.to_str().unwrap();
    let request = serde_json::json!({"cycleId": cycle.to_string(), "label": "First Call"}).to_string();

    let command =
        ["dokime", "operation", "call", "dokime.execution.rename_cycle", "--endpoint", &endpoint, "--json", &request, "--output", output];
    dokime_cli::Cli::try_parse_from(command).unwrap().run(dokime::config::Config::local(root.path())).await.unwrap();
    let snapshot = handle.watch_state().latest();
    assert_eq!(snapshot.state().cycles[&cycle].metadata.label, "First Call");
    let saved = std::fs::read(&destination).unwrap();
    let response: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    let events = response.as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "completion");

    let next = serde_json::json!({"cycleId": cycle.to_string(), "label": "Second Call"}).to_string();
    let command =
        ["dokime", "operation", "call", "dokime.execution.rename_cycle", "--endpoint", &endpoint, "--json", &next, "--output", output];
    let error = dokime_cli::Cli::try_parse_from(command).unwrap().run(dokime::config::Config::local(root.path())).await.unwrap_err();
    assert!(matches!(error, dokime_cli::Error::Input(_)));
    let snapshot = handle.watch_state().latest();
    assert_eq!(snapshot.state().cycles[&cycle].metadata.label, "First Call", "output conflict must prevent the mutation");
    assert_eq!(std::fs::read(&destination).unwrap(), saved);

    let command = command.into_iter().chain(["--overwrite"]);
    dokime_cli::Cli::try_parse_from(command).unwrap().run(dokime::config::Config::local(root.path())).await.unwrap();
    let snapshot = handle.watch_state().latest();
    assert_eq!(snapshot.state().cycles[&cycle].metadata.label, "Second Call");
    let replacement = std::fs::read(&destination).unwrap();

    let request = serde_json::json!({"artifactId": "not-an-artifact-id"}).to_string();
    let command = [
        "dokime",
        "operation",
        "call",
        "dokime.content.download_artifact",
        "--endpoint",
        &endpoint,
        "--json",
        &request,
        "--output",
        output,
        "--overwrite",
    ];
    assert!(dokime_cli::Cli::try_parse_from(command).unwrap().run(dokime::config::Config::local(root.path())).await.is_err());
    assert_eq!(std::fs::read(&destination).unwrap(), replacement, "failed streamed response must preserve the prior complete file");

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn discovery_shares_independently() {
    use dokime::harmos::operations::OperationDeclaration;
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let node_id = handle.identity().node.node_id.to_string();
    let epoch = handle.process_epoch().to_string();
    let service = dokime_server::grpc::Service::new(handle.clone(), Config::default()).await.unwrap();
    let server = service.listen().await.unwrap();
    let address = server.address();
    let client = connect(&server).await;
    let identity = client.unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {}).await.unwrap().into_inner();
    assert_eq!(identity.backend_id, node_id);
    assert_eq!(identity.epoch, epoch);
    assert_eq!(identity.display_name, "gRPC fixture");
    assert_eq!(identity.target, env!("DOKIME_BACKEND_TARGET"));
    assert!(identity.started_at.is_none());
    assert_eq!(identity.api.unwrap().descriptor_sha256.len(), 32);
    let health = client.unary::<contracts::backend::GetHealth>(contracts::backend::GetHealth {}).await.unwrap().into_inner();
    assert_eq!(health.readiness, model::backend::wire::Readiness::Ready);
    assert_eq!(health.epoch, epoch);
    let native = dokime::client::Client::connect(dokime::client::Target::local(server.local_endpoint()).unwrap(), None, Some(&node_id))
        .await
        .unwrap();
    let managed = native.health().await.unwrap();
    let managed_identity = native.identity().await.unwrap();
    assert_eq!(managed_identity.backend_id, node_id);
    assert_eq!(managed_identity.epoch, epoch);
    assert_eq!(managed_identity.display_name, identity.display_name);
    assert_eq!(managed_identity.product_version, identity.product_version);
    assert_eq!(native.backend().target, identity.target);
    let operations = native.operations().await.unwrap();
    assert!(operations.iter().any(|operation| {
        operation.contract.operation().id.as_str() == dokime::operations::evidence::WatchRecords::ID && operation.grpc_available
    }));
    assert!(operations.iter().any(|operation| {
        operation.contract.operation().id.as_str() == dokime::operations::runtime::GetIdentity::ID && operation.grpc_available
    }));
    let response = native.call_operation(dokime::operations::runtime::GetIdentity::ID, None, b"{}", vec![]).await.unwrap();
    let Some(dokime::client::dynamic::Event::Completion(identity_response)) = response.message().await.unwrap() else {
        panic!("unary identity operation must return a completion");
    };
    assert_eq!(identity_response["backendId"], node_id);
    assert_eq!(identity_response["epoch"], epoch);
    assert!(response.message().await.unwrap().is_none());
    response.close().await;
    assert_eq!(managed.backend_id, node_id);
    assert_eq!(managed.epoch, epoch);
    assert_eq!(managed.readiness, model::runtime::wire::Readiness::Ready);
    assert!(managed.observed_at_nanoseconds < 1_000_000_000);
    native.close_and_wait().await;
    let availability =
        client.unary::<contracts::backend::GetAvailability>(contracts::backend::GetAvailability {}).await.unwrap().into_inner();
    use prost::Message;
    let descriptor = prost_types::FileDescriptorSet::decode(dokime_server::grpc::FILE_DESCRIPTOR_SET).unwrap();
    let mut expected = descriptor
        .file
        .into_iter()
        .flat_map(|file| {
            let package = file.package.unwrap_or_default();
            file.service.into_iter().flat_map(move |service| {
                let name = format!("/{package}.{}", service.name.unwrap());
                service.method.into_iter().map(move |method| format!("{name}/{}", method.name.unwrap()))
            })
        })
        .collect::<std::collections::BTreeSet<_>>();
    // Runtime Startup Also Registers The Shared Storage Provider's Original Catalog.
    let storage = dokime::harmos::operations::Contracts::of::<dokime::harmos::builtins::storage::Storage>().unwrap();
    for contract in storage {
        if let Some(binding) = contract.grpc() {
            expected.insert(binding.path());
        }
    }
    let advertised = availability.methods.iter().cloned().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(advertised.len(), availability.methods.len(), "duplicate advertised method");
    assert_eq!(advertised, expected);
    assert_eq!(availability.limits.unwrap().max_request_bytes, 1024 * 1024);
    server.shutdown().await.unwrap();
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
    assert_eq!(rebooted.handle().identity().node.node_id.to_string(), node_id);
    assert_ne!(rebooted.handle().process_epoch().to_string(), epoch);
    rebooted.shutdown().await.unwrap();
}

#[tokio::test]
async fn malformed_secrets_wire() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let config = keyed();
    let server = Server::start(owner.handle(), config).await.unwrap();
    let client = connect(&server).await;
    let mut invalid = Request::new(contracts::backend::GetIdentity {});
    invalid.metadata_mut().insert("authorization", "Bearer a-different-long-token-01234567890123456789".parse().unwrap());
    assert_eq!(client.unary::<contracts::backend::GetIdentity>(invalid).await.unwrap_err().code(), Code::Unauthenticated);
    for authorization in [format!("Basic {TOKEN}"), "Bearer short".to_owned(), format!("Bearer  {TOKEN}")] {
        let mut malformed = Request::new(contracts::backend::GetIdentity {});
        malformed.metadata_mut().insert("authorization", authorization.parse().unwrap());
        let error = client.unary::<contracts::backend::GetIdentity>(malformed).await.unwrap_err();
        assert_eq!(error.code(), Code::Unauthenticated);
        assert_eq!(error.message(), "invalid secrets");
    }
    let mut duplicate = authenticated(contracts::backend::GetIdentity {});
    duplicate.metadata_mut().append("authorization", format!("Bearer {TOKEN}").parse().unwrap());
    assert_eq!(client.unary::<contracts::backend::GetIdentity>(duplicate).await.unwrap_err().code(), Code::Unauthenticated);
    client.unary::<contracts::backend::GetIdentity>(authenticated(contracts::backend::GetIdentity {})).await.unwrap();
    client.unary::<contracts::backend::GetHealth>(authenticated(contracts::backend::GetHealth {})).await.unwrap();
    client.unary::<contracts::backend::GetAvailability>(authenticated(contracts::backend::GetAvailability {})).await.unwrap();
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn remote_requires_binding() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let remote = "0.0.0.0:0".parse().unwrap();
    assert!(Server::start(owner.handle(), Config::local().with_bind_addr(remote)).await.is_err());
    assert!(Config::remote(remote, None, Some("short")).is_err());
    let invalid = Config::remote(remote, Some(TlsIdentity::from_pem(vec![], vec![])), Some(TOKEN)).unwrap();
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
    let config = Config::remote("0.0.0.0:0".parse().unwrap(), Some(TlsIdentity::from_pem(certificate.clone(), key)), Some(TOKEN)).unwrap();
    assert!(!format!("{config:?}").contains(TOKEN));
    assert!(!format!("{config:?}").contains("PRIVATE KEY"));
    let service = dokime_server::grpc::Service::new(owner.handle(), config).await.unwrap();
    let server = service.listen().await.unwrap();
    let endpoint = Endpoint::from_shared(server.local_endpoint()).unwrap();
    assert!(endpoint.clone().connect().await.is_err(), "untrusted certificate must be rejected");
    let stream = endpoint
        .tls_config(ClientTlsConfig::new().domain_name("localhost").ca_certificate(Certificate::from_pem(&certificate)))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let client = WireClient::new(stream);
    assert_eq!(
        client.unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {}).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    client.unary::<contracts::backend::GetIdentity>(authenticated(contracts::backend::GetIdentity {})).await.unwrap();
    let methods = client
        .unary::<contracts::backend::GetAvailability>(authenticated(contracts::backend::GetAvailability {}))
        .await
        .unwrap()
        .into_inner()
        .methods;
    assert!(
        !methods.iter().any(|path| path == "/dokime.v1.Files/FileWatch"),
        "remote loopback traffic cannot inherit native host-path access"
    );
    server.shutdown().await.unwrap();
    let server = service.listen().await.unwrap();
    let channel = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .tls_config(ClientTlsConfig::new().domain_name("localhost").ca_certificate(Certificate::from_pem(&certificate)))
        .unwrap()
        .connect()
        .await
        .unwrap();
    WireClient::new(channel).unary::<contracts::backend::GetIdentity>(authenticated(contracts::backend::GetIdentity {})).await.unwrap();
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
