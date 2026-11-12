/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Installs And Removes Managed Plugins

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn bundle(executable: &[u8], version: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use sha2::Digest;
    let target = match std::env::consts::OS {
        "windows" => "x86_64-pc-windows-msvc",
        _ => "x86_64-unknown-linux-gnu",
    };
    let filename = format!("bin/fixture{}", std::env::consts::EXE_SUFFIX);
    let compatibility = br#"{"schema":1,"requires_dokime":">=0.1.0"}"#;
    let manifest = serde_json::to_vec(
        &serde_json::json!({ "schema": 1, "identity": { "name": "thermal-fixture", "version": version, "description": "Thermal Fixture" }, "files": [
        { "path": filename, "size": executable.len(), "sha256": sha2::Sha256::digest(executable).iter().map(|byte| format!("{byte:02x}")).collect::<String>(), "executable": { "facility": "sidecar", "target": target } },
        { "path": "dokime.json", "size": compatibility.len(), "sha256": sha2::Sha256::digest(compatibility).iter().map(|byte| format!("{byte:02x}")).collect::<String>() }
    ] }),
    )?;
    let mut archive = tar::Builder::new(Vec::new());
    for (path, bytes) in
        [("manifest.json", manifest.as_slice()), (filename.as_str(), executable), ("dokime.json", compatibility.as_slice())]
    {
        let mut header = tar::Header::new_ustar();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o700);
        header.set_cksum();
        archive.append_data(&mut header, path, bytes)?;
    }
    Ok(archive.into_inner()?)
}
fn upload(bytes: Vec<u8>) -> Vec<dokime::v1::PluginsInstallInput> {
    use dokime::v1;
    use sha2::Digest;
    let header = v1::InstallPluginHeader {
        size_bytes: bytes.len() as u64,
        sha256: sha2::Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect(),
    };
    let mut frames = vec![
        v1::PluginsInstallInput {
            value: Some(v1::plugins_install_input::Value::Request(v1::PluginsInstallRequest {})),
        },
        v1::PluginsInstallInput {
            value: Some(v1::plugins_install_input::Value::Item(v1::InstallPluginFrame {
                frame: Some(v1::InstallPluginFrameFrame {
                    kind: Some(v1::install_plugin_frame_frame::Kind::Header(header)),
                }),
            })),
        },
    ];
    for (index, data) in bytes.chunks(1024 * 1024).enumerate() {
        frames.push(v1::PluginsInstallInput {
            value: Some(v1::plugins_install_input::Value::Item(v1::InstallPluginFrame {
                frame: Some(v1::InstallPluginFrameFrame {
                    kind: Some(v1::install_plugin_frame_frame::Kind::Chunk(v1::InstallPluginChunk {
                        offset: (index * 1024 * 1024) as u64,
                        data: data.to_vec(),
                    })),
                }),
            })),
        });
    }
    frames
}

/* === Tests === */

#[tokio::test]
async fn client_installs_and_removes_managed_plugins() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        use dokime::v1;

        // Package the Existing SDK Fixture with Verified Platform Metadata
        let executable = std::fs::read(super::support::thermal_fixture()?)?;
        let first_archive = bundle(&executable, "1.0.0")?;
        let root = tempfile::tempdir()?;
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut plugins = v1::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        let installed = plugins.install(tokio_stream::iter(upload(first_archive.clone()))).await?.into_inner();
        assert_eq!(installed.plugin.as_ref().unwrap().state, v1::PluginState::Ready as i32);
        assert!(matches!(installed.plugin.as_ref().unwrap().placement, Some(v1::plugin::Placement::Managed(_))));
        assert_eq!(
            plugins.get_installation(v1::PluginsGetInstallationRequest { install_id: installed.install_id.clone() }).await?.into_inner(),
            installed
        );
        let page = plugins.list_installations(v1::PluginsListInstallationsRequest { page_size: 1, after: None }).await?.into_inner();
        assert_eq!(page.total_count, 1);

        // Disable and Replace the Bundle While Keeping Its Disabled State
        let disabled = plugins
            .set_enabled(v1::PluginsSetEnabledRequest {
                install_id: installed.install_id.clone(),
                enabled: false,
            })
            .await?
            .into_inner();
        assert_eq!(disabled.state, v1::PluginInstallationState::Disabled as i32);
        let mut health = v1::health_client::HealthClient::connect(process.url.clone()).await?;
        assert!(health.list_health(v1::HealthListHealthRequest::default()).await?.into_inner().items.is_empty());
        let mut replacement_input: Vec<_> = upload(bundle(&executable, "2.0.0")?)
            .into_iter()
            .map(|frame| v1::PluginsInstallRunInput {
                value: frame.value.map(|value| match value {
                    v1::plugins_install_input::Value::Request(request) => v1::plugins_install_run_input::Value::Request(request),
                    v1::plugins_install_input::Value::Item(item) => v1::plugins_install_run_input::Value::Item(item),
                }),
            })
            .collect();
        replacement_input.push(v1::PluginsInstallRunInput {
            value: Some(v1::plugins_install_run_input::Value::Finish(v1::ClientEmpty {})),
        });
        let mut output = plugins.run_install(tokio_stream::iter(replacement_input)).await?.into_inner();
        let Some(v1::plugins_install_run_output::Value::Response(replacement)) = output.message().await?.unwrap().value else {
            panic!("Expected Installation Response");
        };
        assert!(output.message().await?.is_none());
        assert_eq!(replacement.state, v1::PluginInstallationState::Disabled as i32);
        assert!(plugins.get_installation(v1::PluginsGetInstallationRequest { install_id: installed.install_id }).await.is_err());
        let enabled = plugins
            .set_enabled(v1::PluginsSetEnabledRequest {
                install_id: replacement.install_id.clone(),
                enabled: true,
            })
            .await?
            .into_inner();
        assert_eq!(enabled.plugin.as_ref().unwrap().state, v1::PluginState::Ready as i32);

        // Reject an Incomplete Upload Without Changing the Installation Inventory
        let mut incomplete = upload(first_archive);
        incomplete.pop();
        assert!(plugins.install(tokio_stream::iter(incomplete)).await.is_err());
        assert_eq!(plugins.list_installations(v1::PluginsListInstallationsRequest::default()).await?.into_inner().total_count, 1);
        process.close().await?;

        // Restart Starts the Verified Managed Process, Then Remove Only Its Installation
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut plugins = v1::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        let restarted = plugins
            .get_installation(v1::PluginsGetInstallationRequest {
                install_id: replacement.install_id.clone(),
            })
            .await?
            .into_inner();
        assert_eq!(restarted.plugin.as_ref().unwrap().state, v1::PluginState::Ready as i32);
        plugins.remove(v1::RemovePlugin { entry: replacement.install_id }).await?;
        assert!(plugins.list(v1::ListPlugins {}).await?.into_inner().plugins.is_empty());
        assert_eq!(plugins.list_installations(v1::PluginsListInstallationsRequest::default()).await?.into_inner().total_count, 0);
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
