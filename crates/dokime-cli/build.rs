/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Embed the Source Commit and Publication Channel of This Build

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

// Third Party Imports

// Local Imports

/* === Constants === */

const COMMIT_DIGITS: std::ops::RangeInclusive<usize> = 7..=64;
const UNKNOWN: &str = "unknown";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() {
    embed_examples();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    for name in ["GITHUB_SHA", "GITHUB_REF", "GITHUB_WORKFLOW_REF"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let manifest = env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest directory");
    let manifest = PathBuf::from(manifest);
    // CI Names Its Clean Checkout; Local Builds Ask Git And Mark Uncommitted Tracked Changes.
    let declared = env::var("GITHUB_SHA").ok().filter(|value| commit(value));
    let revision = declared.or_else(|| checkout(&manifest)).unwrap_or_else(|| UNKNOWN.to_owned());
    println!("cargo:rustc-env=DOKIME_SOURCE_REVISION={revision}");
    println!("cargo:rustc-env=DOKIME_BUILD_CHANNEL={}", channel());
}

fn embed_examples() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest directory"));
    let repository = manifest.join("../..");
    let mut source = String::new();
    for (name, language, examples) in
        [("RUST", "rust", "examples"), ("PYTHON", "python", "examples"), ("CSHARP", "csharp", "examples"), ("CPP", "cpp", "examples")]
    {
        source.push_str(&format!("const {name}: &[(&str, &[u8])] = &[\n"));
        for example in ["plugin", "test"] {
            let folder = if language == "csharp" { if example == "plugin" { "Plugin" } else { "Test" } } else { example };
            let root = repository.join("sdk").join(language).join(examples).join(folder);
            println!("cargo:rerun-if-changed={}", root.display());
            let mut pending = vec![root.clone()];
            while let Some(directory) = pending.pop() {
                let mut entries = fs::read_dir(&directory)
                    .unwrap_or_else(|error| panic!("Embed {language} {example} Example: {error}"))
                    .map(|entry| entry.expect("Read example entry"))
                    .collect::<Vec<_>>();
                entries.sort_by_key(|entry| entry.file_name());
                for entry in entries {
                    let path = entry.path();
                    let kind = entry.file_type().expect("Read example file type");
                    if kind.is_dir() {
                        let name = entry.file_name();
                        if !["target", "build", "bin", "obj", ".git", ".venv", "__pycache__"].iter().any(|excluded| name == *excluded) {
                            pending.push(path);
                        }
                        continue;
                    }
                    if !kind.is_file() {
                        panic!("Embed Example: Expected a Regular File at {}", path.display());
                    }
                    let relative = path
                        .strip_prefix(&root)
                        .expect("Example file remains below its root")
                        .to_str()
                        .expect("Example path is UTF-8")
                        .replace('\\', "/");
                    let destination = format!("{folder}/{relative}");
                    source.push_str(&format!(
                        "({destination:?}, include_bytes!({:?})),\n",
                        path.canonicalize().expect("Resolve example file")
                    ));
                }
            }
        }
        source.push_str("];\n");
    }
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo supplies the build output directory"));
    fs::write(output.join("examples.rs"), source).expect("Embed SDK examples");
}

// The publishers' own distinctions: stable tags run the release workflow, the nightly workflow publishes nightly.
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
    // Reads Without Refreshing The Index, So This Script Never Invalidates Its Own Run.
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

#[allow(clippy::disallowed_methods, reason = "build metadata runs before the runtime process constructor is available")]
fn git(directory: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new("git").arg("-C").arg(directory).args(arguments).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.trim().to_owned())
}

fn commit(value: &str) -> bool {
    let digit = |byte: u8| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte);
    COMMIT_DIGITS.contains(&value.len()) && value.bytes().all(digit)
}

/* === Tests === */
