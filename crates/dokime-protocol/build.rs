/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::path::PathBuf;
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("proto");
    let output = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let files = [
        "common",
        "checkpoint",
        "value",
        "schema",
        "backend",
        "catalog",
        "planning",
        "query",
        "execution",
        "observation",
        "history",
        "reports",
        "appearance",
        "settings",
        "content",
        "prompts",
        "evidence",
        "notes",
        "documents",
        "plots",
        "plugins",
        "telemetry",
    ]
    .map(|name| root.join(format!("dokime/v1/{name}.proto")));
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
    }
    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc_bin_vendored::protoc_bin_path()?);
    config.file_descriptor_set_path(output.join("dokime.v1.bin"));
    // Descriptors remain portable across absolute checkout paths and consumers.
    config.skip_source_info();
    config.boxed(".dokime.v1.DocumentCard.body.owner");
    config.boxed(".dokime.v1.DocumentFrame.frame.header");
    config.boxed(".dokime.v1.DocumentLayoutNode.node.section");
    config.boxed(".dokime.v1.DocumentBinding.result.plot");
    config.type_attribute(
        ".dokime.v1.HistoricalDocumentCard.body",
        "#[allow(clippy::large_enum_variant)]",
    );
    config.type_attribute(
        ".dokime.v1.ReportHtmlFrame.payload",
        "#[allow(clippy::large_enum_variant)]",
    );
    config.type_attribute(
        ".dokime.v1.DiagnosticsBundleFrame.payload",
        "#[allow(clippy::large_enum_variant)]",
    );
    config.btree_map([
        ".dokime.v1.ValueObject.fields",
        ".dokime.v1.ParameterSchema.extensions",
        ".dokime.v1.ThemeFamily.variables",
        ".dokime.v1.ThemeVariant.variables",
        ".dokime.v1.PluginRuntimeCredentialBindings.bindings",
        ".dokime.v1.LogFrame.fields",
    ]);
    tonic_prost_build::configure()
        .build_client(std::env::var_os("CARGO_FEATURE_CLIENT").is_some())
        .build_server(std::env::var_os("CARGO_FEATURE_SERVER").is_some())
        .compile_with_config(
            config,
            &files,
            &[root, protoc_bin_vendored::include_path()?],
        )?;
    Ok(())
}

/* === Tests === */
