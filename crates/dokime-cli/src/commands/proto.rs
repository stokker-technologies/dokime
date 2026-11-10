/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Export Embedded Protocol Sources Without Connecting to a Backend.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{fs, path::PathBuf};

// Third Party Imports

// Local Imports

/* === Constants === */

/// Embedded Dokime and Harmos sources; well-known imports come from protobuf.
const SOURCES: &[(&str, &str)] = &[
    ("dokime/v1/catalog.proto", include_str!("../../../../proto/dokime/v1/catalog.proto")),
    ("dokime/v1/documents.proto", include_str!("../../../../proto/dokime/v1/documents.proto")),
    ("dokime/v1/execution.proto", include_str!("../../../../proto/dokime/v1/execution.proto")),
    ("dokime/v1/host.proto", include_str!("../../../../proto/dokime/v1/host.proto")),
    ("dokime/v1/plugins.proto", include_str!("../../../../proto/dokime/v1/plugins.proto")),
    ("dokime/v1/values.proto", include_str!("../../../../proto/dokime/v1/values.proto")),
    ("harmos/v1/plugin.proto", include_str!("../../../../proto/harmos/v1/plugin.proto")),
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Export {
    #[arg(long, help = "Plugin Name; Plugin Export Is Not Available Yet.")]
    plugin: Option<String>,

    #[arg(long, help = "Destination Directory for Protobuf Sources.")]
    out: PathBuf,
}

/* === Enums === */

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    #[command(about = "Write the Embedded Dokime and Harmos Protobuf Sources.")]
    Export(Export),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn run(command: Command) -> crate::Result<()> {
    let Command::Export(args) = command;
    if let Some(plugin) = args.plugin {
        return Err(crate::Error::Input(format!(
            "plugin export is not available yet for `{plugin}`; omit --plugin to export Dokime and Harmos contracts, or obtain the plugin's .proto sources from its author"
        )));
    }
    for (relative, source) in SOURCES {
        let destination = args.out.join(relative);
        fs::create_dir_all(destination.parent().expect("embedded source has a package directory"))?;
        fs::write(destination, source)?;
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
