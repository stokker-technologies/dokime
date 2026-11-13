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

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    for name in ["GITHUB_SHA", "GITHUB_REF", "GITHUB_WORKFLOW_REF"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest directory");
    let manifest = std::path::PathBuf::from(manifest);
    // CI Names Its Clean Checkout; Local Builds Ask Git And Mark Uncommitted Tracked Changes.
    let declared = std::env::var("GITHUB_SHA").ok().filter(|value| commit(value));
    let revision = declared.or_else(|| checkout(&manifest)).unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=DOKIME_SOURCE_REVISION={revision}");
    println!("cargo:rustc-env=DOKIME_BUILD_CHANNEL={}", channel());

    // The Shell Names the Product from the Generated Brand, Like the Web UI
    let label = brand_text("default-organization.json", "label");
    println!("cargo:rustc-env=DOKIME_BRAND_LABEL={label}");

    // Track Added and Removed Files When the Frontend Is Embedded.
    if !tauri_build::is_dev() {
        let frontend = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist");
        println!("cargo:rerun-if-changed={}", frontend.display());
    }
    embed_organization(&manifest);
    tauri_build::build();

    // Tauri embeds its manifest in binaries only. Library test executables also
    // load native dialogs, which require Common Controls v6 before startup.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
        // A side-by-side `.manifest` file stays behind when nextest archives the test executable,
        // so every target, including library unit tests, embeds its own manifest.
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        // The Binary Already Links Tauri's Manifest Resource; Disable A Second Linker-Generated One.
        println!("cargo:rustc-link-arg-bin=dokime-gui=/MANIFEST:NO");
    }
}

fn brand_text(file: &str, field: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/generated/organization").join(file);
    println!("cargo:rerun-if-changed={}", path.display());
    let bytes = std::fs::read(&path).expect("generated brand file");
    let manifest: serde_json::Value = serde_json::from_slice(&bytes).expect("generated brand JSON");
    manifest[field].as_str().filter(|text| !text.trim().is_empty()).expect("generated brand text field").to_owned()
}

/* === Tests === */

// The publishers' own distinctions: stable tags run the release workflow, the nightly workflow publishes nightly.
fn channel() -> &'static str {
    let reference = std::env::var("GITHUB_REF").unwrap_or_default();
    let workflow = std::env::var("GITHUB_WORKFLOW_REF").unwrap_or_default();
    if reference.starts_with("refs/tags/v") && !reference.contains("-dev.") {
        "release"
    } else if workflow.contains("/.github/workflows/nightly.yml@") {
        "nightly"
    } else {
        ""
    }
}

fn checkout(manifest: &std::path::Path) -> Option<String> {
    // An enclosing repository that does not track this package, such as a home folder, names no build.
    git(manifest, &["ls-files", "--error-unmatch", "Cargo.toml"])?;
    let head = git(manifest, &["rev-parse", "HEAD"]).filter(|value| commit(value))?;
    watch(manifest);
    // Reads Without Refreshing The Index, So This Script Never Invalidates Its Own Run.
    let changes = git(manifest, &["--no-optional-locks", "status", "--porcelain", "--untracked-files=no"])?;
    Some(if changes.is_empty() { head } else { format!("{head}-dirty") })
}

fn watch(manifest: &std::path::Path) {
    let mut names = vec!["HEAD".to_owned(), "index".to_owned(), "packed-refs".to_owned()];
    names.extend(git(manifest, &["symbolic-ref", "-q", "HEAD"]));
    let paths = names.iter().filter_map(|name| git(manifest, &["rev-parse", "--path-format=absolute", "--git-path", name]));
    // A missing file would rerun this script on every build.
    for path in paths.filter(|path| std::path::Path::new(path).is_file()) {
        println!("cargo:rerun-if-changed={path}");
    }
}

#[allow(clippy::disallowed_methods, reason = "a desktop build script runs before the native runtime and opens no window")]
fn git(directory: &std::path::Path, arguments: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git").arg("-C").arg(directory).args(arguments).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.trim().to_owned())
}

fn commit(value: &str) -> bool {
    let digit = |byte: u8| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte);
    (7..=64).contains(&value.len()) && value.bytes().all(digit)
}

fn embed_organization(manifest: &std::path::Path) {
    let root = manifest.join("../../../organization").canonicalize().expect("organization directory");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut pending = vec![root.clone()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).expect("organization entries") {
            let entry = entry.expect("organization file");
            let path = entry.path();
            if entry.file_type().expect("organization file type").is_dir() {
                pending.push(path);
            } else {
                let relative = path.strip_prefix(&root).expect("pack file").to_string_lossy().replace('\\', "/");
                files.push(format!("({relative:?}, include_bytes!({:?}) as &[u8]),", path.to_str().expect("pack path")));
            }
        }
    }
    files.sort();
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("build output"));
    std::fs::write(output.join("organization.rs"), format!("&[{}]", files.join("\n"))).expect("embedded organization");
}
