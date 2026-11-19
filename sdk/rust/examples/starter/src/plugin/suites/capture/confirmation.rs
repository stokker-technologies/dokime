/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operator Confirmation

/* === Modules === */



/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::templates::{Context, Criterion, Record, Step, Template};
use dokime::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ConfirmationTestTemplate;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ConfirmationTestTemplate {
    pub fn confirmation() -> Record<bool> {
        Record::of(Self::confirmation, "Confirmation")
    }
    pub fn template() -> Template {
        Template::of::<Self>()
            .record(Self::confirmation())
            .step(Step::new("Confirm", Self::confirm))
            .criterion(Criterion::new("Confirmed", Self::confirmed))
    }
    async fn confirm(ctx: &Context) -> Result<()> {
        let answer = ctx.prompt("Continue?", &["Yes", "No"]).await?;
        ctx.record(&Self::confirmation(), answer == "Yes").await?;
        Ok(())
    }
    async fn confirmed(ctx: &Context) -> Result<bool> {
        ctx.records(&Self::confirmation()).step(Self::confirm).last().await
    }
}



/* === Public Functions === */



/* === Private Functions === */



/* === Tests === */


