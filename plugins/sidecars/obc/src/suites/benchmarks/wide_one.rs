/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Wide One Ingest Fixture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::domain::model::ingest::{Prompts, Streams, Wide as Row};
use crate::domain::{Error, model::prompts::checkpoint::SteadyCheck};
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Template;

#[dokime::parameters]
pub struct Parameters {
    #[parameter(label = "Rows", default = 100_000, min = 1, max = 1_000_000)]
    pub rows: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[dokime::template(id = "obc.benchmarks.ingest_wide1", label = "Stream Ingest", parameters = Parameters, streams = Streams, prompts = Prompts)]
impl Template {
    async fn initialize(_: &Parameters, _: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime::step(label = "Publish Rows")]
    async fn publish(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        for index in 0..ctx.parameters().rows {
            let value = index as f64;
            let row = Row {
                value,
                a: value + 1.0,
                b: value + 2.0,
                c: value + 3.0,
                d: value + 4.0,
                e: value + 5.0,
                f: value + 6.0,
                g: value + 7.0,
                h: value + 8.0,
                i: value + 9.0,
                j: value + 10.0,
                k: value + 11.0,
                l: value + 12.0,
                m: value + 13.0,
                n: value + 14.0,
                o: value + 15.0,
            };

            // Publish Row
            ctx.stream::<Row>().open().push(row).await;
        }
        Ok(())
    }

    #[dokime::step(label = "Inspect Saved Rows")]
    async fn inspect(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Ask for Operator Inspection
        let prompt = SteadyCheck::default();
        ctx.ask(prompt).await?;
        Ok(())
    }

    fn document() -> Document {
        let mut section = Section::new("Ingest");
        for index in 0..1 {
            let mut plot = Plot::time_series(format!("Value {index}"));
            plot.id(format!("ingest.value.{index}"));
            plot.y(dokime::expressions::telemetry("obc.benchmarks.wide", "value"));
            section.plot(plot);
        }
        let mut document = Document::new("Stream Ingest");
        document.section(section);
        document
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
