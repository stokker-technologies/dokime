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
use std::{collections::BTreeMap, sync::Arc};
// Third Party Imports
use tonic::{Code, transport::Channel};
// Local Imports
use super::*;
use dokime_app::{
    InstalledPlugin,
    core::{Identifier, Metadata},
    domains::{
        ActionDispatcher, ActionDispatcherError, ActionDispatcherFuture,
        OptionProviderContribution, OptionProviderOption, OptionProviderSource, PluginEntrypoint,
    },
    settings::{SecretSetting, Settings, SettingsEntry, TextArea},
};
use dokime_protocol::v1::{self, settings_client::SettingsClient};

/* === Constants === */
const CONTROL_TOKEN: &str = "settings-control-token-012345678901234567890";
const ADMIN_TOKEN: &str = "settings-admin-token-01234567890123456789012";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct FixtureActionDispatcher;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ActionDispatcher for FixtureActionDispatcher {
    fn status<'a>(&'a self) -> ActionDispatcherFuture<'a, dokime_app::domains::StatusResponse> {
        Box::pin(async { Err(ActionDispatcherError::Unavailable) })
    }

    fn query_options<'a>(
        &'a self,
        entrypoint: &'a str,
    ) -> ActionDispatcherFuture<'a, Vec<OptionProviderOption>> {
        Box::pin(async move {
            if entrypoint != "projects" {
                return Err(ActionDispatcherError::UnknownEntrypoint(
                    entrypoint.to_owned(),
                ));
            }
            Ok(vec![OptionProviderOption {
                value: "fixture-project".to_owned(),
                label: "Fixture Project".to_owned(),
                description: None,
                disabled: false,
                metadata: None,
            }])
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn config() -> Config {
    Config::local()
        .with_bearer_auth(vec![
            grant(Role::Read),
            BearerGrant::new("control".into(), Role::Control, CONTROL_TOKEN).unwrap(),
            BearerGrant::new("admin".into(), Role::Admin, ADMIN_TOKEN).unwrap(),
        ])
        .unwrap()
}

async fn fixture() -> (common::TestEnvironment, Server, SettingsClient<Channel>) {
    let environment = common::TestEnvironment::with_installed_plugins_and_action_dispatcher(
        vec![fixture_module()],
        Identifier::new("fixture.settings").unwrap(),
        Arc::new(FixtureActionDispatcher),
    )
    .await;
    let server = Server::start(environment.runtime.clone(), config())
        .await
        .unwrap();
    let client = SettingsClient::connect(server.local_endpoint())
        .await
        .unwrap();
    (environment, server, client)
}

pub(crate) async fn frame_readable_probe() {
    let (_environment, server, mut client) = fixture().await;
    assert_eq!(
        client
            .get_settings(v1::GetSettingsRequest {})
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );

    let frame = client
        .get_settings(auth(v1::GetSettingsRequest {}, super::TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert!(frame.sections.iter().any(|section| {
        section.section_id == "identity"
            && matches!(
                section.owner.as_ref(),
                Some(v1::settings_section::Owner::App(true))
            )
    }));
    let card = frame
        .cards
        .iter()
        .find(|card| card.card_id == "fixture.settings")
        .expect("the artifact's own card carries the readiness probe");
    assert_eq!(card.plugin_id, "fixture.settings");
    assert_eq!(card.state, v1::SettingsCardState::Disconnected as i32);

    server.shutdown().await.unwrap();
}

pub(crate) async fn storage_sources_admin() {
    let (_environment, server, mut client) = fixture().await;
    assert_eq!(
        client
            .list_storage_sources(v1::ListStorageSourcesRequest {})
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let sources = client
        .list_storage_sources(auth(v1::ListStorageSourcesRequest {}, super::TOKEN))
        .await
        .unwrap()
        .into_inner()
        .sources;
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].identifier, "local");
    assert!(sources[0].enabled);
    for token in [super::TOKEN, CONTROL_TOKEN] {
        assert_eq!(
            client
                .reorder_storage_sources(auth(
                    v1::ReorderStorageSourcesRequest {
                        identifiers: vec!["local".into()],
                    },
                    token
                ))
                .await
                .unwrap_err()
                .code(),
            Code::PermissionDenied
        );
        assert_eq!(
            client
                .set_storage_source_enabled(auth(
                    v1::SetStorageSourceEnabledRequest {
                        identifier: "local".into(),
                        enabled: true,
                    },
                    token
                ))
                .await
                .unwrap_err()
                .code(),
            Code::PermissionDenied
        );
    }
    for identifiers in [
        vec![],
        vec!["local".into(), "local".into()],
        vec!["unknown".into()],
        vec!["x".repeat(257)],
    ] {
        assert_eq!(
            client
                .reorder_storage_sources(auth(
                    v1::ReorderStorageSourcesRequest { identifiers },
                    ADMIN_TOKEN
                ))
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument
        );
    }
    assert_eq!(
        client
            .set_storage_source_enabled(auth(
                v1::SetStorageSourceEnabledRequest {
                    identifier: "local".into(),
                    enabled: false,
                },
                ADMIN_TOKEN
            ))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    // "local" is the only registered source, so no reordering of it is
    // observable. What the contract does decide is which requests it refuses.
    for refused in [
        Vec::new(),
        vec![String::new()],
        vec!["x".repeat(257)],
        vec!["local".into(); 1025],
    ] {
        assert_eq!(
            client
                .reorder_storage_sources(auth(
                    v1::ReorderStorageSourcesRequest {
                        identifiers: refused.clone()
                    },
                    ADMIN_TOKEN
                ))
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument,
            "{} identifiers must be refused",
            refused.len()
        );
    }
    let reply = client
        .reorder_storage_sources(auth(
            v1::ReorderStorageSourcesRequest {
                identifiers: vec!["local".into()],
            },
            ADMIN_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(reply.sources, sources);
    assert_eq!(
        client
            .set_storage_source_enabled(auth(
                v1::SetStorageSourceEnabledRequest {
                    identifier: "local".into(),
                    enabled: true,
                },
                ADMIN_TOKEN
            ))
            .await
            .unwrap()
            .into_inner()
            .sources,
        sources
    );
    server.shutdown().await.unwrap();
}

pub(crate) async fn plugin_settings_admin() {
    let (_environment, server, mut client) = fixture().await;
    let request = v1::GetPluginSettingsRequest {
        plugin_id: "fixture.settings".into(),
    };
    let initial = client
        .get_plugin_settings(auth(request.clone(), super::TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        initial.values.as_ref().unwrap().fields["workspace"].kind,
        Some(v1::value::Kind::TextValue("Lab".into()))
    );
    assert_eq!(
        initial
            .credential_bindings
            .as_ref()
            .unwrap()
            .missing_required,
        vec!["fixture.token"]
    );

    let update = v1::UpdatePluginSettingsRequest {
        plugin_id: "fixture.settings".into(),
        values: Some(v1::ValueObject {
            fields: BTreeMap::from([(
                "workspace".into(),
                v1::Value {
                    kind: Some(v1::value::Kind::TextValue("Production".into())),
                },
            )]),
        }),
    };
    assert_eq!(
        client
            .update_plugin_settings(auth(update.clone(), CONTROL_TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    let updated = client
        .update_plugin_settings(auth(update, ADMIN_TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        updated.values.unwrap().fields["workspace"].kind,
        Some(v1::value::Kind::TextValue("Production".into()))
    );

    let updated = client
        .update_plugin_credential(auth(
            v1::UpdatePluginCredentialRequest {
                plugin_id: "fixture.settings".into(),
                credential_id: "fixture.token".into(),
                secret: "secret-material".into(),
            },
            ADMIN_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner();
    let response_debug = format!("{updated:?}");
    let bindings = updated.credential_bindings.unwrap();
    assert!(bindings.missing_required.is_empty());
    assert!(bindings.bindings["fixture.token"].configured);
    assert!(!response_debug.contains("secret-material"));

    server.shutdown().await.unwrap();
}

fn fixture_module() -> InstalledPlugin {
    InstalledPlugin {
        source: "fixture.settings".to_owned(),
        version: "test".to_owned(),
        description: "Fixture Settings".to_owned(),
        enabled: true,
        settings: Settings {
            contributions: vec![
                SettingsEntry::TextArea(TextArea {
                    id: "workspace".to_owned(),
                    title: "Workspace".to_owned(),
                    description: None,
                    default: "Lab".to_owned(),
                    required: true,
                }),
                SettingsEntry::SecretReference(SecretSetting {
                    id: "fixture.token".to_owned(),
                    title: "Fixture token".to_owned(),
                    description: None,
                    default: None,
                    required: true,
                }),
            ],
        },
        // The card is the artifact, and it exists because a section named a
        // readiness route. Nothing declares a status field in the form.
        settings_status: Some(PluginEntrypoint {
            name: "fixture.status".to_owned(),
        }),
        option_providers: vec![
            OptionProviderContribution {
                metadata: metadata("fixture.workspaces", "Workspaces"),
                input_schema: None,
                permissions: Vec::new(),
                source: OptionProviderSource::Static {
                    options: vec![OptionProviderOption {
                        value: "lab".to_owned(),
                        label: "Lab".to_owned(),
                        description: None,
                        disabled: false,
                        metadata: None,
                    }],
                },
            },
            OptionProviderContribution {
                metadata: metadata("fixture.projects", "Projects"),
                input_schema: None,
                permissions: Vec::new(),
                source: OptionProviderSource::Runtime {
                    entrypoint: PluginEntrypoint {
                        name: "projects".to_owned(),
                    },
                },
            },
        ],
        ..Default::default()
    }
}

fn metadata(identifier: &str, label: &str) -> Metadata {
    common::metadata(identifier, label)
}

/* === Tests === */

pub(crate) async fn backend_policy_revision() {
    let (_environment, server, mut client) = fixture().await;
    let initial = client
        .read_storage_policy(auth(v1::ReadStoragePolicyRequest {}, super::TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(initial.revision, 0);
    let mut policy = initial.configured.unwrap();
    policy.save_interval_ms = 1234;
    let request = v1::UpdateStoragePolicyRequest {
        expected_revision: 0,
        policy: Some(policy),
    };
    for token in [super::TOKEN, CONTROL_TOKEN] {
        assert_eq!(
            client
                .update_storage_policy(auth(request, token))
                .await
                .unwrap_err()
                .code(),
            Code::PermissionDenied
        );
    }
    let saved = client
        .update_storage_policy(auth(request, ADMIN_TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(saved.revision, 1);
    assert_eq!(saved.configured, Some(policy));
    assert_eq!(saved.effective, initial.effective);
    assert!(saved.restart_required);
    assert_eq!(
        client
            .update_storage_policy(auth(request, ADMIN_TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    let invalid = v1::UpdateStoragePolicyRequest {
        expected_revision: 1,
        policy: Some(v1::StoragePolicy::default()),
    };
    assert_eq!(
        client
            .update_storage_policy(auth(invalid, ADMIN_TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let read = client
        .read_storage_policy(auth(v1::ReadStoragePolicyRequest {}, super::TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(read, saved);
    server.shutdown().await.unwrap();
}

pub(crate) async fn app_settings_role() {
    let (environment, server, mut client) = fixture().await;
    let initial = client
        .read_application_settings(auth(v1::ReadApplicationSettingsRequest {}, super::TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(initial.configured_data_root, None);

    // Appearance Needs Control; It Is Saved Without A Restart
    let appearance = v1::UpdateAppearanceRequest {
        appearance: Some(v1::AppearancePreference {
            theme_mode: v1::ThemeMode::Dark as i32,
            theme_family: Some("brand.theme".into()),
            density: v1::Density::Compact as i32,
        }),
    };
    assert_eq!(
        client
            .update_appearance(auth(appearance.clone(), super::TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    let saved = client
        .update_appearance(auth(appearance, CONTROL_TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(saved.appearance.unwrap().theme_mode(), v1::ThemeMode::Dark);
    assert_eq!(saved.restart_required, initial.restart_required);

    // The Data Root Needs Admin And Applies On Restart
    let chosen = environment.root.path().join("chosen-data");
    let data_root = v1::UpdateDataRootRequest {
        data_root: Some(chosen.display().to_string()),
    };
    assert_eq!(
        client
            .update_data_root(auth(data_root.clone(), CONTROL_TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    let relative = v1::UpdateDataRootRequest {
        data_root: Some("relative".into()),
    };
    assert_eq!(
        client
            .update_data_root(auth(relative, ADMIN_TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let saved = client
        .update_data_root(auth(data_root, ADMIN_TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        saved.configured_data_root,
        Some(chosen.display().to_string())
    );
    assert!(saved.restart_required);
    assert!(chosen.is_dir(), "the chosen root is proven writable");
    assert_eq!(saved.effective_data_root, initial.effective_data_root);
    server.shutdown().await.unwrap();
}
