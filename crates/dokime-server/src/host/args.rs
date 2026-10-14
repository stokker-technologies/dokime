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
#[derive(Debug, clap::Args)]
#[command(about = "Listener flags shared by the standalone runtime and the Cargo wrapper.")]
#[group(id = "listener")]
pub struct NetworkArgs {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: IpAddr,
    #[arg(
        long,
        default_value_t = 8765,
        help = "Listen on this exact port; use zero to allocate an available port."
    )]
    pub port: u16,
    #[arg(long, requires_all = ["tls_cert", "tls_key", "auth_file"], conflicts_with = "dev_control", help = "Explicitly permit remote clients. Requires TLS and authenticated grants.")]
    pub allow_remote: bool,
    #[arg(long, requires = "allow_remote")]
    pub tls_cert: Option<PathBuf>,
    #[arg(long, requires = "allow_remote")]
    pub tls_key: Option<PathBuf>,
    #[arg(
        long,
        help = "JSON file containing subject, role and token grants. Never pass tokens in arguments."
    )]
    pub auth_file: Option<PathBuf>,
    #[arg(long, default_value_t = 2)]
    pub message_limit_mib: usize,
    #[arg(
        long,
        help = "Also serve gRPC-Web over HTTP/1.1 for browser clients. The desktop never needs it."
    )]
    pub grpc_web: bool,
    #[arg(
        long = "allowed-origin",
        requires = "grpc_web",
        help = "Allow This Exact Browser Origin For gRPC-Web; Repeat For Multiple Origins."
    )]
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, clap::Args)]
#[command(about = "Server process flags shared by the standalone runtime and Cargo wrapper.")]
#[group(id = "server-runtime-arguments")]
pub struct Args {
    #[arg(
        long,
        help = "Shut down when the controlling process closes standard input."
    )]
    pub shutdown_stdin: bool,
    #[arg(
        long,
        hide = true,
        requires = "shutdown_stdin",
        help = "Private development supervisor protocol."
    )]
    pub dev_control: bool,
    #[command(flatten)]
    pub network: NetworkArgs,

    #[arg(long, help = "Hub title; defaults to the resolved brand's label.")]
    pub title: Option<String>,

    #[arg(long)]
    pub label: Option<String>,

    #[arg(
        long,
        help = "Execution data directory for this process. Defaults to the data root saved in `settings.json`, or `data/` beneath the application root."
    )]
    pub output_root: Option<PathBuf>,

    #[arg(
        long,
        help = "Keep settings, logs, installed plugins, and default data beneath this root. Defaults to `DOKIME_APPLICATION_ROOT` or the per-user directory. `dokime-server` writes its rolling boot/diagnostic log under `<root>/logs`."
    )]
    pub application_root: Option<PathBuf>,

    #[arg(
        long,
        help = "Load environment variables from exactly this file with dotenv, instead of searching upward from the working directory for `.env`. Refused outright when the file does not exist — never a silent skip. Either way, a variable already exported in this process always wins over one loaded from a file, and the environment is read once at startup: changing the file requires restarting `dokime-server`."
    )]
    pub env_file: Option<PathBuf>,

    #[arg(
        long = "plugin",
        help = "Load this trusted .dokime bundle for this process; repeat for multiple plugins."
    )]
    pub plugins: Vec<PathBuf>,

    #[arg(
        long,
        help = "Load every `.dokime` bundle in this development directory for this process. Nothing is installed or written there."
    )]
    pub plugins_root: Option<PathBuf>,

    #[arg(long = "no-seed-validation-artifact")]
    pub no_seed_validation_artifact: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
