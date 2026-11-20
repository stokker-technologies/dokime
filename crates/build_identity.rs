/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Derive Shared Build Identity

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

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

pub fn emit() -> (String, &'static str) {
    for name in ["GITHUB_SHA", "GITHUB_REF", "GITHUB_WORKFLOW_REF"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let manifest = env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest directory");
    let manifest = PathBuf::from(manifest);
    // Resolve the Source Revision
    let declared = env::var("GITHUB_SHA").ok().filter(|value| commit(value));
    let revision = declared.or_else(|| checkout(&manifest)).unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=DOKIME_SOURCE_REVISION={revision}");
    let channel = channel();
    println!("cargo:rustc-env=DOKIME_BUILD_CHANNEL={channel}");
    (revision, channel)
}

#[allow(clippy::disallowed_methods, reason = "build metadata runs before the runtime process constructor is available")]
pub fn git(directory: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new("git").arg("-C").arg(directory).args(arguments).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.trim().to_owned())
}

/* === Private Functions === */

// Publication Workflows Own Release and Nightly Channels
fn channel() -> &'static str {
    let reference = env::var("GITHUB_REF").unwrap_or_default();
    let workflow = env::var("GITHUB_WORKFLOW_REF").unwrap_or_default();
    if reference.starts_with("refs/tags/v") && !reference.contains("-dev.") {
        "release"
    } else if workflow.contains("/.github/workflows/nightly.yml@") {
        "nightly"
    } else {
        ""
    }
}

fn checkout(manifest: &Path) -> Option<String> {
    // An enclosing repository that does not track this package, such as a home folder, names no build.
    git(manifest, &["ls-files", "--error-unmatch", "Cargo.toml"])?;
    let head = git(manifest, &["rev-parse", "HEAD"]).filter(|value| commit(value))?;
    watch(manifest);
    // Read Changes Without Refreshing the Index
    let changes = git(manifest, &["--no-optional-locks", "status", "--porcelain", "--untracked-files=no"])?;
    Some(if changes.is_empty() { head } else { format!("{head}-dirty") })
}

fn watch(manifest: &Path) {
    let mut names = vec!["HEAD".to_owned(), "index".to_owned(), "packed-refs".to_owned()];
    names.extend(git(manifest, &["symbolic-ref", "-q", "HEAD"]));
    let paths = names.iter().filter_map(|name| git(manifest, &["rev-parse", "--path-format=absolute", "--git-path", name]));
    // A missing file would rerun this script on every build.
    for path in paths.filter(|path| Path::new(path).is_file()) {
        println!("cargo:rerun-if-changed={path}");
    }
}

fn commit(value: &str) -> bool {
    let digit = |byte: u8| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte);
    (7..=64).contains(&value.len()) && value.bytes().all(digit)
}

/* === Tests === */
