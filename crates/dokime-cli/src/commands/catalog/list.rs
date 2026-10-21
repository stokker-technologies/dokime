/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// List Registered Suites

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Emit One Complete Suite Definition Per JSON Line.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let client = args.connection.connect().await?;
    let result = async {
        let suites = client.suites()?;
        while let Some(suite) = suites.message().await? {
            if args.json {
                println!("{}", serde_json::to_string(&super::readable(&suite)?)?);
            } else {
                let identity = suite.identity.as_ref().ok_or_else(|| crate::Error::Setup("suite has no identity".into()))?;
                let label = identity.metadata.as_ref().map_or("", |metadata| metadata.label.as_str());
                println!("{}\t{}\t{} Templates", identity.suite_id, label, suite.templates.len());
                // Each template under its suite, by the ID `plan --template` takes.
                for metadata in suite.templates.iter().filter_map(|template| template.metadata.as_ref()) {
                    println!("\t{}\t{}", metadata.identifier, metadata.label);
                }
            }
        }
        suites.close().await;
        Ok(())
    }
    .await;
    client.close_and_wait().await;
    result
}

/* === Private Functions === */

/* === Tests === */
