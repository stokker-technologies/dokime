/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bundle Any Native Plugin Executable Without Compiling

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::Write,
    path::{Path, PathBuf},
};

// Third Party Imports
use harmos_runtime::bundle::Identity;
use prost::Message;

// Local Imports
use crate::error::Error;
use dokime_runtime::PluginProcess;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long)]
    executable: PathBuf,

    #[arg(long)]
    out: PathBuf,

    #[arg(long = "asset")]
    assets: Vec<PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, _endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    // Freeze the Declared Identity and Declarations From the Native Executable
    let executable = std::fs::canonicalize(&args.executable)?;
    let mut process = PluginProcess::start(&executable, false, &settings.placement()).await?;
    let description = process.describe().await;
    let closed = process.close().await;
    let description = description?;
    closed?;
    if description.id.is_empty() || description.declarations.is_none() {
        return Err("Bundle Plugin: Executable Must Describe Its Identity and Declarations".into());
    }
    let identity = Identity {
        name: description.id.clone(),
        version: description.version.parse().map_err(|source| Error::Invalid(format!("Bundle Plugin: Read Its Version: {source}")))?,
        description: String::new(),
        package: Default::default(),
    };
    let declarations = description.encode_to_vec();
    let bytes = dokime_packaging::bundle::create(&executable, &args.assets, identity, Some(&declarations))?;
    let parent = args.out.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut output = tempfile::NamedTempFile::new_in(parent)?;
    output.write_all(&bytes)?;
    output.flush()?;
    output.as_file().sync_all()?;
    inspect(output.path())?;
    output.persist_noclobber(&args.out)?;
    println!("{}", args.out.display());
    Ok(())
}

pub(super) fn inspect(path: &Path) -> crate::Result<serde_json::Value> {
    Ok(dokime_packaging::bundle::inspect(path)?)
}

/* === Private Functions === */

/* === Tests === */
