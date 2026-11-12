/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Export and Compile the Host Protocol Without a Running Backend.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use clap::Parser;

// Local Imports
use dokime_cli::{Cli, Error};

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
async fn proto_export() -> Result<(), Box<dyn std::error::Error>> {
    // Export Without a Backend or Data Root.
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("contracts");
    let destination = output.to_str().unwrap();
    let cli = Cli::try_parse_from(["dokime", "proto", "export", "--out", destination])?;
    let configuration = crate::configuration::resolve(directory.path());
    cli.run(configuration.clone()).await?;

    // Check the Complete Exported File Set.
    let actual = crate::protocol::files(&output)?;
    let sources = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../proto");
    let expected = crate::protocol::files(&sources)?.into_iter().filter(|path| path.ends_with(".proto")).collect::<Vec<_>>();
    assert_eq!(actual, expected);

    // Parse Every Exported File With the Same Vendored Descriptor Compiler.
    let descriptor = directory.path().join("contracts.bin");
    let inputs = expected.iter().map(String::as_str).collect::<Vec<_>>();
    let status = crate::protocol::parse(&output, &inputs, &descriptor)?;
    assert!(status.success());
    let expected_descriptor = directory.path().join("sources.bin");
    assert!(crate::protocol::parse(&sources, &inputs, &expected_descriptor)?.success());
    assert_eq!(std::fs::read(descriptor)?, std::fs::read(expected_descriptor)?);

    // Refuse Unavailable Plugin Export Before Creating Any Output.
    let refused = directory.path().join("plugin");
    let cli = Cli::try_parse_from(["dokime", "proto", "export", "--plugin", "example", "--out", refused.to_str().unwrap()])?;
    let error = cli.run(configuration).await.unwrap_err();
    assert!(matches!(error, Error::Input(_)));
    assert!(!refused.exists());
    Ok(())
}
