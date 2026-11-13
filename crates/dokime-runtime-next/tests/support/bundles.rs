/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verified Managed Plugin Bundle Fixtures

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

pub fn bundle(executable: &[u8], version: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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
pub fn upload(bytes: Vec<u8>) -> Vec<dokime::v1::PluginsInstallInput> {
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

/* === Private Functions === */

/* === Tests === */
