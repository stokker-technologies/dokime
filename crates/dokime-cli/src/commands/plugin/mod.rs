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
mod remove;
mod secrets;

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
    pub(super) fn require_trust(&self) -> crate::Result<()> {
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

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
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

pub(super) fn require_trust(trust: bool) -> crate::Result<()> {
    if !trust {
        return Err(crate::error::Error::Invalid(
            "Installing Runs the Plugin's Code with the Host's Access; Review It and Pass --trust".into(),
        ));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::{cli::Cli, commands, error::Error, settings::Settings};

    #[tokio::test]
    async fn untrusted_install_and_add_refuse_before_io() {
        for arguments in [["dokime", "plugin", "install", "missing.dokime"], ["dokime", "plugin", "add", "invalid-url"]] {
            let cli = Cli::try_parse_from(arguments).unwrap();
            let settings = Settings { remote: Some("missing.dokime-remote".into()), ..Settings::default() };
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
}
