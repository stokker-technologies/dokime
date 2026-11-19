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
            let repository = super::root().canonicalize()?;
            for (language, folders) in [("rust", &[""][..]), ("python", &[""][..]), ("csharp", &[""][..]), ("cpp", &["plugin", "test"][..])]
            {
                // Create the Checkout's Starter With Its Native Local Dependency.
                let project = directory.path().join(language);
                let arguments = ["new", project.to_str().unwrap(), "--lang", language, "--from", repository.to_str().unwrap()];
                super::run(&arguments).await?;
                for folder in folders {
                    for file in ["dokime.toml", "mise.toml", ".gitignore", ".gitattributes"] {
                        assert!(project.join(folder).join(file).is_file(), "{language}/{folder}/{file}");
                    }
                }
                match language {
                    "rust" => {
                        let manifest: toml::Value = toml::from_str(&fs::read_to_string(project.join("Cargo.toml"))?)?;
                        assert_eq!(
                            std::path::Path::new(manifest["dependencies"]["dokime"]["path"].as_str().unwrap()).canonicalize()?,
                            repository.join("sdk/rust")
                        );
                        assert!(project.join("src/bin/client.rs").is_file());
                    }
                    "python" => {
                        let manifest: toml::Value = toml::from_str(&fs::read_to_string(project.join("pyproject.toml"))?)?;
                        assert!(manifest["tool"]["uv"]["sources"]["dokime-sdk"]["path"].is_str());
                        assert!(project.join("src/python/__main__.py").is_file());
                    }
                    "csharp" => {
                        assert!(project.join("Csharp.slnx").is_file());
                        let manifest = fs::read_to_string(project.join("src/Csharp/Csharp.csproj"))?;
                        assert!(manifest.contains("ProjectReference"));
                        assert!(!manifest.contains("PackageReference Include=\"Dokime\""));
                    }
                    _ => {
                        let manifest = fs::read_to_string(project.join("plugin/CMakeLists.txt"))?;
                        assert!(manifest.contains("add_subdirectory("));
                        assert!(!manifest.contains("FetchContent"));
                    }
                }

                // Preserve the Existing Project on a Repeated Invocation.
                fs::write(project.join("owned.txt"), b"retained work")?;
                let output = super::command().args(arguments).output().await?;
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
