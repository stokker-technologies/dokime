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
use crate::support::wire::WireMessage;
use prost::Message;
use sha2::{Digest, Sha256};
// Local Imports
use super::*;
use dokime::domain::model;
use dokime::domains::installed_plugins::{ResolvedBrand, load_brand};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn brand(path: &std::path::Path) -> ResolvedBrand {
    let mut brand = load_brand(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../branding")).unwrap();
    brand.customization.logo_assets[0].path = path.to_string_lossy().into_owned();
    brand
}

fn failure(status: tonic::Status, code: Code, kind: model::appearance::wire::AppearanceFailureKind) {
    assert_eq!(status.code(), code);
    let envelope = tonic_types::Status::decode(status.details()).unwrap();
    assert_eq!(envelope.code, code as i32);
    assert_eq!(envelope.details[0].type_url, "type.googleapis.com/dokime.v1.AppearanceFailure");
    assert_eq!(
        <model::appearance::wire::AppearanceFailure as WireMessage>::from_wire(envelope.details[0].value.as_slice()).unwrap().kind,
        kind
    );
}

#[tokio::test]
async fn appearance_asset_bound() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("logo.svg");
    let bytes = vec![b'a'; 150_000];
    std::fs::write(&path, &bytes).unwrap();
    let mut owner = runtime(root.path()).await;
    owner.handle().set_brand(Some(brand(&path)));
    let expected = owner.handle().appearance_frame().await;
    let server = Server::start(owner.handle(), keyed()).await.unwrap();
    let appearance = WireClient::connect(server.local_endpoint()).await.unwrap();
    let content = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(
        appearance.unary::<contracts::appearance::GetAppearance>(contracts::appearance::GetAppearance {}).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    assert_eq!(
        content
            .download::<contracts::content::DownloadAppearanceAsset>(contracts::content::DownloadAppearanceAsset { reference: None })
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let actual = appearance
        .unary::<contracts::appearance::GetAppearance>(authenticated(contracts::appearance::GetAppearance {}))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(actual.app_name, expected.app_name);
    assert_eq!(actual.organization.unwrap().name, expected.organization.unwrap().name);
    assert_eq!(actual.title_template, expected.title_template);
    assert_eq!(actual.active_theme, expected.active_theme.map(|id| id.to_string()));
    assert_eq!(actual.theme_family_locked, expected.theme_family_locked);
    assert_eq!(actual.mode_locked, expected.mode_locked);
    assert_eq!(actual.mode_policy, model::appearance::wire::ThemeModePolicy::UserOrSystem);
    assert_eq!(actual.source_plugin_id.as_deref(), Some("brand"));
    assert_eq!(actual.themes.len(), expected.themes.len());
    let theme = &actual.themes[0];
    assert_eq!(theme.stylesheet, expected.themes[0].stylesheet);
    let family_variables: std::collections::BTreeMap<_, _> =
        theme.variables.iter().map(|entry| (entry.key.clone(), entry.value.clone())).collect();
    assert_eq!(family_variables, expected.themes[0].variables);
    let light_variables: std::collections::BTreeMap<_, _> =
        theme.light.as_ref().unwrap().variables.iter().map(|entry| (entry.key.clone(), entry.value.clone())).collect();
    assert_eq!(light_variables, expected.themes[0].light.variables);
    assert_eq!(theme.dark.as_ref().unwrap().plot_palette, expected.themes[0].dark.plot_palette);
    assert_eq!(theme.corner_radii.as_ref().unwrap().four_xl, expected.themes[0].corner_radii.four_xl);
    assert_eq!(actual.logos.len(), 4);
    assert_eq!(
        actual.logos.iter().map(|logo| logo.variant).collect::<Vec<_>>(),
        expected.logos.iter().map(|logo| logo.variant).collect::<Vec<_>>()
    );
    let logo = actual.logos.into_iter().next().unwrap();
    assert_eq!(logo.alt.as_deref(), Some("Dokime"));
    let metadata = logo.asset.unwrap();
    assert_eq!(metadata.size_bytes, bytes.len() as u64);
    let reference = metadata.reference.clone().unwrap();
    assert_eq!(reference.sha256, Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    failure(
        content
            .download::<contracts::content::DownloadAppearanceAsset>(authenticated(contracts::content::DownloadAppearanceAsset {
                reference: Some(model::appearance::wire::AppearanceAssetReference {
                    plugin_id: "unselected.owner".into(),
                    ..reference.clone()
                }),
            }))
            .await
            .unwrap()
            .into_inner()
            .message()
            .await
            .unwrap_err(),
        Code::NotFound,
        model::appearance::wire::AppearanceFailureKind::AssetMissing,
    );
    let request = || authenticated(contracts::content::DownloadAppearanceAsset { reference: Some(reference.clone()) });
    let mut stream = content.download::<contracts::content::DownloadAppearanceAsset>(request()).await.unwrap().into_inner();
    assert_eq!(
        stream.message().await.unwrap().unwrap().payload,
        Some(model::content::wire::AppearanceAssetFramePayload::Metadata(metadata))
    );
    // Once admitted, changing the source cannot mix bytes into this transfer.
    std::fs::write(&path, b"changed").unwrap();
    let mut received = Vec::new();
    let mut complete = false;
    while let Some(frame) = stream.message().await.unwrap() {
        match frame.payload.unwrap() {
            model::content::wire::AppearanceAssetFramePayload::Chunk(chunk) => {
                assert!(!complete);
                assert_eq!(chunk.offset, received.len() as u64);
                assert!(chunk.data.len() <= 64 * 1024);
                received.extend(chunk.data);
            }
            model::content::wire::AppearanceAssetFramePayload::Complete(value) => {
                assert!(!complete);
                assert_eq!(value.size_bytes, bytes.len() as u64);
                assert_eq!(value.sha256, reference.sha256);
                complete = true;
            }
            _ => panic!("unexpected transfer frame"),
        }
    }
    assert!(complete);
    assert_eq!(received, bytes);
    failure(
        content.download::<contracts::content::DownloadAppearanceAsset>(request()).await.unwrap().into_inner().message().await.unwrap_err(),
        Code::FailedPrecondition,
        model::appearance::wire::AppearanceFailureKind::AssetChanged,
    );
    std::fs::remove_file(&path).unwrap();
    failure(
        content.download::<contracts::content::DownloadAppearanceAsset>(request()).await.unwrap().into_inner().message().await.unwrap_err(),
        Code::NotFound,
        model::appearance::wire::AppearanceFailureKind::AssetMissing,
    );
    failure(
        content
            .download::<contracts::content::DownloadAppearanceAsset>(authenticated(contracts::content::DownloadAppearanceAsset {
                reference: Some(model::appearance::wire::AppearanceAssetReference { asset_id: "../logo".into(), ..reference }),
            }))
            .await
            .unwrap()
            .into_inner()
            .message()
            .await
            .unwrap_err(),
        Code::InvalidArgument,
        model::appearance::wire::AppearanceFailureKind::InvalidReference,
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn appearance_refuses_transfer() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("logo.svg");
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(8 * 1024 * 1024 + 1).unwrap();
    let mut owner = runtime(root.path()).await;
    let mut selected = brand(&path);
    owner.handle().set_brand(Some(selected.clone()));
    let server = Server::start(owner.handle(), Config::local()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    failure(
        client.unary::<contracts::appearance::GetAppearance>(contracts::appearance::GetAppearance {}).await.unwrap_err(),
        Code::ResourceExhausted,
        model::appearance::wire::AppearanceFailureKind::TooLarge,
    );
    // Source preflight also bounds CSS independently of a missing logo file.
    std::fs::remove_file(path).unwrap();
    selected.theme.as_mut().unwrap().stylesheet = Some("x".repeat(1024 * 1024));
    owner.handle().set_brand(Some(selected));
    failure(
        client.unary::<contracts::appearance::GetAppearance>(contracts::appearance::GetAppearance {}).await.unwrap_err(),
        Code::ResourceExhausted,
        model::appearance::wire::AppearanceFailureKind::TooLarge,
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
