/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Generate Every Authoritative SDK Example and Preserve Existing Work

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{error::Error, fs};

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

#[test]
fn scaffold_languages_and_export_protocol_without_a_host() -> Result<(), Box<dyn Error>> {
    super::credentials::run(Box::pin(async {
        tokio::time::timeout(super::HANG_GUARD, async {
            let directory = super::credentials::tempdir()?;
            for (language, examples, folders) in [
                ("rust", "sdk/rust/examples", ["plugin", "test"]),
                ("python", "sdk/python/examples", ["plugin", "test"]),
                ("csharp", "sdk/csharp/examples", ["Plugin", "Test"]),
                ("cpp", "sdk/cpp/examples", ["plugin", "test"]),
            ] {
                // Generate Both SDK Examples Without Running Their Journeys Twice
                let project = directory.path().join(language);
                let mut arguments = vec!["new", project.to_str().unwrap(), "--lang", language];
                if language == "cpp" {
                    arguments.extend([
                        "--sdk-url",
                        "https://example.com/sdk.tar.gz",
                        "--sdk-sha256",
                        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                    ]);
                }
                super::run(&arguments).await?;
                for folder in folders {
                    let root = super::root().join(examples).join(folder);
                    let mut pending = vec![root.clone()];
                    while let Some(directory) = pending.pop() {
                        for entry in fs::read_dir(directory)? {
                            let entry = entry?;
                            let path = entry.path();
                            if entry.file_type()?.is_dir() {
                                if !["target", "build", "bin", "obj", ".git", ".venv", "__pycache__"]
                                    .iter()
                                    .any(|name| entry.file_name() == *name)
                                {
                                    pending.push(path);
                                }
                                continue;
                            }
                            let destination = project.join(folder).join(path.strip_prefix(&root)?);
                            if language == "python" && entry.file_name() == "uv.lock" {
                                assert!(!destination.exists());
                                continue;
                            }
                            if path.file_name().is_some_and(|name| name == "CMakePresets.json") {
                                let presets: serde_json::Value = serde_json::from_slice(&fs::read(destination)?)?;
                                let coordinates = &presets["configurePresets"][0]["cacheVariables"];
                                assert_eq!(coordinates["DOKIME_SDK_URL"], arguments[5]);
                                assert_eq!(coordinates["DOKIME_SDK_SHA256"], arguments[7]);
                            } else {
                                assert_eq!(fs::read(destination)?, fs::read(path)?);
                            }
                        }
                    }
                }

                // Preserve the Existing Project on a Repeated Invocation
                fs::write(project.join("owned.txt"), b"retained work")?;
                let output = super::command().args(&arguments).output().await?;
                assert!(!output.status.success());
                assert_eq!(fs::read(project.join("owned.txt"))?, b"retained work");
            }
            // Export the Embedded Protocol Without Configuration or a Backend
            let protos = directory.path().join("proto");
            super::run(&["proto", "export", "--out", protos.to_str().unwrap()]).await?;
            for (relative, source) in dokime_protocol::SOURCES {
                assert_eq!(fs::read(protos.join(relative))?, source.as_bytes());
            }
            Ok(())
        })
        .await?
    }))
}

/* === Tests === */
