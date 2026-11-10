/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub(crate) mod operations;

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

pub(crate) async fn unsecured_pairing_has_no_key() {
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
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
    assert!(status.listening_address.is_some());

    let pairing = native
        .unary::<contracts::remote::CreatePairing>(contracts::remote::CreatePairing { label: "Unsecured Lab".into() })
        .await
        .unwrap()
        .into_inner();
    assert!(pairing.pairing.access_key.is_none());
    assert!(pairing.pairing.certificate_pem.is_none());

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn pairing_rotation_survives_restart() {
    // Start The Backend And Export The First Pairing.
    let root = tempfile::tempdir().unwrap();
    let prepared = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared).await;
    let server = prepared.ready().await.unwrap().listen().await.unwrap();
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
    let first = native
        .unary::<contracts::remote::CreatePairing>(contracts::remote::CreatePairing { label: "Lab".into() })
        .await
        .unwrap()
        .into_inner()
        .pairing;
    let remote = WireClient::connect(format!("http://{}", status.listening_address.unwrap())).await.unwrap();
    let mut request = Request::new(contracts::backend::GetIdentity {});
    request.metadata_mut().insert("authorization", format!("Bearer {}", first.access_key.as_ref().unwrap()).parse().unwrap());
    remote.unary::<contracts::backend::GetIdentity>(request).await.unwrap();

    // Export Again; Only The Replacement Key Is Accepted.
    let second = native
        .unary::<contracts::remote::CreatePairing>(contracts::remote::CreatePairing { label: "Lab".into() })
        .await
        .unwrap()
        .into_inner()
        .pairing;
    for (pairing, allowed) in [(&first, false), (&second, true)] {
        let mut request = Request::new(contracts::backend::GetIdentity {});
        request.metadata_mut().insert("authorization", format!("Bearer {}", pairing.access_key.as_ref().unwrap()).parse().unwrap());
        let result = remote.unary::<contracts::backend::GetIdentity>(request).await;
        if allowed {
            result.unwrap();
        } else {
            assert_eq!(result.unwrap_err().code(), Code::Unauthenticated);
        }
    }
    let persisted = std::fs::read_to_string(root.path().join("application/remote/access.json")).unwrap();
    assert!(!persisted.contains(first.access_key.as_ref().unwrap()));
    assert!(!persisted.contains(second.access_key.as_ref().unwrap()));
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();

    // Reopen And Authenticate With The Same Replacement File.
    let prepared = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared).await;
    let server = prepared.ready().await.unwrap().listen().await.unwrap();
    let native = connect(&server).await;
    let status = native.unary::<contracts::remote::ReadListener>(contracts::remote::ReadListener {}).await.unwrap().into_inner();
    let remote = WireClient::connect(format!("http://{}", status.listening_address.unwrap())).await.unwrap();
    let mut request = Request::new(contracts::backend::GetIdentity {});
    request.metadata_mut().insert("authorization", format!("Bearer {}", second.access_key.unwrap()).parse().unwrap());
    remote.unary::<contracts::backend::GetIdentity>(request).await.unwrap();
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn listener_policy_isolation() {
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
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
    let service = prepared_owner.ready().await.unwrap();
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
    assert!(local_methods.iter().any(|path| path == "/dokime.v1.Media/Screenshot"));
    assert!(remote_methods.iter().any(|path| path == "/dokime.v1.Media/Screenshot"));
    assert_eq!(local_methods, remote_methods);

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

pub(crate) async fn remote_cycle_seal() {
    use dokime::domain::model::execution::result::CyclePhase;

    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let handle = owner.handle();
    let definition = dokime::transactions::CycleDefinition {
        metadata: support::metadata("remote.seal", "Remote Campaign"),
        continue_automatically: false,
    };
    let cycle = handle.create_cycle(definition.clone(), Default::default()).await.unwrap();
    let other = handle.create_cycle(definition.clone(), Default::default()).await.unwrap();
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
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

pub(crate) async fn operation_response_files() {
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let handle = owner.handle();
    let definition = dokime::transactions::CycleDefinition {
        metadata: support::metadata("remote.operations", "Original"),
        continue_automatically: false,
    };
    let cycle = handle.create_cycle(definition, Default::default()).await.unwrap();
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
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

pub(crate) async fn discovery_shares_independently() {
    use dokime::harmos::operations::OperationDeclaration;
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::default()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let handle = owner.handle();
    let node_id = handle.identity().node.node_id.to_string();
    let epoch = handle.process_epoch().to_string();
    let service = prepared_owner.ready().await.unwrap();
    let server = service.listen().await.unwrap();
    let address = server.address();
    let client = connect(&server).await;
    let identity = client.unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {}).await.unwrap().into_inner();
    assert_eq!(identity.backend_id, node_id);
    assert_eq!(identity.epoch, epoch);
    assert_eq!(identity.display_name, "gRPC fixture");
    assert_eq!(identity.target, env!("DOKIME_BACKEND_TARGET"));
    assert!(identity.started_at.is_none());
    let build = dokime::version::current();
    assert_eq!(identity.source_revision, build.revision.map(|revision| revision.full()));
    assert_eq!(identity.channel.as_deref(), build.channel);
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
    assert_eq!(native.backend().source_revision, identity.source_revision);
    assert_eq!(native.backend().channel, identity.channel);
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
    // Availability Lists Registered Bindings, Including Shared Storage And Lifecycle Operations.
    let operations = dokime::harmos::operations::Contracts::of::<dokime::operations::Operations>().unwrap();
    let storage = dokime::harmos::operations::Contracts::of::<dokime::harmos::builtins::storage::Storage>().unwrap();
    let lifecycle = dokime::harmos::operations::Contracts::of::<dokime::harmos::builtins::lifecycle::Lifecycle>().unwrap();
    let expected = operations
        .into_iter()
        .chain(storage)
        .chain(lifecycle)
        .filter_map(|contract| contract.grpc().map(|binding| binding.path()))
        .collect::<std::collections::BTreeSet<_>>();
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
    let mut rebooted = runtime(root.path(), &dokime_server::grpc::Service::prepare(Config::local()).unwrap()).await;
    assert_eq!(rebooted.handle().identity().node.node_id.to_string(), node_id);
    assert_ne!(rebooted.handle().process_epoch().to_string(), epoch);
    rebooted.shutdown().await.unwrap();
}

pub(crate) async fn malformed_secrets_wire() {
    let root = tempfile::tempdir().unwrap();
    let config = keyed();
    let prepared_owner = dokime_server::grpc::Service::prepare(config.clone()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;

    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
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

pub(crate) async fn remote_requires_binding() {
    let root = tempfile::tempdir().unwrap();
    let prepared = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared).await;
    let service = prepared.ready().await.unwrap();
    let remote = "0.0.0.0:0".parse().unwrap();
    assert!(service.listen_with(Config::local().with_bind_addr(remote)).await.is_err());
    assert!(Config::remote(remote, None, Some("short")).is_err());
    let invalid = Config::remote(remote, Some(TlsIdentity::from_pem(vec![], vec![])), Some(TOKEN)).unwrap();
    assert!(service.listen_with(invalid).await.is_err());
    assert!(!owner.handle().is_shutting_down());
    owner.shutdown().await.unwrap();
}

pub(crate) async fn tls_listener_clients() {
    // A throwaway localhost identity generated for this run; nothing is committed.
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let certificate = generated.cert.pem().into_bytes();
    let key = generated.signing_key.serialize_pem().into_bytes();
    let root = tempfile::tempdir().unwrap();
    let config = Config::remote("0.0.0.0:0".parse().unwrap(), Some(TlsIdentity::from_pem(certificate.clone(), key)), Some(TOKEN)).unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(config.clone()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;

    assert!(!format!("{config:?}").contains(TOKEN));
    assert!(!format!("{config:?}").contains("PRIVATE KEY"));
    let service = prepared_owner.ready().await.unwrap();
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
        methods.iter().any(|path| path == "/dokime.v1.Media/Screenshot"),
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

pub(crate) async fn names_refresh_preserves_remote_stream() {
    use std::io::Write as _;
    let root = tempfile::tempdir().unwrap();
    let prepared = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared).await;
    let handle = owner.handle();
    let server = prepared.ready().await.unwrap().listen().await.unwrap();
    let native = connect(&server).await;
    let reservation = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let listener_address = reservation.local_addr().unwrap().to_string();
    drop(reservation);
    let status = native
        .unary::<contracts::remote::UpdateListener>(contracts::remote::UpdateListener {
            settings: Some(model::remote::RemoteListenerSettings {
                enabled: true,
                address: listener_address,
                tls_enabled: true,
                authentication_enabled: false,
                certificate_names: vec!["localhost".into()],
            }),
        })
        .await
        .unwrap()
        .into_inner();
    let address = status.listening_address.unwrap();
    let certificate = status.certificate_pem.unwrap();
    let pairing = native
        .unary::<contracts::remote::CreatePairing>(contracts::remote::CreatePairing { label: "Stream Fixture".into() })
        .await
        .unwrap()
        .into_inner()
        .pairing;
    let endpoint = Endpoint::from_shared(format!("https://{address}")).unwrap();
    let ca = pairing.certificate_pem.unwrap();
    let channel = endpoint
        .clone()
        .tls_config(ClientTlsConfig::new().domain_name("localhost").ca_certificate(Certificate::from_pem(&ca)))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let remote = WireClient::new(channel);
    let log = root.path().join("application/logs/dokime.jsonl");
    std::fs::create_dir_all(log.parent().unwrap()).unwrap();
    let before = br#"{"timestamp":"2026-09-30T14:22:33Z","level":"ERROR","target":"fixture","fields":{"message":"before reload"}}
"#;
    let after = br#"{"timestamp":"2026-09-30T14:22:34Z","level":"ERROR","target":"fixture","fields":{"message":"after reload"}}
"#;
    std::fs::write(&log, before).unwrap();
    // Raw WireStream Has No Retry Or Resubscription To Hide A Transport Restart.
    let mut stream = remote
        .download::<contracts::backend::WatchLogs>(contracts::backend::WatchLogs {
            min_level: model::backend::wire::LogLevel::Error,
            replay_lines: 10,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(stream.message().await.unwrap().unwrap().message, "before reload");
    let snapshot = handle.read_application_settings().await.unwrap();
    let mut settings = snapshot.settings;
    settings.listener.certificate_names.push("next.example".into());
    handle
        .update_application_settings(model::settings::wire::UpdateSettings {
            expected_revision: snapshot.revision,
            settings,
        })
        .await
        .unwrap();
    // Export Replaces A Certificate Whose Names Changed.
    native.unary::<contracts::remote::CreatePairing>(contracts::remote::CreatePairing { label: "Refreshed Fixture".into() }).await.unwrap();
    let status = native.unary::<contracts::remote::ReadListener>(contracts::remote::ReadListener {}).await.unwrap().into_inner();
    assert_eq!(status.listening_address.as_deref(), Some(address.as_str()));
    assert_ne!(status.certificate_pem.as_deref(), Some(certificate.as_str()));
    std::fs::OpenOptions::new().append(true).open(&log).unwrap().write_all(after).unwrap();
    loop {
        let frame = stream.message().await.unwrap().unwrap();
        if frame.level != model::backend::wire::LogLevel::Unspecified {
            assert_eq!(frame.message, "after reload");
            break;
        }
    }
    // The Old File No Longer Trusts New Handshakes; The New Export Does.
    assert!(
        endpoint
            .clone()
            .tls_config(ClientTlsConfig::new().domain_name("localhost").ca_certificate(Certificate::from_pem(&ca)))
            .unwrap()
            .connect()
            .await
            .is_err()
    );
    let ca = status.certificate_pem.unwrap();
    let fresh = endpoint
        .tls_config(ClientTlsConfig::new().domain_name("next.example").ca_certificate(Certificate::from_pem(ca)))
        .unwrap()
        .connect()
        .await
        .unwrap();
    WireClient::new(fresh).unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity {}).await.unwrap();

    // Refresh Leaves Pending Listener Edits For Their Explicit Commit.
    let occupied = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let snapshot = handle.read_application_settings().await.unwrap();
    let active = snapshot.settings.listener.clone();
    let mut settings = snapshot.settings;
    settings.listener.address = occupied.local_addr().unwrap().to_string();
    let pending = settings.listener.clone();
    handle
        .update_application_settings(model::settings::wire::UpdateSettings {
            expected_revision: snapshot.revision,
            settings,
        })
        .await
        .unwrap();
    native
        .unary::<contracts::remote::CreatePairing>(contracts::remote::CreatePairing { label: "Pending Edit Fixture".into() })
        .await
        .unwrap();
    let status = native.unary::<contracts::remote::ReadListener>(contracts::remote::ReadListener {}).await.unwrap().into_inner();
    assert_eq!(status.listening_address.as_deref(), Some(address.as_str()));
    assert_eq!(status.settings.as_ref(), Some(&pending));
    assert!(status.error.is_none());
    assert!(
        native.unary::<contracts::remote::UpdateListener>(contracts::remote::UpdateListener { settings: Some(pending) }).await.is_err()
    );
    let status = native.unary::<contracts::remote::ReadListener>(contracts::remote::ReadListener {}).await.unwrap().into_inner();
    assert_eq!(status.listening_address.as_deref(), Some(address.as_str()));
    assert!(status.error.is_some());
    native.unary::<contracts::remote::UpdateListener>(contracts::remote::UpdateListener { settings: Some(active) }).await.unwrap();
    drop(stream);
    drop(handle);
    owner.shutdown().await.unwrap();
    server.shutdown().await.unwrap();
}

struct TransientIngress {
    path: std::sync::Mutex<String>,
    calls: std::sync::atomic::AtomicUsize,
    failures: std::sync::atomic::AtomicUsize,
}

impl dokime::harmos::grpc::middleware::Middleware for TransientIngress {
    fn enter<'a>(&'a self, call: &'a mut dokime::harmos::grpc::middleware::Invocation) -> dokime::harmos::grpc::middleware::Enter<'a> {
        Box::pin(async move {
            use std::sync::atomic::Ordering;
            if call.path() != *self.path.lock().unwrap() {
                return Ok(());
            }
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.failures.try_update(Ordering::SeqCst, Ordering::SeqCst, |count| count.checked_sub(1)).is_ok() {
                return Err(dokime::harmos::Error::NotReady { domain: "fixture listener" });
            }
            Ok(())
        })
    }
}

pub(crate) async fn client_retries_reads_without_replaying_mutations() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let root = tempfile::tempdir().unwrap();
    let prepared = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared).await;
    let handle = owner.handle();
    let ingress = Arc::new(TransientIngress {
        path: std::sync::Mutex::new(String::new()),
        calls: AtomicUsize::new(0),
        failures: AtomicUsize::new(0),
    });
    handle.grpc().register_middleware(ingress.clone()).unwrap();
    let server = prepared.ready().await.unwrap().listen().await.unwrap();
    let client =
        dokime::client::Client::connect(dokime::client::Target::local(server.local_endpoint()).unwrap(), None, None).await.unwrap();
    *ingress.path.lock().unwrap() = "/dokime.v1.Backend/GetIdentity".into();
    ingress.failures.store(1, Ordering::SeqCst);
    client.call(contracts::backend::GetIdentity {}).await.unwrap();
    assert_eq!(ingress.calls.swap(0, Ordering::SeqCst), 2);
    ingress.failures.store(1, Ordering::SeqCst);
    client.call_raw("/dokime.v1.Backend/GetIdentity", Vec::new(), Duration::from_secs(10)).await.unwrap();
    assert_eq!(ingress.calls.swap(0, Ordering::SeqCst), 2);
    ingress.failures.store(1, Ordering::SeqCst);
    let bounded = client.with_timeout(Duration::from_millis(10)).call(contracts::backend::GetIdentity {}).await;
    assert!(matches!(bounded, Err(dokime::client::Error::Rpc(status)) if status.code() == Code::DeadlineExceeded));
    assert_eq!(ingress.calls.swap(0, Ordering::SeqCst), 1);
    ingress.failures.store(2, Ordering::SeqCst);
    assert!(
        matches!(client.call(contracts::backend::GetIdentity {}).await, Err(dokime::client::Error::Rpc(status)) if status.code() == Code::Unavailable)
    );
    assert_eq!(ingress.calls.swap(0, Ordering::SeqCst), 2);
    *ingress.path.lock().unwrap() = "/dokime.v1.Execution/StartExecution".into();
    ingress.failures.store(1, Ordering::SeqCst);
    let result = client
        .call(contracts::execution::StartExecution {
            execution_id: uuid::Uuid::now_v7().to_string(),
            mode: model::execution::wire::ExecutionMode::Automatic,
        })
        .await;
    assert!(matches!(result, Err(dokime::client::Error::Rpc(status)) if status.code() == Code::Unavailable));
    assert_eq!(ingress.calls.swap(0, Ordering::SeqCst), 1);
    *ingress.path.lock().unwrap() = String::new();
    let suite = support::fixture_suite(&["passed"]);
    support::register_fixture_progress_execution(&handle, suite.clone()).await;
    let cases = support::materialize_cases("fixture.suite", &suite);
    let cycle = support::cycle_with_cases(&handle, support::metadata("retry.cycle", "Retry Fixture"), cases).await;
    let execution = client.cycle(cycle).executions().await.unwrap().remove(0);
    *ingress.path.lock().unwrap() = "/dokime.v1.Telemetry/WatchSamples".into();
    ingress.failures.store(1, Ordering::SeqCst);
    let samples = execution.samples(Vec::new(), false, 10).unwrap();
    samples.message().await.unwrap().unwrap();
    samples.close().await;
    assert_eq!(ingress.calls.swap(0, Ordering::SeqCst), 2);
    *ingress.path.lock().unwrap() = "/dokime.v1.Evidence/WatchRecords".into();
    ingress.failures.store(1, Ordering::SeqCst);
    let records = execution.records(Vec::new(), true, None).unwrap();
    records.message().await.unwrap().unwrap();
    records.close().await;
    assert_eq!(ingress.calls.load(Ordering::SeqCst), 2);
    client.close_and_wait().await;
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}
