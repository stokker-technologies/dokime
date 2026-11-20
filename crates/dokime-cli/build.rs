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

mod build_identity;
mod source_identity {
    include!("../build_identity.rs");
}

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    env, fs,
    path::{Path, PathBuf},
};

// Third Party Imports
use tauri_winres::WindowsResource;

// Local Imports
use source_identity::git;

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
    embed_examples();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    let (revision, channel) = source_identity::emit();
    let version = env::var("CARGO_PKG_VERSION").expect("Cargo supplies the package version");
    let version = build_identity::version(&version, &revision, channel);
    println!("cargo:rustc-env=DOKIME_BUILD_VERSION={version}");

    // Embed the Same Identity in Windows Explorer.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        WindowsResource::new()
            .set("ProductName", "Dokime")
            .set("ProductVersion", &version)
            .compile()
            .expect("Embed Command Version: Install the Windows SDK Resource Compiler");
    }
}

fn embed_examples() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest directory"));
    let repository = manifest.join("../..");
    let mut source = String::new();
    embed_csharp(&repository, &mut source);
    for (name, language, folders) in
        [("RUST", "rust", &["starter"][..]), ("PYTHON", "python", &["starter"][..]), ("CPP", "cpp", &["starter"][..])]
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
                let destination = relative.to_owned();
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

/* === Tests === */
