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
use std::{collections::BTreeSet, error::Error, fs};

// Third Party Imports
use serde_json::Value;
use tokio::process::Command;

// Local Imports
use dokime_runtime::process;

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
                ("rust", "sdk/rust/examples", &["plugin", "test"][..]),
                ("python", "sdk/python/examples/starter", &[""][..]),
                ("csharp", "sdk/csharp/examples", &["Plugin", "Test"][..]),
                ("cpp", "sdk/cpp/examples", &["plugin", "test"][..]),
            ] {
                // Generate the Language-Specific Project Layout
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
                    // Compare the Complete Created Inventory With the Tracked Starter Sources.
                    let source = if folder.is_empty() { examples.to_owned() } else { format!("{examples}/{folder}") };
                    let output = Command::from(process::command("git"))
                        .arg("-C")
                        .arg(super::root())
                        .args(["ls-files", "-z", "--", &source])
                        .output()
                        .await?;
                    if !output.status.success() {
                        return Err("Inspect Starter Sources: Run the Story From a Git Checkout".into());
                    }
                    let files = String::from_utf8(output.stdout)?;
                    let mut expected = BTreeSet::new();
                    for file in files.split('\0').filter(|file| !file.is_empty()) {
                        let relative =
                            file.strip_prefix(&format!("{source}/")).ok_or("Inspect Starter Sources: Keep Files Under Their Project")?;
                        if language == "python" && relative == "uv.lock" {
                            continue;
                        }
                        let relative =
                            if language == "python" { relative.replace("starter_project", language) } else { relative.to_owned() };
                        expected.insert(relative.clone());
                        let destination = project.join(folder).join(&relative);
                        if relative == "CMakePresets.json" {
                            let presets: Value = serde_json::from_slice(&fs::read(destination)?)?;
                            let coordinates = &presets["configurePresets"][0]["cacheVariables"];
                            assert_eq!(coordinates["DOKIME_SDK_URL"], arguments[5]);
                            assert_eq!(coordinates["DOKIME_SDK_SHA256"], arguments[7]);
                        } else {
                            let bytes = fs::read(super::root().join(file))?;
                            let expected = if language == "python" {
                                match str::from_utf8(&bytes) {
                                    Ok(text) => text.replace("starter_project", language).replace("starter-project", language).into_bytes(),
                                    Err(_) => bytes,
                                }
                            } else {
                                bytes
                            };
                            assert_eq!(fs::read(destination)?, expected);
                        }
                    }
                    assert!(!expected.is_empty());
                    let root = project.join(folder);
                    let mut actual = BTreeSet::new();
                    let mut pending = vec![root.clone()];
                    while let Some(directory) = pending.pop() {
                        for entry in fs::read_dir(directory)? {
                            let entry = entry?;
                            let path = entry.path();
                            if entry.file_type()?.is_dir() {
                                pending.push(path);
                                continue;
                            }
                            actual.insert(path.strip_prefix(&root)?.to_str().ok_or("Inspect Starter: Use UTF-8 Paths")?.replace('\\', "/"));
                        }
                    }
                    assert_eq!(actual, expected);
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
