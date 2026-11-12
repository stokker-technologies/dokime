/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Generate the Authoritative Python Starter Without a Backend.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{fs, path::Path};

// Third Party Imports
use clap::Parser;

// Local Imports
use dokime_cli::Cli;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn python_starter_matches_source_and_preserves_existing_destination() -> Result<(), Box<dyn std::error::Error>> {
    // Generate Without a Backend.
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("capture-plugin");
    let configuration = crate::configuration::resolve(directory.path());
    let cli = Cli::try_parse_from(["dokime", "new", output.to_str().unwrap(), "--lang", "python"])?;
    cli.run(configuration.clone()).await?;

    // Compare Every File to the Single Starter Source.
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sdk/python/examples/starter");
    let files = crate::protocol::files(&output)?;
    assert_eq!(
        files,
        [".gitattributes", ".gitignore", "main.py", "mise.toml", "pyproject.toml", "suite.py", "tests/host.py", "tests/test_capture.py"]
    );
    for relative in &files {
        let expected = fs::read(source.join(relative))?;
        let expected = if relative == "pyproject.toml" {
            String::from_utf8(expected)?.replacen("\"dokime-starter\"", "\"capture-plugin\"", 1).into_bytes()
        } else {
            expected
        };
        assert_eq!(fs::read(output.join(relative))?, expected, "{relative}");
    }

    // Refuse an Existing Destination Without Overwriting Its Files.
    fs::write(output.join("main.py"), b"preserve this file")?;
    let cli = Cli::try_parse_from(["dokime", "new", output.to_str().unwrap(), "--lang", "python"])?;
    assert!(cli.run(configuration).await.is_err());
    assert_eq!(fs::read(output.join("main.py"))?, b"preserve this file");
    assert_eq!(crate::protocol::files(&output)?, files);
    Ok(())
}
