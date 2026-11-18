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

// Local Imports

use crate::{Result, error::Error, settings::Settings};

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
    pub(super) fn require_trust(&self) -> Result<()> {
        match self {
            Self::Add(args) => require_trust(args.trust),
            Self::Install(args) => require_trust(args.trust),
            _ => Ok(()),
        }
    }

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

pub(super) fn require_trust(trust: bool) -> Result<()> {
    if !trust {
        return Err(Error::Invalid("Installing Runs the Plugin's Code with the Host's Access; Review It and Pass --trust".into()));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use clap::Parser;
    use dokime_packaging::project::PluginProject;

    use crate::{cli::Cli, commands, error::Error, settings::Settings};

    #[tokio::test]
    async fn untrusted_install_and_add_refuse_before_io() {
        for arguments in [["dokime", "plugin", "install", "missing.dokime"], ["dokime", "plugin", "add", "invalid-url"]] {
            let cli = Cli::try_parse_from(arguments).unwrap();
            let settings = Settings {
                remote: Some("missing.dokime-remote".into()),
                ..Settings::default()
            };
            let result = commands::run(cli.command, None, settings).await;
            assert!(matches!(result, Err(Error::Invalid(_))));
        }
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
    async fn new_embeds_a_buildable_manifest_for_each_starter() -> crate::Result<()> {
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
            let project = PluginProject::read(&destination.join(folder))?;
            assert!(project.manifest.build.is_some());
            assert_eq!(project.manifest.launch.is_some(), language == "python");
            assert_eq!(project.manifest.executable.is_some(), language != "python");
        }
        Ok(())
    }
}
