/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read or Update the Host Through the SDK

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_client_internal::SuitesGetOptions;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    suite: String,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let client = settings.sdk(endpoint).await?;
    let value = client
        .suites
        .get_with(
            args.suite,
            "",
            0,
            SuitesGetOptions {
                context: settings.context()?,
                ..Default::default()
            },
        )
        .await?;
    if args.json {
        return crate::output::print(&value, true);
    }
    let suite = value.value.ok_or("Read Suite: Select an Available Suite")?;
    let metadata = suite.metadata.ok_or("Read Suite: Read a Suite With Metadata")?;
    println!("{}\t{}\t{} Templates", metadata.identifier, metadata.label, suite.templates.len());
    for template in suite.templates {
        let metadata = template.metadata.ok_or("Read Suite: Read a Template With Metadata")?;
        println!("{}\t{}\t{} Parameters\t{} Steps", metadata.identifier, metadata.label, template.parameters.len(), template.steps.len());
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
