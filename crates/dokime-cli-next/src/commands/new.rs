/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Create an SDK Starter

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use std::{fs, path::PathBuf};

/* === Constants === */

const RUST: &[(&str, &[u8])] = &[
    (".gitattributes", include_bytes!("../../../../sdk/rust/starter/.gitattributes")),
    ("Cargo.toml", include_bytes!("../../../../sdk/rust/starter/Cargo.toml")),
    ("src/main.rs", include_bytes!("../../../../sdk/rust/starter/src/main.rs")),
    ("mise.toml", include_bytes!("../../../../sdk/rust/starter/mise.toml")),
    (".gitignore", include_bytes!("../../../../sdk/rust/starter/.gitignore")),
];
const PYTHON: &[(&str, &[u8])] = &[
    (".gitattributes", include_bytes!("../../../../sdk/python/examples/starter/.gitattributes")),
    (".gitignore", include_bytes!("../../../../sdk/python/examples/starter/.gitignore")),
    ("main.py", include_bytes!("../../../../sdk/python/examples/starter/main.py")),
    ("suite.py", include_bytes!("../../../../sdk/python/examples/starter/suite.py")),
    ("pyproject.toml", include_bytes!("../../../../sdk/python/examples/starter/pyproject.toml")),
    ("mise.toml", include_bytes!("../../../../sdk/python/examples/starter/mise.toml")),
    ("tests/host.py", include_bytes!("../../../../sdk/python/examples/starter/tests/host.py")),
    ("tests/test_capture.py", include_bytes!("../../../../sdk/python/examples/starter/tests/test_capture.py")),
];
const CSHARP: &[(&str, &[u8])] = &[
    ("Starter.csproj", include_bytes!("../../../../sdk/csharp/starter/Starter.csproj")),
    ("Program.cs", include_bytes!("../../../../sdk/csharp/starter/Program.cs")),
    ("mise.toml", include_bytes!("../../../../sdk/csharp/starter/mise.toml")),
    (".gitignore", include_bytes!("../../../../sdk/csharp/starter/.gitignore")),
    ("packages.lock.json", include_bytes!("../../../../sdk/csharp/starter/packages.lock.json")),
];
const CPP: &[(&str, &[u8])] = &[
    ("CMakeLists.txt", include_bytes!("../../../../sdk/cpp/starter/CMakeLists.txt")),
    ("CMakePresets.json", include_bytes!("../../../../sdk/cpp/starter/CMakePresets.json")),
    ("main.cpp", include_bytes!("../../../../sdk/cpp/starter/main.cpp")),
    ("mise.toml", include_bytes!("../../../../sdk/cpp/starter/mise.toml")),
    ("vcpkg.json", include_bytes!("../../../../sdk/cpp/starter/vcpkg.json")),
    (".gitignore", include_bytes!("../../../../sdk/cpp/starter/.gitignore")),
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    path: PathBuf,

    #[arg(long, value_enum)]
    lang: Language,

    #[arg(long, requires = "sdk_sha256", help = "C++ SDK Source Archive URL")]
    sdk_url: Option<String>,

    #[arg(long, requires = "sdk_url", help = "C++ SDK Source Archive SHA-256")]
    sdk_sha256: Option<String>,
}

/* === Enums === */

#[derive(Clone, Copy, clap::ValueEnum)]
enum Language {
    Rust,
    Python,
    Csharp,
    Cpp,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let name = args.path.file_name().and_then(|value| value.to_str()).ok_or("Create Project: Supply a UTF-8 Directory Name")?;
    let name = serde_json::to_string(name)?;
    let files = match args.lang {
        Language::Rust => RUST,
        Language::Python => PYTHON,
        Language::Csharp => CSHARP,
        Language::Cpp => CPP,
    };
    let mut presets = None;
    if matches!(args.lang, Language::Cpp) {
        let url = args.sdk_url.unwrap_or_else(|| {
            format!(
                "https://github.com/stokker-technologies/dokime/releases/download/v{0}/dokime-cpp-{0}.tar.gz",
                env!("CARGO_PKG_VERSION")
            )
        });
        let digest = match args.sdk_sha256 {
            Some(digest) => digest,
            None => {
                let _ = rustls::crypto::ring::default_provider().install_default();
                reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()?
                    .get(format!("{url}.sha256"))
                    .send()
                    .await?
                    .error_for_status()?
                    .text()
                    .await?
                    .split_whitespace()
                    .next()
                    .ok_or("Create C++ Project: Published Checksum Is Empty")?
                    .into()
            }
        };
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("Create C++ Project: Supply the Published 64-Digit SHA-256".into());
        }
        let source = CPP.iter().find(|(name, _)| *name == "CMakePresets.json").unwrap().1;
        let mut value: serde_json::Value = serde_json::from_slice(source)?;
        value["configurePresets"][0]["cacheVariables"]["DOKIME_SDK_URL"] = url.into();
        value["configurePresets"][0]["cacheVariables"]["DOKIME_SDK_SHA256"] = digest.into();
        presets = Some(serde_json::to_vec_pretty(&value)?);
    }
    fs::create_dir(&args.path)?;
    let result = (|| -> crate::Result<()> {
        for (relative, bytes) in files {
            let destination = args.path.join(relative);
            fs::create_dir_all(destination.parent().unwrap())?;
            if *relative == "pyproject.toml" || *relative == "Cargo.toml" {
                let source = std::str::from_utf8(bytes)?;
                fs::write(destination, source.replacen("\"dokime-starter\"", &name, 1))?;
            } else if *relative == "CMakePresets.json" {
                fs::write(destination, presets.as_ref().unwrap())?;
            } else {
                fs::write(destination, bytes)?;
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        fs::remove_dir_all(&args.path)?;
        return Err(error);
    }
    println!("Created {}", args.path.display());
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
