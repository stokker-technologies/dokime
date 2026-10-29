/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::path::PathBuf;

use harmos_packaging::{
    proto::Artifacts,
    python::{self, Settings},
};

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Args)]
pub struct PythonArgs {
    #[arg(long, help = "The Cargo OUT_DIR Contract Export Of The Selected Build.")]
    export: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    distribution: String,
    #[arg(long)]
    module: String,
    #[arg(long)]
    package_version: semver::Version,
    #[arg(long)]
    description: String,
    #[arg(long)]
    license: String,
    #[arg(long, default_value = "protoc")]
    protoc: PathBuf,
}

#[derive(clap::Subcommand)]
enum Command {
    #[command(about = "Generate The Python Client Package From One Contract Export")]
    Python(PythonArgs),
}

pub fn run(args: Args) -> Result<(), crate::error::Error> {
    let Command::Python(args) = args.command;
    let artifacts = Artifacts::read(&args.export).map_err(python::Error::from)?;
    let contracts = artifacts.operation_contracts().map_err(python::Error::from)?;
    let settings = Settings {
        distribution: args.distribution,
        module: args.module,
        version: args.package_version,
        description: args.description,
        license: args.license,
        operations: Vec::new(),
    };
    let catalog = python::Catalog {
        contracts: &contracts,
        settings: &settings,
        protoc: &args.protoc,
    };
    println!("{}", python::generate(catalog, &args.output)?.display());
    Ok(())
}
