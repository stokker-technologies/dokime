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
use std::io::{Cursor, Read as _, Write as _};
// Third Party Imports
use sha2::Digest as _;
// Local Imports
use super::*;
use dokime::domain::model;
use {
    dokime::Identifier, dokime::InstalledPlugin, dokime::Value, dokime::domains::installed_plugins::PluginSecretUpdate,
    dokime::settings::SecretSetting, dokime::settings::SettingValues, dokime::settings::Settings, dokime::settings::SettingsEntry,
    dokime::settings::TextArea,
};

/* === Constants === */

const SECRET: &str = "diagnostics-fake-secret-material";

const DISCOVERED_SECRET: &str = "diagnostics-fake-discovered-material";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(super) async fn fixture(journal: std::sync::Arc<dokime_server::trace::Journal>) -> (support::TestEnvironment, Server, WireClient) {
    let prepared_environment = dokime_server::grpc::Service::prepare(Config::default().with_trace_journal(journal)).unwrap();
    let environment = support::TestEnvironment::with_installed_plugins(vec![secret_module()], &prepared_environment).await;
    let artifact_id = Identifier::new("fixture_diagnostics").unwrap();
    environment
        .runtime
        .save_plugin_secret(&artifact_id, "fixture.token", PluginSecretUpdate { secret: SECRET.to_owned() })
        .await
        .expect("the fixture secret is declared and storable");
    environment
        .runtime
        .record_discovered_settings(
            &artifact_id,
            SettingValues::from([("api_token".to_owned(), Value::from(DISCOVERED_SECRET.to_owned()))]),
        )
        .await
        .expect("a discovered value for a declared field is recordable");
    let server = prepared_environment.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    (environment, server, client)
}

fn secret_module() -> InstalledPlugin {
    InstalledPlugin {
        source: "fixture_diagnostics".to_owned(),
        version: "test".to_owned(),
        description: "Fixture Diagnostics".to_owned(),
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
                // Declared as free text, so nothing reduces it to a reference:
                // its name is the only thing that says it holds a secret.
                SettingsEntry::TextArea(TextArea {
                    id: "api_token".to_owned(),
                    title: "API token".to_owned(),
                    description: None,
                    default: String::new(),
                    required: false,
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
        ..Default::default()
    }
}

fn record(level: &str, message: &str) -> String {
    format!(
        r#"{{"timestamp":"2026-09-30T14:22:33.000001Z","level":"{level}","target":"dokime::fixture","fields":{{"message":"{message}"}}}}"#
    )
}

/* === Tests === */

pub(super) async fn collected_archive_material(client: &WireClient) {
    let mut stream = client
        .download::<contracts::backend::CollectDiagnostics>(contracts::backend::CollectDiagnostics { max_log_tail_bytes: None })
        .await
        .unwrap()
        .into_inner();

    let mut header = None;
    let mut archive = Vec::new();
    let mut completion = None;
    while let Some(frame) = stream.message().await.unwrap() {
        match frame.payload.expect("every frame carries a payload") {
            model::backend::wire::DiagnosticsBundleFramePayload::Header(value) => header = Some(value),
            model::backend::wire::DiagnosticsBundleFramePayload::Chunk(chunk) => {
                assert_eq!(chunk.offset, archive.len() as u64, "chunks are contiguous");
                archive.extend_from_slice(&chunk.data);
            }
            model::backend::wire::DiagnosticsBundleFramePayload::Complete(value) => completion = Some(value),
        }
    }

    let header = header.expect("the stream opens with a header");
    assert!(
        header.filename.starts_with("dokime-diagnostics-") && header.filename.ends_with(".zip"),
        "suggested filename was {}",
        header.filename
    );
    assert_eq!(header.content_type, "application/zip");
    assert!(header.generated_at.is_some());
    assert_eq!(header.total_size_bytes, Some(archive.len() as u64));
    let completion = completion.expect("the stream ends with a completion");
    assert_eq!(completion.size_bytes, archive.len() as u64);
    assert_eq!(completion.sha256, sha2::Sha256::digest(&archive).iter().map(|byte| format!("{byte:02x}")).collect::<String>());

    // The hub log is a declared convention with no writer in this runtime, so
    // its entry is listed and absent rather than missing from the inventory.
    let log = header.entries.iter().find(|entry| entry.path == "logs/dokime.jsonl").expect("the hub log convention is listed");
    assert!(!log.present);
    assert!(log.note.is_some());

    let mut zip = zip::ZipArchive::new(Cursor::new(archive.clone())).unwrap();
    let manifest: serde_json::Value = {
        let mut entry = zip.by_name("manifest.json").expect("manifest.json is present");
        serde_json::from_reader(&mut entry).unwrap()
    };
    assert_eq!(manifest["filename"], header.filename);
    assert!(manifest["generated_at"].as_str().is_some_and(|stamp| !stamp.is_empty()));
    let listed = manifest["entries"].as_array().expect("the manifest lists entries");
    assert_eq!(listed.len(), header.entries.len());
    for entry in listed {
        let path = entry["path"].as_str().unwrap();
        let declared = header.entries.iter().find(|entry| entry.path == path).unwrap();
        assert_eq!(entry["note"].as_str(), declared.note.as_deref());
        assert_eq!(entry["present"].as_bool(), Some(declared.present));
        if entry["present"] == serde_json::Value::Bool(true) {
            let mut member = zip.by_name(path).expect("a present entry is in the archive");
            let mut bytes = Vec::new();
            member.read_to_end(&mut bytes).unwrap();
            assert_eq!(bytes.len() as u64, entry["size_bytes"].as_u64().unwrap());
        }
    }
    for required in
        ["backend/identity.json", "backend/health.json", "settings/settings-frame.json", "settings/artifacts/fixture_diagnostics.json"]
    {
        assert!(zip.by_name(required).is_ok(), "{required} is archived");
    }

    // The redaction proof. Entries are STORED, so the archive's own bytes are
    // the entry bytes: a hit here would be a hit in the file an operator opens.
    for secret in [SECRET, DISCOVERED_SECRET] {
        let needle = secret.as_bytes();
        assert!(!archive.windows(needle.len()).any(|window| window == needle), "the archive bytes name {secret}");
        for index in 0..zip.len() {
            let mut member = zip.by_index(index).unwrap();
            let name = member.name().unwrap().into_owned();
            let mut bytes = Vec::new();
            member.read_to_end(&mut bytes).unwrap();
            assert!(!bytes.windows(needle.len()).any(|window| window == needle), "{name} names {secret}");
        }
    }
    // The binding is still there to read: redaction removed the material, not
    // the evidence that a secret is configured.
    let settings: serde_json::Value = {
        let mut entry = zip.by_name("settings/artifacts/fixture_diagnostics.json").unwrap();
        serde_json::from_reader(&mut entry).unwrap()
    };
    assert_eq!(settings["secret_bindings"]["bindings"]["fixture.token"]["configured"], serde_json::Value::Bool(true));
    // The discovered field is still reported as discovered; only its value went.
    assert_eq!(settings["discovered"]["api_token"], "<redacted>");
}

pub(super) async fn collected_tail_preserves_selected_bytes(environment: &support::TestEnvironment, client: &WireClient) {
    let log = environment.root.path().join("application/logs").join("dokime.jsonl");
    std::fs::create_dir_all(log.parent().unwrap()).unwrap();
    type TailCase = (&'static [u8], u64, Option<&'static [u8]>);
    let cases: &[TailCase] = &[
        (b"older\npart\xe2\x82\xac\nkept\xff", 8, Some(b"kept\xff")),
        (b"prefix\xfftail", 5, Some(b"\xfftail")),
        (b"", 8, Some(b"")),
        (b"ignored", 0, None),
    ];
    for (input, maximum, expected) in cases {
        std::fs::write(&log, input).unwrap();
        let mut stream = client
            .download::<contracts::backend::CollectDiagnostics>(contracts::backend::CollectDiagnostics {
                max_log_tail_bytes: Some(*maximum),
            })
            .await
            .unwrap()
            .into_inner();
        let mut archive = Vec::new();
        let mut selected = None;
        while let Some(frame) = stream.message().await.unwrap() {
            match frame.payload.unwrap() {
                model::backend::wire::DiagnosticsBundleFramePayload::Header(header) => {
                    selected = header.entries.into_iter().find(|entry| entry.path == "logs/dokime.jsonl");
                }
                model::backend::wire::DiagnosticsBundleFramePayload::Chunk(chunk) => {
                    assert_eq!(chunk.offset, archive.len() as u64);
                    archive.extend_from_slice(&chunk.data);
                }
                model::backend::wire::DiagnosticsBundleFramePayload::Complete(completion) => {
                    assert_eq!(completion.size_bytes, archive.len() as u64);
                    let hash = sha2::Sha256::digest(&archive).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                    assert_eq!(completion.sha256, hash);
                }
            }
        }
        let selected = selected.unwrap();
        assert_eq!(selected.present, expected.is_some());
        assert_eq!(selected.size_bytes, expected.map_or(0, |bytes| bytes.len() as u64));
        let mut archive = zip::ZipArchive::new(Cursor::new(archive)).unwrap();
        assert_eq!(archive.by_index(0).unwrap().name().unwrap(), "manifest.json");
        match expected {
            Some(expected) => {
                let mut bytes = Vec::new();
                archive.by_name("logs/dokime.jsonl").unwrap().read_to_end(&mut bytes).unwrap();
                assert_eq!(bytes, *expected);
            }
            None => assert!(archive.by_name("logs/dokime.jsonl").is_err()),
        }
    }
}

pub(super) async fn log_roundtrip(
    environment: &support::TestEnvironment, client: &WireClient,
) -> WireStream<model::backend::wire::LogFrame> {
    let log = environment.root.path().join("application/logs/dokime.jsonl");
    assert!(!log.exists());
    let request = contracts::backend::WatchLogs {
        min_level: model::backend::wire::LogLevel::Warn,
        replay_lines: 0,
    };
    let mut live = client.download::<contracts::backend::WatchLogs>(request).await.unwrap().into_inner();
    std::fs::create_dir_all(log.parent().unwrap()).unwrap();
    let warning = r#"{"timestamp":"2026-09-30T14:22:34.000001Z","level":"WARN","target":"dokime::fixture","fields":{"message":"fixture.warn","api_token":"fake-material"}}"#;
    std::fs::write(&log, format!("{}\n{warning}\nnot json\n{}\n", record("INFO", "fixture.info"), record("DEBUG", "fixture.debug")))
        .unwrap();
    let first = live.message().await.unwrap().unwrap();
    assert_eq!(first.level, model::backend::wire::LogLevel::Warn);
    assert_eq!(first.target, "dokime::fixture");
    assert_eq!(first.message, "fixture.warn");
    assert_eq!(first.fields.iter().find(|entry| entry.key == "api_token").unwrap().value, "<redacted>");
    assert_eq!(first.dropped, 0);
    assert!(first.time.is_some());

    let mut replay = client
        .download::<contracts::backend::WatchLogs>(contracts::backend::WatchLogs {
            min_level: model::backend::wire::LogLevel::Warn,
            replay_lines: 500,
        })
        .await
        .unwrap()
        .into_inner();
    let restored = replay.message().await.unwrap().unwrap();
    assert_eq!(restored.level, first.level);
    assert_eq!(restored.time, first.time);
    assert_eq!(restored.message, first.message);
    assert_eq!(restored.fields, first.fields);
    let mut appended = std::fs::OpenOptions::new().append(true).open(&log).unwrap();
    writeln!(appended, "{}", record("ERROR", "fixture.error")).unwrap();
    drop(appended);
    for stream in [&mut live, &mut replay] {
        let frame = stream.message().await.unwrap().unwrap();
        assert_eq!(frame.level, model::backend::wire::LogLevel::Error);
        assert_eq!(frame.message, "fixture.error");
        assert_eq!(frame.dropped, 0);
    }
    drop(replay);
    live
}
