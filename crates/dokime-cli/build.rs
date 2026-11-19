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
    ops::RangeInclusive,
    path::{Path, PathBuf},
    process::Command,
};

// Third Party Imports

// Local Imports

/* === Constants === */

const COMMIT_DIGITS: RangeInclusive<usize> = 7..=64;
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
    // CI Names Its Clean Checkout; Local Builds Ask Git and Mark Uncommitted Tracked Changes.
    let declared = env::var("GITHUB_SHA").ok().filter(|value| commit(value));
    let revision = declared.or_else(|| checkout(&manifest)).unwrap_or_else(|| UNKNOWN.to_owned());
    println!("cargo:rustc-env=DOKIME_SOURCE_REVISION={revision}");
    println!("cargo:rustc-env=DOKIME_BUILD_CHANNEL={}", channel());
}

fn embed_examples() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest directory"));
    let repository = manifest.join("../..");
    let mut source = String::new();
    embed_csharp(&repository, &mut source);
    for (name, language, folders) in
        [("RUST", "rust", &["starter"][..]), ("PYTHON", "python", &["starter"][..]), ("CPP", "cpp", &["plugin", "test"][..])]
    {
        source.push_str(&format!("const {name}: &[(&str, &[u8])] = &[\n"));
        for folder in folders {
            let root = format!("sdk/{language}/examples/{folder}");
            println!("cargo:rerun-if-changed={}", repository.join(&root).display());
            let files = git(&repository, &["ls-files", "-z", "--", &root])
                .unwrap_or_else(|| panic!("Embed {language} Example: Run the Build From a Git Checkout"));
            for file in files.split('\0').filter(|file| !file.is_empty()) {
                let relative = file.strip_prefix(&format!("{root}/")).expect("Example file remains below its root");
                let path = Path::new(relative);
                let filename = path.file_name().and_then(|name| name.to_str()).expect("Example filename is UTF-8");
                let excluded_directory = path.components().any(|component| {
                    matches!(
                        component.as_os_str().to_str(),
                        Some("target" | "build" | ".git" | ".venv" | "__pycache__" | "_input" | "_output" | "credentials")
                    )
                });
                if excluded_directory
                    || (language == "python" && filename == "uv.lock")
                    || filename == ".env"
                    || (filename.starts_with(".env.") && filename != ".env.example")
                    || matches!(filename, "credentials.json" | ".netrc" | ".pypirc" | "id_rsa" | "id_ed25519")
                    || [".key", ".pem", ".pair", ".pyc"].iter().any(|extension| filename.ends_with(extension))
                {
                    continue;
                }
                let path = repository.join(file);
                assert!(fs::symlink_metadata(&path).expect("Read example file type").is_file(), "Embed Example: Use Regular Files");
                let destination = if language != "cpp" { relative.to_owned() } else { format!("{folder}/{relative}") };
                source.push_str(&format!("({destination:?}, include_bytes!({:?})),\n", path.canonicalize().expect("Resolve example file")));
            }
        }
        source.push_str("];\n");
    }
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo supplies the build output directory"));
    fs::write(output.join("examples.rs"), source).expect("Embed SDK examples");
}

fn embed_csharp(repository: &Path, source: &mut String) {
    let root = "sdk/csharp/examples/Starter";
    println!("cargo:rerun-if-changed={}", repository.join(root).display());
    let files = git(repository, &["ls-files", "-z", "--", root]).expect("Embed C# Starter: Run the Build From a Git Checkout");
    source.push_str("const CSHARP: &[(&str, &[u8])] = &[\n");
    for file in files.split('\0').filter(|file| !file.is_empty()) {
        let relative = file.strip_prefix(&format!("{root}/")).expect("A C# starter file remains below its root");
        let path = Path::new(relative);
        let filename = path.file_name().and_then(|name| name.to_str()).expect("A C# starter filename is UTF-8");
        if path.components().any(|component| matches!(component.as_os_str().to_str(), Some("bin" | "obj" | ".git" | "_input" | "_output")))
            || relative.starts_with("assets/plugin/")
            || filename == ".env"
            || (filename.starts_with(".env.") && filename != ".env.example")
            || [".key", ".pem", ".pair", ".dokime"].iter().any(|extension| filename.ends_with(extension))
        {
            continue;
        }
        let path = repository.join(file);
        assert!(fs::symlink_metadata(&path).expect("Read C# starter file type").is_file(), "Embed C# Starter: Use Regular Files");
        source.push_str(&format!("({relative:?}, include_bytes!({:?})),\n", path.canonicalize().expect("Resolve C# starter file")));
    }
    source.push_str("];\n");
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
    // Reads Without Refreshing the Index, so This Script Never Invalidates Its Own Run.
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
