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
use std::{net::IpAddr, path::PathBuf};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, clap::Args)]
#[command(about = "Listener flags shared by the standalone runtime and the Cargo wrapper.")]
#[group(id = "listener")]
pub struct NetworkArgs {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: IpAddr,

    #[arg(long, help = "Listen On This Exact Port; Defaults To Local Configuration. Zero Allocates An Available Port.")]
    pub port: Option<u16>,

    #[arg(long, help = "Override The Configured Message Limit In MiB.")]
    pub message_limit_mib: Option<usize>,
}

#[derive(Clone, Debug, clap::Args)]
#[command(about = "Server process flags shared by the standalone runtime and Cargo wrapper.")]
#[group(id = "server-runtime-arguments")]
pub struct Args {
    #[arg(long, help = "Shut down when the controlling process closes standard input.")]
    pub shutdown_stdin: bool,

    #[arg(long, hide = true, requires = "shutdown_stdin", help = "Private development supervisor protocol.")]
    pub dev_control: bool,

    #[command(flatten)]
    pub network: NetworkArgs,

    #[arg(long, help = "Hub title; defaults to the resolved brand's label.")]
    pub title: Option<String>,

    #[arg(long)]
    pub label: Option<String>,

    #[arg(long, help = "Execution data directory for this process. Defaults to `data/` beneath the application root.")]
    pub output_root: Option<PathBuf>,

    #[arg(
        long,
        help = "Keep settings, installed plugins, and default data beneath this root. Defaults to `DOKIME_APPLICATION_ROOT` or the distribution's per-user directory. Explicit roots keep logs under `<root>/logs`; Unix defaults use XDG state home."
    )]
    pub application_root: Option<PathBuf>,

    #[arg(
        long,
        help = "Load environment variables from exactly this file with dotenv, instead of searching upward from the working directory for `.env`. Refused outright when the file does not exist — never a silent skip. Either way, a variable already exported in this process always wins over one loaded from a file, and the environment is read once at startup: changing the file requires restarting `dokime server`."
    )]
    pub env_file: Option<PathBuf>,

    #[arg(long = "plugin", help = "Load this trusted .dokime bundle for this process; repeat for multiple plugins.")]
    pub plugins: Vec<PathBuf>,

    #[arg(
        long,
        help = "Load every `.dokime` bundle in this development directory for this process. Nothing is installed or written there."
    )]
    pub plugins_root: Option<PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Args {
    pub fn configuration(&self) -> super::Result<dokime::config::Config> {
        let options = dokime::config::ApplicationOptions {
            application_root: self.application_root.clone(),
            storage_root: self.output_root.clone(),
            plugins_root: self.plugins_root.clone(),
        };
        Ok(options.resolve()?)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
