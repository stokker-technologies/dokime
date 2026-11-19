/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Desktop's Capture Folders Decide Where Its Built-In Host Captures From

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{error::Error, num::NonZeroUsize, path::Path, time::Duration};

// Third Party Imports
use prost::Message;
use tokio_stream::StreamExt;

// Local Imports
use dokime_client_internal::ArtifactsListResponse;
use dokime_client_internal::CasesCreateResponse;
use dokime_client_internal::CyclesCreateResponse;
use dokime_client_internal::ExecutionsGetResponse;
use dokime_client_internal::ExecutionsListResponse;
use dokime_client_internal::PluginPlacement;
use dokime_client_internal::PluginRemote;
use dokime_client_internal::PluginValue;
use dokime_client_internal::PluginsCreateOptions;
use dokime_client_internal::ProgressWatchResponse;
use dokime_client_internal::RegisteredSelection;
use dokime_client_internal::SelectionPlan;
use dokime_client_internal::plugin_placement;
use dokime_client_internal::{PluginState, PluginsDeleteOptions, PluginsGetOptions, PluginsWatchOptions};
use dokime_gui::capture_folders;
use dokime_gui::connection::ConnectionSession;
use dokime_gui::connection::host::Session;
use dokime_gui::native_session::DesktopSession;
use dokime_gui::native_session::backend::Backend;
use dokime_protocol::host::CasesCreateRequest;
use dokime_protocol::host::CyclesCreateRequest;
use dokime_protocol::host::ExecutionsGetRequest;
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::RegisteredSelectionParametersEntry;
use dokime_protocol::host::SelectionValues;
use dokime_protocol::host::content::ArtifactsListRequest;
use dokime_protocol::host::execution_outcome::Attribution;
use dokime_protocol::host::executions::ProgressWatchRequest;
use dokime_protocol::model::ArtifactScope;
use dokime_runtime::{PlacementSettings, PluginProcess};
use dokime_sdk::Termination;
use dokime_sdk::value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

async fn admit_plugin(session: &Session, endpoint: &str) -> Result<(), Box<dyn Error>> {
    let options = PluginsCreateOptions {
        context: session.call_options()?,
        ..Default::default()
    };
    let placement = PluginPlacement {
        kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: endpoint.into() })),
    };
    let admitted =
        session.execute(session.client().plugins.create_with("capture", placement, PluginValue { enabled: true }, options)).await?;
    ready(session, admitted.entry.ok_or("Admit Capture Fixture: Missing Entry")?).await
}

async fn ready(session: &Session, mut entry: harmos_protocol::host::PluginEntry) -> Result<(), Box<dyn Error>> {
    let options = PluginsWatchOptions {
        name: Some(entry.name.clone()),
        context: session.call_options()?,
        ..Default::default()
    };
    let mut watching = session.execute(session.client().plugins.watch_with(options)).await?;
    while entry.state != PluginState::Ready as i32 {
        if entry.failure.is_some() {
            return Err(format!("Admit Capture Fixture: {:?}", entry.failure).into());
        }
        session.execute(watching.next()).await?.ok_or("Admit Capture Fixture: Watch Closed")?;
        let options = PluginsGetOptions {
            context: session.call_options()?,
            ..Default::default()
        };
        entry = session
            .execute(session.client().plugins.get_with(entry.name, options))
            .await?
            .entry
            .ok_or("Admit Capture Fixture: Entry Removed")?;
    }
    Ok(())
}

// Copies the Device Folder Once and Returns the Termination and Captured Artifact Names.
async fn copy(session: &Session, device: &Path, request: &str) -> Result<(i32, Vec<String>), Box<dyn Error>> {
    let text = |value: &Path| SelectionValues {
        values: vec![dokime_sdk::Value {
            kind: Some(value::Kind::Text(value.display().to_string())),
        }],
    };
    let parameters = vec![
        RegisteredSelectionParametersEntry {
            key: "root".into(),
            value: Some(text(device)),
        },
        RegisteredSelectionParametersEntry {
            key: "outside".into(),
            value: Some(text(device)),
        },
    ];
    let plan: CasesCreateResponse = session
        .unary(
            "/dokime.host.v1.Cases/Create",
            CasesCreateRequest {
                plan: Some(SelectionPlan {
                    selections: vec![RegisteredSelection {
                        suite_id: "capture".into(),
                        template_id: "capture.copy".into(),
                        parameters,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                validate_only: true,
                request_id: format!("{request}-preview"),
            },
        )
        .await?;
    let origins = plan.cases.into_iter().map(|case| case.origin.ok_or("Plan Copy: Missing Origin")).collect::<Result<Vec<_>, _>>()?;
    let cycle: CyclesCreateResponse = session
        .unary(
            "/dokime.host.v1.Cycles/Create",
            CyclesCreateRequest {
                origins,
                continue_automatically: true,
                request_id: format!("{request}-cycle"),
                ..Default::default()
            },
        )
        .await?;
    let executions: ExecutionsListResponse = session
        .unary(
            "/dokime.host.v1.Executions/List",
            ExecutionsListRequest {
                parent: cycle.cycle.ok_or("Start Copy: Missing Cycle")?.cycle_id,
                ..Default::default()
            },
        )
        .await?;
    let execution =
        executions.executions.first().and_then(|item| item.summary.as_ref()).ok_or("Start Copy: Missing Execution")?.execution_id.clone();
    let request = ProgressWatchRequest { name: execution.clone(), at: None }.encode_to_vec();
    let mut progress = session.watch_raw("/dokime.host.v1.executions.Progress/Watch", &request).await?;
    loop {
        let frame = progress.next().await.ok_or("Observe Copy: Closed Before Completion")??;
        if ProgressWatchResponse::decode(frame.as_slice())?.finished {
            break;
        }
    }
    drop(progress);
    let detail: ExecutionsGetResponse =
        session.unary("/dokime.host.v1.Executions/Get", ExecutionsGetRequest { name: execution.clone(), at: None }).await?;
    let termination = match detail.execution.and_then(|execution| execution.outcome).and_then(|outcome| outcome.attribution) {
        Some(Attribution::NotEvaluated(outcome)) => outcome.termination,
        Some(Attribution::Evaluated(_)) => Termination::Failed as i32,
        other => return Err(format!("Observe Copy: Unexpected Outcome {other:?}").into()),
    };
    let artifacts: ArtifactsListResponse = session
        .unary(
            "/dokime.host.v1.content.Artifacts/List",
            ArtifactsListRequest {
                scope: Some(ArtifactScope {
                    execution_id: Some(execution),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;
    Ok((termination, artifacts.artifacts.into_iter().map(|artifact| artifact.filename).collect()))
}

/* === Tests === */

#[test]
fn capture_folders_persist_and_apply_to_the_built_in_host() -> Result<(), Box<dyn Error>> {
    dokime_gui::test_support::run(async {
        // Start the Owned Host With No Capture Folders Saved
        let application = dokime_gui::test_support::tempdir()?;
        let storage = dokime_gui::test_support::tempdir()?;
        let device = dokime_gui::test_support::tempdir()?;
        std::fs::write(device.path().join("device.log"), b"boot\n")?;
        let folders = capture_folders::path(application.path());
        let backend = Backend::new(storage.path().to_path_buf()).with_capture_folders(folders.clone());
        let placement = PlacementSettings {
            reconnect_delay: Duration::from_millis(25),
            stream_window: NonZeroUsize::new(8).unwrap(),
            interval_window: dokime_runtime::INTERVAL_WINDOW,
            interval_lag: dokime_runtime::INTERVAL_LAG,
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        };
        let mut plugin = PluginProcess::start(&dokime_gui::test_support::fixture("dokime-rust-capture"), true, &placement).await?;
        let owner = DesktopSession::with_local_backend(ConnectionSession::default(), backend.clone());
        let session = owner.local_client().await.map_err(|error| format!("{error:?}"))?;
        admit_plugin(&session, plugin.endpoint()).await?;

        // An Empty List Refuses Every Capture
        assert!(capture_folders::load(&folders)?.is_empty());
        let (termination, artifacts) = copy(&session, device.path(), "refused").await?;
        assert_eq!(termination, Termination::Failed as i32);
        assert!(artifacts.is_empty());

        // An Added Folder Is Saved and Applies to the Next Execution Without a Restart
        let added = vec![device.path().display().to_string()];
        capture_folders::save(&folders, &added)?;
        owner.set_capture_folders(vec![device.path().to_path_buf()]).await;
        let (termination, artifacts) = copy(&session, device.path(), "applied").await?;
        assert_eq!(termination, Termination::Completed as i32);
        assert_eq!(artifacts, vec!["device/device.log".to_owned()]);
        owner.shutdown().await.map_err(|error| format!("{error:?}"))?;

        // A Restarted Desktop Starts Its Host With the Saved Folders
        plugin.close().await?;
        assert_eq!(capture_folders::load(&folders)?, added);
        let owner = DesktopSession::with_local_backend(ConnectionSession::default(), backend);
        let session = owner.local_client().await.map_err(|error| format!("{error:?}"))?;
        let options = PluginsGetOptions {
            context: session.call_options()?,
            ..Default::default()
        };
        let entry = session
            .execute(session.client().plugins.get_with("capture", options))
            .await?
            .entry
            .ok_or("Reopen Capture Fixture: Entry Removed")?;
        let options = PluginsDeleteOptions { context: session.call_options()? };
        session.execute(session.client().plugins.delete_with(entry.name, entry.revision, options)).await?;
        let mut plugin = PluginProcess::start(&dokime_gui::test_support::fixture("dokime-rust-capture"), true, &placement).await?;
        admit_plugin(&session, plugin.endpoint()).await?;
        let (termination, artifacts) = copy(&session, device.path(), "restarted").await?;
        assert_eq!(termination, Termination::Completed as i32);
        assert_eq!(artifacts, vec!["device/device.log".to_owned()]);
        owner.shutdown().await.map_err(|error| format!("{error:?}"))?;
        plugin.close().await?;
        Ok::<(), Box<dyn Error>>(())
    })
}
