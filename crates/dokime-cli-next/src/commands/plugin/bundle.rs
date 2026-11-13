/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bundle Any Native harmos.v1 Executable Without Compiling

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

// Third Party Imports
use prost::Message;
use sha2::Digest;

// Local Imports
use dokime_runtime_next::PluginProcess;

/* === Constants === */

const FILE_BYTES: u64 = 128 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long)]
    executable: PathBuf,

    #[arg(long)]
    out: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, _endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    // Freeze the Declared Identity and Catalog From the Native Executable
    let executable = std::fs::canonicalize(&args.executable)?;
    let binary = read(&executable, FILE_BYTES)?;
    let mut process = PluginProcess::start(&executable, false, &settings.placement()).await?;
    let description = process.describe().await;
    let closed = process.close().await;
    let description = description?;
    closed?;
    if description.id.is_empty() || description.catalog.is_none() {
        return Err("Bundle Plugin: Executable Must Describe Its Identity and Catalog".into());
    }
    let name = executable.file_name().and_then(|name| name.to_str()).ok_or("Bundle Plugin: Supply a UTF-8 Executable Filename")?;
    let path = format!("bin/{name}");
    let catalog = description.encode_to_vec();
    let compatibility = serde_json::to_vec(&serde_json::json!({"schema":1,"requires_dokime":format!("={}", env!("CARGO_PKG_VERSION"))}))?;
    let target = match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        _ => return Err("Bundle Plugin: Unsupported Native Target".into()),
    };
    let manifest = serde_json::json!({"schema":1,"identity":{"name":description.id,"version":description.version,"description":"",
        "package":{"authors":[],"license":null,"repository":null,"homepage":null,"documentation":null,"keywords":[],"categories":[]}},
        "files":[{"path":path,"size":binary.len(),"sha256":digest(&binary),"executable":{"facility":"sidecar","target":target,"registrations":[]}},
            {"path":"description.pb","size":catalog.len(),"sha256":digest(&catalog)},
            {"path":"dokime.json","size":compatibility.len(),"sha256":digest(&compatibility)}]});
    let parent = args.out.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut output = tempfile::NamedTempFile::new_in(parent)?;
    let mut archive = tar::Builder::new(output.as_file_mut());
    let bytes = serde_json::to_vec(&manifest)?;
    for (path, bytes, mode) in [
        ("manifest.json", bytes.as_slice(), 0o644),
        (path.as_str(), binary.as_slice(), 0o755),
        ("description.pb", catalog.as_slice(), 0o644),
        ("dokime.json", compatibility.as_slice(), 0o644),
    ] {
        let mut header = tar::Header::new_ustar();
        header.set_size(bytes.len() as u64);
        header.set_mode(mode);
        header.set_mtime(0);
        header.set_uid(0);
        header.set_gid(0);
        header.set_cksum();
        archive.append_data(&mut header, path, bytes)?;
    }
    archive.finish()?;
    drop(archive);
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

fn read(path: &Path, limit: u64) -> crate::Result<Vec<u8>> {
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > limit {
        return Err("Read Plugin Input: File Exceeds the Byte Bound".into());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("Read Plugin Input: File Grew Beyond the Byte Bound".into());
    }
    Ok(bytes)
}

fn digest(bytes: &[u8]) -> String {
    sha2::Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Tests === */
