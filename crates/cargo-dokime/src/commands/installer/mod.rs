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
use std::path::PathBuf;

// Third Party Imports

// Local Imports
use crate::error::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
#[command(about = "Assemble Or Inspect Native Dokime Installers")]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Args)]
pub struct PackageArgs {
    #[arg(long, default_value = "installer.toml")]
    manifest: PathBuf,

    #[arg(long)]
    artifacts: Option<PathBuf>,

    #[arg(long = "plugin", value_name = "PATH", help = "Include An External Plugin Bundle Or Sidecars Archive; Repeat For Each Path")]
    plugins: Vec<PathBuf>,

    #[arg(long, default_value = "dist")]
    output: PathBuf,

    #[arg(long)]
    sign: bool,

    #[arg(long="format",value_delimiter=',',value_parser=["nsis","deb","rpm"])]
    formats: Vec<String>,

    #[arg(long)]
    target: Option<String>,

    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
pub struct InspectArgs {
    package: PathBuf,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

#[derive(clap::Subcommand)]
enum Command {
    Package(PackageArgs),
    Inspect(InspectArgs),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn run(args: Args) -> Result<(), Error> {
    match args.command {
        Command::Package(args) => {
            let formats = args
                .formats
                .iter()
                .map(|s| match s.as_str() {
                    "nsis" => dokime_packaging::installer::Format::Nsis,
                    "deb" => dokime_packaging::installer::Format::Deb,
                    _ => dokime_packaging::installer::Format::Rpm,
                })
                .collect();
            let options = dokime_packaging::installer::PackageOptions {
                artifacts: args.artifacts,
                plugins: args.plugins,
                output: args.output,
                sign: args.sign,
                formats,
                target: args.target,
            };
            let outputs = dokime_packaging::installer::package(&args.manifest, &options).await?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&outputs)?);
            } else {
                for output in outputs {
                    println!("{}", output.display());
                }
            }
        }
        Command::Inspect(args) => {
            let result = dokime_packaging::installer::inspect(&args.package)?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!(
                    "{} {} ({}) — {} Files Verified\nSHA-256: {}\nInstaller Signature: {}\nUpdater Signature: {}",
                    result.distribution.label,
                    result.distribution.version,
                    result.distribution.target,
                    result.verified_files,
                    result.sha256,
                    result.installer_signature,
                    result.updater_signature
                );
            }
        }
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Installer {
        #[command(flatten)]
        args: Args,
    }

    #[test]
    fn plugin_flags_are_explicit_and_repeatable() {
        let parsed = Installer::try_parse_from(["installer", "package"]).unwrap();
        let Command::Package(args) = parsed.args.command else { panic!("package command") };
        assert!(args.plugins.is_empty());

        let parsed = Installer::try_parse_from(["installer", "package", "--plugin", "host.dokime", "--plugin", "bench.dokime"]).unwrap();
        let Command::Package(args) = parsed.args.command else { panic!("package command") };
        assert_eq!(args.plugins, [PathBuf::from("host.dokime"), PathBuf::from("bench.dokime")]);
        assert!(Installer::try_parse_from(["installer", "package", "--plugin"]).is_err());
    }
}
