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

mod bundle;
mod inspect;
pub(super) mod install;
mod list;
pub(super) mod project;
pub(super) mod register;
mod remove;
mod secrets;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, settings::Settings};
use dokime_client_internal::{Client, PluginsGetOptions, PluginsWatchOptions, host::PluginEntry};

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

    /// Register a Remote gRPC Plugin by Address.
    Register(register::Args),

    /// Install a Managed Plugin Bundle.
    Install(install::Args),
}

/* === Traits === */

/* === Implementations === */

impl Command {
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Bundle(_) => "Bundle Plugin",
            Self::Inspect(_) => "Inspect Plugin",
            Self::Register(_) => "Register Plugin",
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
        Command::Register(args) => register::run(args, endpoint, settings).await,
        Command::Install(args) => install::run(args, endpoint, settings).await,
    }
}

/* === Private Functions === */

async fn await_ready(client: &Client, entry: PluginEntry, settings: &Settings, action: &str) -> Result<PluginEntry> {
    let name = entry.name.clone();
    let watch = client
        .plugins
        .watch_with(PluginsWatchOptions {
            name: Some(name.clone()),
            context: settings.context()?,
            ..Default::default()
        })
        .await?;
    let mut readiness = (Some(entry), client, name, settings, action, watch);
    let result = dokime_runtime::await_plugin_ready(
        &mut readiness,
        |(initial, client, name, settings, action, _)| {
            Box::pin(async move {
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
            })
        },
        |(_, _, _, _, _, watch)| Box::pin(async move { Ok(watch.next().await?.is_some()) }),
        |entry| {
            let summary = list::summary(&entry);
            let Some(mut failure) = entry.failure else {
                return Some(format!("{action} {summary}: Restart the Plugin and Check Its Status").into());
            };
            failure.message = format!("{action} {summary}: {}", failure.message);
            Some(dokime_runtime::Error::Plugin(Box::new(failure)).into())
        },
        || format!("{action}: Plugin Watch Closed Before Readiness").into(),
    )
    .await;
    drop(readiness);
    result
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        path::Path,
    };

    use clap::Parser;
    use dokime_packaging::project::PluginProject;
    use harmos_runtime::bundle::{Bundle, Identity, Limits};

    use crate::{Result, cli::Cli, commands, settings::Settings};

    #[test]
    fn remote_plugins_register_and_bundles_install() {
        let cli = Cli::try_parse_from(["dokime", "plugin", "register", "http://127.0.0.1:7001", "--json"]).unwrap();
        assert!(matches!(cli.command, Some(commands::Command::Plugin(super::Command::Register(_)))));
        let cli = Cli::try_parse_from(["dokime", "plugin", "install", "plugin.dokime"]).unwrap();
        assert!(matches!(cli.command, Some(commands::Command::Plugin(super::Command::Install(_)))));
        assert!(Cli::try_parse_from(["dokime", "plugin", "add", "http://127.0.0.1:7001"]).is_err());
    }

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
        for language in ["python", "rust", "csharp", "cpp"] {
            let destination = root.path().join(language);
            let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
            let arguments = vec![
                "dokime".into(),
                "new".into(),
                destination.to_string_lossy().into_owned(),
                "--lang".into(),
                language.into(),
                "--from".into(),
                repository.to_string_lossy().into_owned(),
            ];
            let cli = Cli::try_parse_from(arguments).unwrap();
            commands::run(cli.command.unwrap(), None, Settings::default()).await?;
            let project = PluginProject::read(&destination)?;
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
        Ok(())
    }

    #[test]
    fn launch_folder_round_trip_runs_the_native_interpreter_in_its_root() -> dokime_packaging::Result<()> {
        // Bundle a Script With an External Interpreter.
        let parent = tempfile::tempdir()?;
        let root = parent.path().join("plugin");
        fs::create_dir(&root)?;
        fs::write(
            root.join("dokime.toml"),
            r#"[plugin]
[scripts]
launch = ["python", "-c", "from pathlib import Path; print(Path('payload.bin').read_bytes().hex())"]
"#,
        )?;
        fs::write(root.join("payload.bin"), [2, 3, 5, 7])?;
        let project = PluginProject::read(&root)?;
        let identity = Identity {
            name: "launch-fixture".into(),
            version: "1.0.0".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = dokime_packaging::bundle::create_project(&project, identity, None)?;
        let bundle = Bundle::inspect(&bytes, Limits::default())?;

        // Launch the Extracted Project in Its Own Working Directory.
        let extracted = bundle.extract(parent.path())?;
        let project = PluginProject::read(extracted.path())?;
        let launch = project.manifest.scripts.launch.as_deref().ok_or("Run Plugin: Retain Its Launch Command")?;
        let mut command = dokime_runtime::PluginProcess::command(&project, launch)?;
        let output = command.output()?;
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8(output.stdout)?.trim(), "02030507");
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
        assert_eq!(bundle.manifest().launch, project.manifest.scripts.launch);
        assert_eq!(bundle.file("payload.bin"), Some([2, 3, 5, 7].as_slice()));
        Ok(())
    }
}
