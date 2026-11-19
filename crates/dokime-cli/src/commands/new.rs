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
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

// Third Party Imports
use reqwest::Client;
use serde_json::Value;

// Local Imports
use crate::{Result, settings::Settings};

/* === Constants === */

include!(concat!(env!("OUT_DIR"), "/examples.rs"));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
#[command(about = "Create an SDK Project")]
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

pub(crate) async fn run(args: Args, settings: Settings) -> Result<()> {
    if matches!(args.lang, Language::Csharp) {
        return csharp_project(&args.path);
    }
    let files = match args.lang {
        Language::Rust => RUST,
        Language::Python => PYTHON,
        Language::Csharp => CSHARP,
        Language::Cpp => CPP,
    };
    let python = if matches!(args.lang, Language::Python) { Some(python_names(&args.path)?) } else { None };
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
    let result = (|| -> Result<()> {
        for (relative, bytes) in files {
            let relative = match &python {
                Some((_, package)) => relative.replace("starter_project", package),
                None => (*relative).to_owned(),
            };
            let destination = args.path.join(&relative);
            fs::create_dir_all(destination.parent().unwrap())?;
            if relative.ends_with("/CMakePresets.json") {
                let (url, digest) = presets.as_ref().ok_or("Create C++ Project: Missing SDK Coordinates")?;
                let mut value: Value = serde_json::from_slice(bytes)?;
                value["configurePresets"][0]["cacheVariables"]["DOKIME_SDK_URL"] = url.clone().into();
                value["configurePresets"][0]["cacheVariables"]["DOKIME_SDK_SHA256"] = digest.clone().into();
                fs::write(destination, serde_json::to_vec_pretty(&value)?)?;
            } else if let Some((name, package)) = &python {
                match str::from_utf8(bytes) {
                    Ok(text) => fs::write(
                        destination,
                        text.replace("starter_project", package)
                            .replace("starter-project", name)
                            .replace("Starter Project", &python_title(name)),
                    )?,
                    Err(_) => fs::write(destination, bytes)?,
                }
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

fn csharp_project(directory: &Path) -> Result<()> {
    let name = directory.file_name().and_then(|value| value.to_str()).ok_or("Create C# Project: Choose a UTF-8 Folder Name")?;
    let namespace = name
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut characters = word.chars();
            let first = characters.next().expect("A project word is nonempty").to_ascii_uppercase();
            format!("{first}{}", characters.as_str())
        })
        .collect::<String>();
    if !namespace.starts_with(|character: char| character.is_ascii_alphabetic()) || namespace == "Dokime" || namespace == "System" {
        return Err("Create C# Project: Choose a Name Beginning with a Letter That Does Not Shadow Dokime or System".into());
    }
    fs::create_dir(directory)?;
    let result = (|| -> Result<()> {
        for (relative, bytes) in CSHARP {
            let relative = relative.replace("Example", &namespace);
            let destination = directory.join(relative);
            fs::create_dir_all(destination.parent().expect("A starter file has a project directory"))?;
            match str::from_utf8(bytes) {
                Ok(text) => fs::write(destination, text.replace("Example", &namespace).replace("Starter Project", &python_title(name)))?,
                Err(_) => fs::write(destination, bytes)?,
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        fs::remove_dir_all(directory)?;
        return Err(error);
    }
    println!("Created {}", directory.display());
    Ok(())
}

fn python_title(name: &str) -> String {
    let words = name.split(|character: char| !character.is_alphanumeric()).filter(|word| !word.is_empty()).collect::<Vec<_>>();
    words
        .iter()
        .enumerate()
        .map(|(index, word)| {
            let word = word.to_lowercase();
            if index != 0
                && index + 1 != words.len()
                && matches!(
                    word.as_str(),
                    "a" | "an"
                        | "the"
                        | "and"
                        | "but"
                        | "or"
                        | "nor"
                        | "for"
                        | "so"
                        | "yet"
                        | "as"
                        | "at"
                        | "by"
                        | "in"
                        | "of"
                        | "on"
                        | "per"
                        | "to"
                        | "up"
                        | "via"
                )
            {
                return word;
            }
            let mut characters = word.chars();
            match characters.next() {
                Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
                None => word,
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn python_names(path: &Path) -> Result<(String, String)> {
    let name = path.file_name().and_then(|name| name.to_str()).ok_or("Create Python Project: Choose a UTF-8 Project Folder Name")?;
    let package = name
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '_' })
        .collect::<String>();
    if !package.bytes().any(|byte| byte.is_ascii_alphanumeric()) {
        return Err("Create Python Project: Choose a Name With at Least One ASCII Letter".into());
    }
    if package.starts_with(|character: char| character.is_ascii_digit()) {
        return Err("Create Python Project: Start the Name With a Letter or Underscore, Before Any Digits".into());
    }
    if matches!(
        package.as_str(),
        "and"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "del"
            | "elif"
            | "else"
            | "except"
            | "finally"
            | "for"
            | "from"
            | "global"
            | "if"
            | "import"
            | "in"
            | "is"
            | "lambda"
            | "nonlocal"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "try"
            | "while"
            | "with"
            | "yield"
    ) {
        return Err(format!("Create Python Project: Choose a Name That Does Not Normalize to the Python Keyword {package}").into());
    }
    if package == "dokime_sdk" {
        return Err("Create Python Project: Choose a Name Other Than dokime_sdk to Avoid Shadowing the SDK".into());
    }
    Ok((name.to_owned(), package))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::{error::Error, fs, path::Path};

    use super::{Args, CPP, CSHARP, Language, PYTHON, RUST, python_names, python_title, run};
    use crate::settings::Settings;

    #[test]
    fn every_buildable_starter_embeds_project_configuration() {
        for (files, folders) in [(RUST, &["plugin", "test"][..]), (PYTHON, &[""][..]), (CSHARP, &[""][..]), (CPP, &["plugin", "test"][..])]
        {
            for folder in folders {
                for name in ["dokime.toml", "mise.toml", ".gitignore", ".gitattributes"] {
                    let relative = if folder.is_empty() { name.to_owned() } else { format!("{folder}/{name}") };
                    assert!(files.iter().any(|(path, _)| *path == relative), "{relative}");
                }
            }
        }
    }

    #[test]
    fn python_display_titles_follow_project_names() {
        assert_eq!(python_title("my-project"), "My Project");
        assert_eq!(python_title("number-of-rows"), "Number of Rows");
        assert_eq!(python_title("the-test-and-the-result"), "The Test and the Result");
        assert_eq!(python_title("read_it_in"), "Read It In");
    }

    #[test]
    fn python_project_names_are_safe_import_packages() {
        assert_eq!(python_names(Path::new("My-Project")).unwrap(), ("My-Project".into(), "my_project".into()));
        for name in ["---", "123", "1-name", "class", "await", "dokime-sdk", "DOKIME_SDK"] {
            assert!(python_names(Path::new(name)).is_err(), "{name}");
        }
    }

    #[tokio::test]
    async fn python_substitutes_package_paths_and_project_text() -> Result<(), Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        let project = directory.path().join("My-Project");
        run(
            Args {
                path: project.clone(),
                lang: Language::Python,
                sdk_url: None,
                sdk_sha256: None,
            },
            Settings::default(),
        )
        .await?;
        let manifest = fs::read_to_string(project.join("pyproject.toml"))?;
        assert!(manifest.contains("name = \"My-Project\""));
        assert!(manifest.contains("module-name = \"my_project\""));
        let launch = fs::read_to_string(project.join("dokime.toml"))?;
        assert!(launch.contains("my_project.plugin"));
        assert!(launch.contains("label = \"My Project\""));
        let plugin = fs::read_to_string(project.join("src/my_project/plugin/__init__.py"))?;
        assert!(plugin.contains("from my_project.plugin."));
        Ok(())
    }

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

        assert!(project.join("pyproject.toml").is_file());
        assert!(project.join("src/python/plugin/__main__.py").is_file());
        assert!(!project.join("plugin").exists());
        assert!(!project.join("test").exists());
        let mut pending = vec![project];
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(directory)? {
                let entry = entry?;
                assert_ne!(entry.file_name(), "uv.lock", "{}", entry.path().display());
                if entry.file_type()?.is_dir() {
                    pending.push(entry.path());
                }
            }
        }
        Ok(())
    }
}
