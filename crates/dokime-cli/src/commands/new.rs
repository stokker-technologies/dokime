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
    env,
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

    #[arg(long, help = "Use Starter Sources and the Local SDK From This Checkout (DOKIME_NEW_FROM)")]
    from: Option<PathBuf>,

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
    let checkout = source_checkout(args.from, env::var_os("DOKIME_NEW_FROM").map(PathBuf::from))?;
    let embedded = match args.lang {
        Language::Rust => RUST,
        Language::Python => PYTHON,
        Language::Csharp => CSHARP,
        Language::Cpp => CPP,
    };
    let files = starter_files(args.lang, embedded, checkout.as_deref())?;
    if matches!(args.lang, Language::Csharp) {
        return csharp_project(&args.path, files, checkout.as_deref());
    }
    let python = if matches!(args.lang, Language::Python | Language::Rust) { Some(python_names(&args.path)?) } else { None };
    let mut presets = None;
    if matches!(args.lang, Language::Cpp) && checkout.is_none() {
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
        for (relative, bytes) in &files {
            let relative = match &python {
                Some((_, package)) => relative.replace("starter_project", package),
                None => relative.to_owned(),
            };
            let destination = args.path.join(&relative);
            fs::create_dir_all(destination.parent().unwrap())?;
            if relative.ends_with("/CMakePresets.json") && checkout.is_none() {
                let (url, digest) = presets.as_ref().ok_or("Create C++ Project: Missing SDK Coordinates")?;
                let mut value: Value = serde_json::from_slice(bytes)?;
                value["configurePresets"][0]["cacheVariables"]["DOKIME_SDK_URL"] = url.clone().into();
                value["configurePresets"][0]["cacheVariables"]["DOKIME_SDK_SHA256"] = digest.clone().into();
                fs::write(destination, serde_json::to_vec_pretty(&value)?)?;
            } else if let Some((name, package)) = &python {
                match str::from_utf8(bytes) {
                    Ok(text) => fs::write(
                        destination,
                        local_dependency(args.lang, &relative, text.replace("starter_project", package)
                            .replace("starter-project", name)
                            .replace("Starter Project", &python_title(name)), checkout.as_deref())?,
                    )?,
                    Err(_) => fs::write(destination, bytes)?,
                }
            } else {
                let bytes = match str::from_utf8(bytes) {
                    Ok(text) => local_dependency(args.lang, &relative, text.into(), checkout.as_deref())?.into_bytes(),
                    Err(_) => bytes.clone(),
                };
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

fn source_checkout(argument: Option<PathBuf>, environment: Option<PathBuf>) -> Result<Option<PathBuf>> {
    argument.or(environment).map(|path| {
        path.canonicalize().map_err(|error| format!("Create Project: Choose a Readable SDK Checkout {}: {error}", path.display()).into())
    }).transpose()
}

fn starter_files(language: Language, embedded: &[(&str, &[u8])], checkout: Option<&Path>) -> Result<Vec<(String, Vec<u8>)>> {
    let Some(checkout) = checkout else {
        return Ok(embedded.iter().map(|(path, bytes)| ((*path).into(), bytes.to_vec())).collect());
    };
    let folder = match language {
        Language::Rust => "sdk/rust/examples/starter",
        Language::Python => "sdk/python/examples/starter",
        Language::Csharp => "sdk/csharp/examples/Starter",
        Language::Cpp => return cpp_starter_files(checkout),
    };
    read_starter(&checkout.join(folder))
}

// The C++ Owner Replaces This Two-Folder Selection When Its Single Starter Lands.
fn cpp_starter_files(checkout: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut files = Vec::new();
    for folder in ["plugin", "test"] {
        for (relative, bytes) in read_starter(&checkout.join("sdk/cpp/examples").join(folder))? {
            files.push((format!("{folder}/{relative}"), bytes));
        }
    }
    Ok(files)
}

fn read_starter(root: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    if !root.is_dir() {
        return Err(format!("Create Project: Starter Is Missing at {}; Choose a Checkout Containing That Starter", root.display()).into());
    }
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if matches!(name.as_ref(), "target" | "build" | "bin" | "obj" | ".git" | ".venv" | "__pycache__" | "_input" | "_output" | "credentials")
                || name == ".env" || (name.starts_with(".env.") && name != ".env.example")
                || matches!(name.as_ref(), "uv.lock" | "Cargo.lock" | "packages.lock.json")
                || [".key", ".pem", ".pair", ".dokime", ".pyc"].iter().any(|suffix| name.ends_with(suffix)) {
                continue;
            }
            let path = entry.path();
            let relative = path.strip_prefix(root).expect("Starter File Remains Below Its Root").to_string_lossy().replace('\\', "/");
            if relative.starts_with("assets/plugin/") { continue; }
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                return Err(format!("Create Project: Replace Starter Symlink {} With a Regular File", path.display()).into());
            }
            if kind.is_dir() { pending.push(path); }
            else if kind.is_file() { files.push((relative, fs::read(path)?)); }
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(files)
}

fn local_dependency(language: Language, relative: &str, text: String, checkout: Option<&Path>) -> Result<String> {
    let Some(checkout) = checkout else { return Ok(text); };
    let directory = checkout.join("sdk").join(match language { Language::Rust => "rust", Language::Python => "python", Language::Csharp => "csharp", Language::Cpp => "cpp" });
    let path = directory.to_string_lossy().replace('\\', "/");
    let literal = serde_json::to_string(&path)?;
    match language {
        Language::Python if relative == "pyproject.toml" => Ok(format!("{text}\n[tool.uv.sources]\ndokime-sdk = {{ path = {literal} }}\n")),
        Language::Rust if relative == "Cargo.toml" => {
            let mut manifest: toml::Value = toml::from_str(&text)?;
            let dependencies = manifest.get_mut("dependencies").and_then(toml::Value::as_table_mut).ok_or("Create Rust Project: Declare dependencies.dokime in the Starter Cargo.toml")?;
            dependencies.insert("dokime".into(), toml::Value::Table(toml::map::Map::from_iter([("path".into(), toml::Value::String(path))])));
            Ok(toml::to_string_pretty(&manifest).map_err(|error| format!("Create Rust Project: Write the Local SDK Dependency: {error}"))?)
        }
        Language::Csharp if relative.ends_with(".csproj") => {
            let local = format!("{path}/src/Dokime/Dokime.csproj").replace('&', "&amp;").replace('"', "&quot;");
            Ok(text.lines().map(|line| {
                if line.contains("<PackageReference Include=\"Dokime\"") { format!("    <ProjectReference Include=\"{local}\" />\n") }
                else { format!("{line}\n") }
            }).collect())
        }
        Language::Cpp if relative.ends_with("CMakeLists.txt") => {
            let start = text.find("if(DOKIME_SDK_URL)").ok_or("Create C++ Project: Declare the SDK Source Selection in CMakeLists.txt")?;
            let end = text.find("find_package(Dokime CONFIG REQUIRED)").ok_or("Create C++ Project: Declare the Dokime CMake Dependency")? + "find_package(Dokime CONFIG REQUIRED)".len();
            Ok(format!("{}set(DOKIME_BUILD_TESTS OFF CACHE BOOL \"\" FORCE)\nadd_subdirectory({literal} \"${{CMAKE_BINARY_DIR}}/dokime-sdk\"){}", &text[..start], &text[end..]))
        }
        _ => Ok(text),
    }
}

fn csharp_project(directory: &Path, files: Vec<(String, Vec<u8>)>, checkout: Option<&Path>) -> Result<()> {
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
        for (relative, bytes) in &files {
            let relative = relative.replace("Example", &namespace);
            let destination = directory.join(&relative);
            fs::create_dir_all(destination.parent().expect("A starter file has a project directory"))?;
            match str::from_utf8(bytes) {
                Ok(text) => fs::write(destination, local_dependency(Language::Csharp, &relative, text.replace("Example", &namespace).replace("Starter Project", &python_title(name)), checkout)?)?,
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
    fn checkout_argument_wins_over_environment() -> Result<(), Box<dyn Error>> {
        let argument = tempfile::tempdir()?;
        let environment = tempfile::tempdir()?;
        assert_eq!(super::source_checkout(Some(argument.path().into()), Some(environment.path().into()))?, Some(argument.path().canonicalize()?));
        assert_eq!(super::source_checkout(None, Some(environment.path().into()))?, Some(environment.path().canonicalize()?));
        assert_eq!(super::source_checkout(None, None)?, None);
        Ok(())
    }

    #[test]
    fn checkout_writes_native_local_sdk_dependencies() -> Result<(), Box<dyn Error>> {
        let checkout = tempfile::tempdir()?;
        for (language, relative, input, expected) in [
            (Language::Python, "pyproject.toml", "[project]\nname = 'example'\n", "dokime-sdk = { path = "),
            (Language::Rust, "Cargo.toml", "[dependencies]\ndokime = '0.1.5'\n", "[dependencies.dokime]"),
            (Language::Csharp, "src/Example/Example.csproj", "<Project>\n<PackageReference Include=\"Dokime\" Version=\"0.1.0-dev\" />\n</Project>", "<ProjectReference Include="),
            (Language::Cpp, "plugin/CMakeLists.txt", "if(DOKIME_SDK_URL)\nendif()\nfind_package(Dokime CONFIG REQUIRED)", "add_subdirectory("),
        ] {
            let text = super::local_dependency(language, relative, input.into(), Some(checkout.path()))?;
            assert!(text.contains(expected), "{text}");
            assert!(text.contains(&checkout.path().to_string_lossy().replace('\\', "/")), "{text}");
            assert!(!text.contains("PackageReference"));
            assert!(!text.contains("FetchContent"));
        }
        Ok(())
    }

    #[test]
    fn missing_checkout_starter_names_the_required_path() -> Result<(), Box<dyn Error>> {
        let checkout = tempfile::tempdir()?;
        for (language, folder) in [(Language::Rust, "rust/examples/starter"), (Language::Python, "python/examples/starter"), (Language::Csharp, "csharp/examples/Starter"), (Language::Cpp, "cpp/examples/plugin")] {
            let missing = checkout.path().join("sdk").join(folder);
            let error = super::starter_files(language, &[], Some(checkout.path())).unwrap_err().to_string();
            assert!(error.contains(&missing.display().to_string()), "{error}");
        }
        Ok(())
    }

    #[test]
    fn every_buildable_starter_embeds_project_configuration() {
        for (files, folders) in [(RUST, &[""][..]), (PYTHON, &[""][..]), (CSHARP, &[""][..]), (CPP, &["plugin", "test"][..])]
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
                from: None,
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
            from: None,
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
