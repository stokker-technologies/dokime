/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Generate Every Authoritative SDK Starter and Preserve Existing Work

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

#[tokio::test]
async fn scaffold_languages_and_export_protocol_without_a_host() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::HANG_GUARD, async {
        let directory = super::credentials::tempdir()?;
        for (language, folder, file) in [
            ("rust", "sdk/rust/starter", "src/main.rs"),
            ("python", "sdk/python/examples/starter", "suite.py"),
            ("csharp", "sdk/csharp/starter", "Program.cs"),
            ("cpp", "sdk/cpp/starter", "main.cpp"),
        ] {
            // Generate the SDK's Own Starter
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
            assert_eq!(std::fs::read(project.join(file))?, std::fs::read(super::root().join(folder).join(file))?);
            if language == "python" {
                for file in ["host.py", "test_capture.py"] {
                    assert_eq!(
                        std::fs::read(project.join("tests").join(file))?,
                        std::fs::read(super::root().join("sdk/tests/fixtures/python/starter").join(file))?
                    );
                }
            }
            if language == "cpp" {
                let presets: serde_json::Value = serde_json::from_slice(&std::fs::read(project.join("CMakePresets.json"))?)?;
                let coordinates = &presets["configurePresets"][0]["cacheVariables"];
                assert_eq!(coordinates["DOKIME_SDK_URL"], arguments[5]);
                assert_eq!(coordinates["DOKIME_SDK_SHA256"], arguments[7]);
            }

            // Preserve the Existing Project on a Repeated Invocation
            std::fs::write(project.join("owned.txt"), b"retained work")?;
            let output = super::command().args(&arguments).output().await?;
            assert!(!output.status.success());
            assert_eq!(std::fs::read(project.join("owned.txt"))?, b"retained work");
        }
        // Export the Embedded Protocol Without Configuration or a Backend
        let protos = directory.path().join("proto");
        super::run(&["proto", "export", "--out", protos.to_str().unwrap()]).await?;
        for (relative, source) in dokime_protocol::SOURCES {
            assert_eq!(std::fs::read(protos.join(relative))?, source.as_bytes());
        }
        Ok(())
    })
    .await?
}

/* === Tests === */
