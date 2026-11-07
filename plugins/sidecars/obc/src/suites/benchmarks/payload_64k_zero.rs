/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Payload 64K Zero Ingest Fixture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::domain::model::ingest::{Payload as Row, Prompts, Streams};
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

#[dokime::template(id = "obc.benchmarks.ingest_payload65536_0", label = "Stream Ingest", parameters = Parameters, streams = Streams, prompts = Prompts)]
impl Template {
    async fn initialize(_: &Parameters, _: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime::step(label = "Publish Rows")]
    async fn publish(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        for index in 0..ctx.parameters().rows {
            let value = index as f64;
            // Generate Deterministic Printable Payload
            let mut state = index.wrapping_add(1).wrapping_mul(0x9e3779b97f4a7c15);
            let data = (0..(65536_usize).saturating_sub(8))
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    (32 + state % 95) as u8 as char
                })
                .collect();
            let row = Row { value, data };

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
        let section = Section::new("Ingest");
        let mut document = Document::new("default", "Stream Ingest");
        document.section(section);
        document
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
