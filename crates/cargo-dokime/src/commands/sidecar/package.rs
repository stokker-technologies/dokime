/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::path::PathBuf;

use sha2::Digest;

#[derive(clap::Args)]
#[command(about = "Bundle the executables a collected build names into a .dokime bundle, without compiling", long_about = None)]
pub struct Args {
    #[arg(long)]
    build_json: PathBuf,

    #[arg(short, long)]
    output: PathBuf,

    #[arg(long)]
    json: bool,
}

pub async fn run(args: Args) -> Result<(), crate::error::Error> {
    let collected: dokime_packaging::build::Collected = serde_json::from_slice(&std::fs::read(&args.build_json)?)?;
    std::fs::create_dir_all(&args.output)?;
    let bundle = dokime_packaging::build::package(collected, &args.output).await?;
    if args.json {
        let sha256 = sha2::Sha256::digest(std::fs::read(&bundle)?).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        println!("{}", serde_json::json!([{ "path": bundle, "sha256": sha256 }]));
    } else {
        println!("{}", bundle.display());
    }
    Ok(())
}
