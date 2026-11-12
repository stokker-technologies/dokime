/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A First Plugin With an Operator Prompt and Retained Evidence

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use dokime::{prelude::*, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Record)]
#[harmos(package = "example.v1")]
struct Confirmation {
    accepted: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    let confirmation = Record::<Confirmation>::new("example.confirmation", "Confirmation");
    let template = Template::new("example.capture", "Capture")
        .step(Step::new("Confirm", confirm).identifier("confirm"))
        .criterion(Criterion::new("Confirmed", confirmed).identifier("confirmed"));
    let plugin = Plugin::new("example", "0.1.0")
        .suite(Suite::new("example", "Example").template(template))
        .record(confirmation)
        .prompt(Prompt::new("example.confirmation", "Confirmation", "Continue?").action("yes", "Yes").action("no", "No"))
        .requires_host::<v1::OperatorQuestion>()
        .requires_host::<v1::PublishRecord>()
        .requires_host::<v1::RecordsRequest>();
    let plugin = if std::env::args().any(|argument| argument == "--remote") { plugin.remote() } else { plugin };
    plugin.serve()
}
async fn confirm(ctx: &Context) -> Result<()> {
    let answer = ctx
        .prompt(v1::OperatorQuestion {
            prompt: "confirmation".into(),
            prompt_type: "example.confirmation".into(),
            ..Default::default()
        })
        .await?;
    let record = Record::<Confirmation>::new("example.confirmation", "Confirmation");
    ctx.record(
        &record,
        Confirmation {
            accepted: answer.accepted && answer.action == "yes",
        },
    )
    .await?;
    Ok(())
}
async fn confirmed(ctx: &Context) -> Result<bool> {
    let records = ctx.records("example.confirmation", "confirm", 0).await?;
    let record = records.last.ok_or_else(|| Error::Invalid("Read Confirmation: Missing Record".into()))?;
    let value = <Confirmation as dokime::encoding::Message>::decode(record.payload.as_slice())
        .map_err(|error| Error::Invalid(error.to_string()))?;
    Ok(value.accepted)
}

/* === Tests === */
