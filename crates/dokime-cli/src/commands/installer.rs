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
use std::{path::PathBuf, time::Instant};

// Third Party Imports

// Local Imports
use crate::Result;
use dokime_packaging::installer::{Observer, Step};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
#[command(about = "Assemble Native Dokime Installers")]
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

    #[arg(long = "plugin", value_name = "PATH", help = "Include an External Plugin Bundle or Plugin Archive; Repeat for Each Path")]
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

/* === Enums === */

#[derive(clap::Subcommand)]
enum Command {
    Package(PackageArgs),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn run(args: Args) -> Result<()> {
    match args.command {
        Command::Package(args) => {
            let started = Instant::now();
            if !args.json {
                eprintln!("Dokime {}\n", env!("CARGO_PKG_VERSION"));
            }
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
                command: |program| dokime_runtime::process::command(program),
                artifacts: args.artifacts,
                plugins: args.plugins,
                output: args.output,
                sign: args.sign,
                formats,
                target: args.target,
            };
            let mut report = |step: Step| {
                if step.failed {
                    eprint!("  {:<12}", format!("{:?}", step.stage));
                    return;
                }
                for (index, detail) in step.details.iter().enumerate() {
                    let label = if index == 0 { format!("{:?}", step.stage) } else { String::new() };
                    if index + 1 == step.details.len() {
                        eprintln!("  {label:<12}{detail}    {:.1} s", step.duration.as_secs_f64());
                    } else {
                        eprintln!("  {label:<12}{detail}");
                    }
                }
            };
            let mut observer = if args.json { Observer::default() } else { Observer::new(&mut report) };
            let outputs = dokime_packaging::installer::package(&args.manifest, &options, &mut observer).await?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&outputs)?);
            } else {
                for output in &outputs {
                    println!("{}", output.display());
                }
                let noun = if outputs.len() == 1 { "Installer" } else { "Installers" };
                eprintln!("\nPackaged {} {noun} in {:.1} s", outputs.len(), started.elapsed().as_secs_f64());
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
        let Command::Package(args) = parsed.args.command;
        assert!(args.plugins.is_empty());

        let parsed =
            Installer::try_parse_from(["installer", "package", "--plugin", "plugin-host.dokime", "--plugin", "bench.dokime"]).unwrap();
        let Command::Package(args) = parsed.args.command;
        assert_eq!(args.plugins, [PathBuf::from("plugin-host.dokime"), PathBuf::from("bench.dokime")]);
        assert!(Installer::try_parse_from(["installer", "package", "--plugin"]).is_err());
    }
}
