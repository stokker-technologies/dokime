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
use sha2::{Digest, Sha256};
// Local Imports
use super::*;
use dokime_protocol::v1::{plugins_client::PluginsClient, stage_plugin_frame::Frame};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn bundle() -> Vec<u8> {
    bundle_description("Fixture")
}

fn bundle_description(description: &str) -> Vec<u8> {
    use harmos::plugin::{Bundle, Executable, Facility, Identity, Limits, Source};
    let owner = "grpc-plugin";
    let version = "1.0.0";
    let declaration = serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":version,"description":"A fixture.","config":"()"});
    let policy = harmos::OperationPolicy {
        timeout: std::time::Duration::from_secs(5),
        attempt_timeout: std::time::Duration::from_secs(5),
        retries: harmos::RetryPolicy::Never,
    };
    let lifecycle = [
        harmos::operations::lifecycle::Descriptor::new(
            harmos::operations::lifecycle::Action::Initialize,
            policy,
        )
        .unwrap(),
        harmos::operations::lifecycle::Descriptor::new(
            harmos::operations::lifecycle::Action::Shutdown,
            policy,
        )
        .unwrap(),
    ];
    let contracts = serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"provides":[],"requires":[],"lifecycle":lifecycle});
    let contributions = serde_json::json!({"kind":"contributions","facility":"sidecar","owner":owner,"sections":[]});
    let restart = serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}});
    let publications =
        serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]});
    let observers =
        serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]});
    let executable = format!(
        "inert{prefix}{declaration}\0{prefix}{contracts}\0{prefix}{contributions}\0{prefix}{restart}\0{prefix}{publications}\0{prefix}{observers}\0",
        prefix = std::str::from_utf8(harmos::artifact::PREFIX).unwrap()
    );
    Bundle::build(
        Identity {
            name: "grpc-fixture".into(),
            version: "1.0.0".parse().unwrap(),
            description: description.into(),
            package: Default::default(),
        },
        &[Source {
            path: "bin/fixture",
            bytes: executable.as_bytes(),
            executable: Some(Executable {
                facility: Facility::Sidecar,
                target: dokime_app::plugins::NATIVE_TARGET.into(),
                registrations: vec![],
            }),
        }],
        Limits::default(),
    )
    .unwrap()
    .bytes()
    .to_vec()
}

async fn fixture(root: &std::path::Path) -> Runtime {
    let mut config = RuntimeConfig::new(
        root,
        Metadata::new(Identifier::new("plugins.fixture").unwrap(), "Plugins", None),
    );
    config.installed_plugins.push(dokime_app::InstalledPlugin {
        source: "grpc.host".into(),
        version: "1.0.0".into(),
        enabled: true,
        ..Default::default()
    });
    Runtime::start(config).await.unwrap()
}

fn frames(bytes: &[u8]) -> Vec<v1::StagePluginFrame> {
    let mut frames = vec![v1::StagePluginFrame {
        frame: Some(Frame::Header(v1::StagePluginHeader {
            size_bytes: bytes.len() as u64,
            sha256: Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
        })),
    }];
    for (index, chunk) in bytes.chunks(37).enumerate() {
        frames.push(v1::StagePluginFrame {
            frame: Some(Frame::Chunk(v1::StagePluginChunk {
                offset: (index * 37) as u64,
                data: chunk.to_vec(),
            })),
        });
    }
    frames
}

pub(crate) async fn plugin_stage_identity() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Admin)])
            .unwrap(),
    )
    .await
    .unwrap();
    let mut client = PluginsClient::connect(server.local_endpoint())
        .await
        .unwrap();
    assert_eq!(
        client
            .list_installations(v1::ListInstallationsRequest::default())
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let staged = client
        .stage(authenticated(tokio_stream::iter(frames(&bundle()))))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(staged.state, v1::PluginInstallationState::Staged as i32);
    assert!(!staged.active);
    assert!(!staged.restart_required);
    assert_eq!(staged.metadata.as_ref().unwrap().owners, ["grpc-plugin"]);
    assert_eq!(
        staged
            .metadata
            .as_ref()
            .unwrap()
            .identity
            .as_ref()
            .unwrap()
            .name,
        "grpc-fixture"
    );
    assert_eq!(
        client
            .consent(authenticated(v1::ConsentPluginRequest {
                install_id: staged.install_id.clone(),
                expected_sha256: "0".repeat(64)
            }))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    let installed = client
        .consent(authenticated(v1::ConsentPluginRequest {
            install_id: staged.install_id.clone(),
            expected_sha256: staged.sha256.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        installed.state,
        v1::PluginInstallationState::Installed as i32
    );
    assert!(installed.consented_at.is_some());
    assert!(installed.restart_required);
    assert!(!installed.active);
    let disabled = client
        .set_enabled(authenticated(v1::SetPluginEnabledRequest {
            install_id: staged.install_id.clone(),
            enabled: false,
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(disabled.state, v1::PluginInstallationState::Disabled as i32);
    let page = client
        .list_installations(authenticated(v1::ListInstallationsRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.total_count, 1);
    assert_eq!(page.installations[0].install_id, staged.install_id);
    let removed = client
        .remove(authenticated(v1::RemovePluginRequest {
            install_id: staged.install_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        removed.installation.unwrap().state,
        v1::PluginInstallationState::RemovalPending as i32
    );
    let current = client
        .get_installation(authenticated(v1::GetInstallationRequest {
            install_id: staged.install_id,
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(current.restart_required);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn invalid_uploads_mutate() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Admin)])
            .unwrap(),
    )
    .await
    .unwrap();
    let mut client = PluginsClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let mut bad = frames(&bundle());
    if let Some(Frame::Chunk(chunk)) = &mut bad[1].frame {
        chunk.offset = 1;
    }
    assert_eq!(
        client
            .stage(authenticated(tokio_stream::iter(bad)))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let mut short = frames(&bundle());
    short.pop();
    assert_eq!(
        client
            .stage(authenticated(tokio_stream::iter(short)))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        client
            .list_installations(authenticated(v1::ListInstallationsRequest::default()))
            .await
            .unwrap()
            .into_inner()
            .total_count,
        0
    );
    let (sender, upload) = open_upload(&mut client).await;
    upload.abort();
    let _ = upload.await;
    drop(sender);
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match client
                .list_installations(authenticated(v1::ListInstallationsRequest::default()))
                .await
            {
                Ok(page) => {
                    assert_eq!(page.into_inner().total_count, 0);
                    break;
                }
                Err(error) if error.code() == Code::ResourceExhausted => {
                    tokio::task::yield_now().await
                }
                other => panic!("unexpected cancellation result: {other:?}"),
            }
        }
    })
    .await
    .unwrap();
    server.shutdown().await.unwrap();
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Read)])
            .unwrap(),
    )
    .await
    .unwrap();
    let mut client = PluginsClient::connect(server.local_endpoint())
        .await
        .unwrap();
    assert_eq!(
        client
            .list_installations(authenticated(v1::ListInstallationsRequest::default()))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    let inventory = client
        .list_admitted_plugins(authenticated(v1::ListAdmittedPluginsRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(inventory.plugins.len(), 1);
    assert_eq!(inventory.plugins[0].plugin_id, "grpc.host");
    assert_eq!(
        inventory.plugins[0].owner,
        Some(v1::admitted_plugin::Owner::Host(true))
    );
    assert_eq!(
        client
            .stage(authenticated(tokio_stream::iter(frames(&bundle()))))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn listener_shutdown_admission() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Admin)])
            .unwrap(),
    )
    .await
    .unwrap();
    let mut client = PluginsClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let (sender, upload) = open_upload(&mut client).await;
    tokio::time::timeout(Duration::from_secs(2), server.shutdown())
        .await
        .unwrap()
        .unwrap();
    let error = tokio::time::timeout(Duration::from_secs(2), upload)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert_eq!(error.code(), Code::Unavailable);
    drop(sender);
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Admin)])
            .unwrap(),
    )
    .await
    .unwrap();
    let mut client = PluginsClient::connect(server.local_endpoint())
        .await
        .unwrap();
    assert_eq!(
        client
            .list_installations(authenticated(v1::ListInstallationsRequest::default()))
            .await
            .unwrap()
            .into_inner()
            .total_count,
        0
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

// A competing query can win the sole permit before the upload arrives. Retry
// only that explicit pre-admission Busy outcome, never an uncertain mutation.
async fn open_upload(
    client: &mut PluginsClient<Channel>,
) -> (
    tokio::sync::mpsc::Sender<v1::StagePluginFrame>,
    tokio::task::JoinHandle<Result<tonic::Response<v1::PluginInstallation>, tonic::Status>>,
) {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let (sender, receiver) = tokio::sync::mpsc::channel(1);
            sender.send(frames(&bundle()).remove(0)).await.unwrap();
            let mut uploader = client.clone();
            let upload = tokio::spawn(async move {
                uploader
                    .stage(authenticated(tokio_stream::wrappers::ReceiverStream::new(
                        receiver,
                    )))
                    .await
            });
            // Let the streaming RPC submit its request before the admission
            // probe; otherwise this task wins the sole permit every time.
            tokio::task::yield_now().await;
            loop {
                match client
                    .list_installations(authenticated(v1::ListInstallationsRequest::default()))
                    .await
                {
                    Err(error) if error.code() == Code::ResourceExhausted => {
                        return (sender, upload);
                    }
                    Ok(_) => {
                        if upload.is_finished() {
                            assert_eq!(
                                upload.await.unwrap().unwrap_err().code(),
                                Code::ResourceExhausted
                            );
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                    other => panic!("unexpected admission result: {other:?}"),
                }
            }
        }
    })
    .await
    .unwrap()
}

pub(crate) async fn oversized_permissions_consent() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_max_message_bytes(4096)
            .with_bearer_auth(vec![grant(Role::Admin)])
            .unwrap(),
    )
    .await
    .unwrap();
    let mut client = PluginsClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let bytes = bundle_description(&"d".repeat(8192));
    let staged = client
        .stage(authenticated(tokio_stream::iter(frames(&bytes))))
        .await
        .unwrap()
        .into_inner();
    assert!(staged.metadata.is_none());
    assert!(staged.error.as_ref().unwrap().contains("budget"));
    let page = client
        .list_installations(authenticated(v1::ListInstallationsRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.installations[0].install_id, staged.install_id);
    let entry = client
        .get_installation(authenticated(v1::GetInstallationRequest {
            install_id: staged.install_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(entry.sha256, staged.sha256);
    assert!(entry.metadata.is_none());
    assert_eq!(
        client
            .consent(authenticated(v1::ConsentPluginRequest {
                install_id: staged.install_id.clone(),
                expected_sha256: staged.sha256
            }))
            .await
            .unwrap_err()
            .code(),
        Code::ResourceExhausted
    );
    let entry = client
        .get_installation(authenticated(v1::GetInstallationRequest {
            install_id: staged.install_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(entry.state, v1::PluginInstallationState::Staged as i32);
    assert!(entry.consented_at.is_none());
    assert!(
        client
            .remove(authenticated(v1::RemovePluginRequest {
                install_id: staged.install_id
            }))
            .await
            .unwrap()
            .into_inner()
            .installation
            .is_none()
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
