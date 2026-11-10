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
    let expected = [
        "dokime/v1/catalog.proto",
        "dokime/v1/documents.proto",
        "dokime/v1/execution.proto",
        "dokime/v1/host.proto",
        "dokime/v1/plugins.proto",
        "dokime/v1/values.proto",
        "harmos/v1/plugin.proto",
    ];
    assert_eq!(actual, expected);

    // Parse Every Exported File With the Same Vendored Descriptor Compiler.
    let descriptor = directory.path().join("contracts.bin");
    let status = crate::protocol::parse(&output, &expected, &descriptor)?;
    assert!(status.success());

    // Refuse Unavailable Plugin Export Before Creating Any Output.
    let refused = directory.path().join("plugin");
    let cli = Cli::try_parse_from(["dokime", "proto", "export", "--plugin", "example", "--out", refused.to_str().unwrap()])?;
    let error = cli.run(configuration).await.unwrap_err();
    assert!(matches!(error, Error::Input(_)));
    assert!(!refused.exists());
    Ok(())
}
