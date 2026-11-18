/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Commands

/* === Modules === */

pub(super) mod add;
mod bundle;
mod inspect;
pub(super) mod install;
mod list;
pub(super) mod project;
mod remove;
mod secrets;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use tonic::Status;

// Local Imports

use crate::{Result, settings::Settings};
use dokime::Client;
use dokime::client::{PluginEntry, PluginsGetOptions, PluginsWatchOptions};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(clap::Subcommand)]
pub(crate) enum Command {
    Remove(remove::Args),
    Secrets(secrets::Args),
    List(list::Args),
    Bundle(bundle::Args),
    Inspect(inspect::Args),
    Add(add::Args),
    Install(install::Args),
}

/* === Traits === */

/* === Implementations === */

impl Command {
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Bundle(_) => "Bundle Plugin",
            Self::Inspect(_) => "Inspect Plugin",
            Self::Add(_) => "Add Plugin",
            Self::Install(_) => "Install Plugin",
            Self::List(_) => "List Plugins",
            Self::Remove(_) => "Remove Plugin",
            Self::Secrets(_) => "Manage Plugin Secrets",
        }
    }
}

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: Settings) -> Result<()> {
    match command {
        Command::Remove(args) => remove::run(args, endpoint, settings).await,
        Command::Secrets(args) => secrets::run(args, endpoint, settings).await,
        Command::List(args) => list::run(args, endpoint, settings).await,
        Command::Bundle(args) => bundle::run(args, endpoint, settings).await,
        Command::Inspect(args) => inspect::run(args, endpoint, settings).await,
        Command::Add(args) => add::run(args, endpoint, settings).await,
        Command::Install(args) => install::run(args, endpoint, settings).await,
    }
}

/* === Private Functions === */

async fn await_ready(client: &Client, entry: PluginEntry, settings: &Settings, action: &str) -> Result<PluginEntry> {
    let name = entry.name.clone();
    let mut watch = client
        .plugins
        .watch_with(PluginsWatchOptions {
            name: Some(name.clone()),
            context: settings.context()?,
            ..Default::default()
        })
        .await?;
    let mut initial = Some(entry);
    let result = dokime_runtime::await_plugin_ready(
        async || {
            if initial.is_some() {
                return Ok(initial.take());
            }
            let response = client
                .plugins
                .get_with(
                    name.clone(),
                    PluginsGetOptions {
                        context: settings.context()?,
                        ..Default::default()
                    },
                )
                .await?;
            let entry = response.entry.ok_or_else(|| format!("{action}: Plugin Entry Was Removed"))?;
            Ok(Some(entry))
        },
        async || Ok(watch.next().await?.is_some()),
        |entry| {
            Some(
                Status::failed_precondition(format!(
                    "{action} {}: {}",
                    entry.name,
                    entry.failure.map_or_else(|| "Activation Stopped".into(), |failure| failure.message)
                ))
                .into(),
            )
        },
        || format!("{action}: Plugin Watch Closed Before Readiness").into(),
    )
    .await;
    drop(watch);
    result
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::{self, File},
        path::Path,
    };

    use clap::Parser;
    use dokime_packaging::project::PluginProject;
    use dokime_runtime::process;
    use harmos_runtime::bundle::{Bundle, Identity, Limits};

    use crate::{Result, cli::Cli, commands, settings::Settings};

    #[test]
    fn development_command_replaces_plugin_dev() {
        assert!(Cli::try_parse_from(["dokime", "dev", "--plugin", "example.dokime"]).is_ok());
        assert!(Cli::try_parse_from(["dokime", "plugin", "dev", "--executable", "plugin"]).is_err());
        assert!(Cli::try_parse_from(["dokime", "dev"]).is_err());
    }

    #[test]
    fn bundle_requires_a_project_folder_and_rejects_removed_flags() {
        assert!(Cli::try_parse_from(["dokime", "plugin", "bundle", "project", "--out", "plugin.dokime"]).is_ok());
        assert!(Cli::try_parse_from(["dokime", "plugin", "bundle", "--executable", "plugin", "--out", "plugin.dokime"]).is_err());
        assert!(Cli::try_parse_from(["dokime", "plugin", "bundle", "project", "--asset", "asset", "--out", "plugin.dokime"]).is_err());
    }

    #[tokio::test]
    async fn created_starters_resolve_native_metadata_and_asset_inputs() -> Result<()> {
        let root = tempfile::tempdir()?;
        for (language, folder) in [("python", "test"), ("rust", "test"), ("csharp", "Test"), ("cpp", "test")] {
            let destination = root.path().join(language);
            let mut arguments =
                vec!["dokime".into(), "new".into(), destination.to_string_lossy().into_owned(), "--lang".into(), language.into()];
            if language == "cpp" {
                arguments.extend(["--sdk-url".into(), "fixture.tar.gz".into(), "--sdk-sha256".into(), "0".repeat(64)]);
            }
            let cli = Cli::try_parse_from(arguments).unwrap();
            commands::run(cli.command, None, Settings::default()).await?;
            let folders = if folder == "Test" { ["Plugin", "Test"] } else { ["plugin", "test"] };
            for folder in folders {
                let project = PluginProject::read(&destination.join(folder))?;
                project.validate_inputs()?;
                let metadata = project.resolve_metadata("")?;
                assert!(metadata.version.is_some());
                assert!(metadata.description.is_some());
                assert!(!metadata.authors.unwrap_or_default().is_empty());
                assert!(metadata.license.is_some());
                assert!(metadata.repository.is_some());
                let icon = metadata.icon.ok_or("Create Starter: Include Its Plugin Icon")?;
                assert!(project.path(&icon)?.is_file());
            }
        }
        Ok(())
    }

    #[test]
    fn launch_folder_inventory_excludes_external_runtime_images() -> dokime_packaging::Result<()> {
        // Prepare an External Runtime Above the Bundle's File Bound
        let parent = tempfile::tempdir()?;
        let runtime = File::create(parent.path().join("interpreter"))?;
        runtime.set_len(Limits::default().file_bytes as u64 + 1)?;
        let root = parent.path().join("plugin");
        fs::create_dir(&root)?;
        fs::write(root.join("dokime.toml"), "[plugin]\n[scripts]\nlaunch = ['interpreter', 'payload.bin']\n")?;
        fs::write(root.join("payload.bin"), [2, 3, 5, 7])?;

        // Bundle Only the Plugin's Sources
        let project = PluginProject::read(&root)?;
        let identity = Identity {
            name: "launch-fixture".into(),
            version: "1.0.0".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = dokime_packaging::bundle::create_project(&project, identity, None)?;
        let bundle = Bundle::inspect(&bytes, Limits::default())?;
        let inventory = bundle.manifest().files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>();
        assert_eq!(inventory, ["dokime.json", "dokime.toml", "payload.bin"]);
        assert_eq!(bundle.manifest().launch, project.manifest.scripts.launch);
        assert_eq!(bundle.file("payload.bin"), Some([2, 3, 5, 7].as_slice()));
        Ok(())
    }

    #[test]
    fn launch_folder_round_trip_runs_the_native_interpreter_in_its_root() -> dokime_packaging::Result<()> {
        // The Reexecuted Native Test Program Interprets the Packaged Payload
        if env::var_os("DOKIME_LAUNCH_FIXTURE").is_some() {
            assert_eq!(fs::read("payload.bin")?, [2, 3, 5, 7]);
            assert!(Path::new("dokime.toml").is_file());
            assert!(!Path::new("ignored.bin").exists());
            return Ok(());
        }

        // Keep the Native Interpreter Outside the Shipped Project
        let root = tempfile::tempdir()?;
        let interpreter = env::current_exe()?;
        let program = interpreter.file_name().unwrap().to_str().unwrap();
        let test = "commands::plugin::tests::launch_folder_round_trip_runs_the_native_interpreter_in_its_root";
        fs::write(
            root.path().join("dokime.toml"),
            format!("[plugin]\n[scripts]\nlaunch = ['{program}', '--exact', '{test}', '--nocapture']\n"),
        )?;
        fs::write(root.path().join("payload.bin"), [2, 3, 5, 7])?;
        fs::write(root.path().join(".gitignore"), "ignored.bin\n")?;
        fs::write(root.path().join("ignored.bin"), [11])?;
        fs::create_dir(root.path().join(".git"))?;
        fs::write(root.path().join(".git/config"), b"private repository data")?;
        let project = PluginProject::read(root.path())?;
        let identity = Identity {
            name: "launch-fixture".into(),
            version: "1.0.0".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = dokime_packaging::bundle::create_project(&project, identity, Some(b"description"))?;
        let bundle = Bundle::inspect(&bytes, Limits::default())?;
        assert_eq!(bundle.manifest().launch, project.manifest.scripts.launch);
        assert!(bundle.manifest().files.iter().all(|file| file.executable.is_none()));
        let inventory = bundle.manifest().files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>();
        assert_eq!(inventory, [".gitignore", "description.pb", "dokime.json", "dokime.toml", "payload.bin"]);
        assert!(bundle.file("ignored.bin").is_none());
        assert!(bundle.file(".git/config").is_none());
        assert_eq!(bundle.file("description.pb"), Some(b"description".as_slice()));

        // Extract and Run the Declared Command Exactly from the Bundle Root
        let parent = tempfile::tempdir()?;
        let extracted = bundle.extract(parent.path())?;
        let launch = bundle.manifest().launch.as_ref().unwrap();
        let mut search = vec![interpreter.parent().unwrap().to_path_buf()];
        search.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
        let search = env::join_paths(search)?;
        let output = process::command(&launch[0])
            .args(&launch[1..])
            .current_dir(extracted.path())
            .env("PATH", search)
            .env("DOKIME_LAUNCH_FIXTURE", "1")
            .output()?;
        assert!(
            output.status.success(),
            "stdout: {}; stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}
