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

// Standard Library Imports
use std::{fs, path::PathBuf, time::Duration};

// Third Party Imports
use reqwest::Client;
use serde_json::Value;

// Local Imports
use crate::settings::Settings;

/* === Constants === */

include!(concat!(env!("OUT_DIR"), "/examples.rs"));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    path: PathBuf,

    #[arg(long, value_enum)]
    lang: Language,

    #[arg(long, requires = "sdk_sha256", help = "C++ Installed SDK Archive URL or Local Path")]
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

pub(crate) async fn run(args: Args, settings: Settings) -> crate::Result<()> {
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
                Client::builder()
                    .timeout(Duration::from_secs(settings.read_timeout_s))
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
        presets = Some((url, digest));
    }
    fs::create_dir(&args.path)?;
    let result = (|| -> crate::Result<()> {
        for (relative, bytes) in files {
            let destination = args.path.join(relative);
            fs::create_dir_all(destination.parent().unwrap())?;
            if relative.ends_with("/CMakePresets.json") {
                let (url, digest) = presets.as_ref().ok_or("Create C++ Project: Missing SDK Coordinates")?;
                let mut value: Value = serde_json::from_slice(bytes)?;
                value["configurePresets"][0]["cacheVariables"]["DOKIME_SDK_URL"] = url.clone().into();
                value["configurePresets"][0]["cacheVariables"]["DOKIME_SDK_SHA256"] = digest.clone().into();
                fs::write(destination, serde_json::to_vec_pretty(&value)?)?;
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

#[cfg(test)]
mod tests {
    use std::{error::Error, fs};

    use super::{Args, Language, run};
    use crate::settings::Settings;

    #[tokio::test]
    async fn python_starters_leave_dependency_locking_to_the_user() -> Result<(), Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        let project = directory.path().join("python");
        let args = Args {
            path: project.clone(),
            lang: Language::Python,
            sdk_url: None,
            sdk_sha256: None,
        };
        run(args, Settings::default()).await?;

        for folder in ["plugin", "test"] {
            let root = project.join(folder);
            assert!(root.join("pyproject.toml").is_file());
            let mut pending = vec![root];
            while let Some(directory) = pending.pop() {
                for entry in fs::read_dir(directory)? {
                    let entry = entry?;
                    assert_ne!(entry.file_name(), "uv.lock", "{}", entry.path().display());
                    if entry.file_type()?.is_dir() {
                        pending.push(entry.path());
                    }
                }
            }
        }
        Ok(())
    }
}
