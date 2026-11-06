/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Scalar And Wide Ingest Fixture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{error::Error, prompts::checkpoint::SteadyCheck};
use dokime::prelude::*;

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ScalarZero;
pub struct ScalarOne;
pub struct ScalarMany;
pub struct WideZero;
pub struct WideOne;
pub struct WideMany;

#[dokime::parameters]
pub struct Parameters {
    #[parameter(label = "Rows", default = 100_000, min = 1, max = 1_000_000)]
    pub rows: u64,
}

#[dokime::stream(id = "obc.benchmarks.scalar", window = 4096, label = "Scalar Ingest")]
pub struct Scalar {
    pub value: f64,
}

#[dokime::stream(id = "obc.benchmarks.wide", window = 4096, label = "Wide Ingest")]
pub struct Wide {
    pub value: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
    pub g: f64,
    pub h: f64,
    pub i: f64,
    pub j: f64,
    pub k: f64,
    pub l: f64,
    pub m: f64,
    pub n: f64,
    pub o: f64,
}

/* === Enums === */

#[dokime::catalog]
pub enum Streams {
    Scalar(Scalar),
    Wide(Wide),
}

#[dokime::catalog]
pub enum Prompts {
    Check(SteadyCheck),
}

/* === Traits === */

/* === Implementations === */

macro_rules! fixture {
    ($name:ident, $id:literal, $wide:expr, $plots:expr) => {
        #[dokime::template(id = $id, label = "Stream Ingest", parameters = Parameters, streams = Streams, prompts = Prompts)]
        impl $name {
            async fn initialize(_: &Parameters, _: &mut InitializationContext<'_>) -> Result<Self, Error> {
                Ok(Self)
            }

            fn document() -> Document {
                let stream = if $wide { "obc.benchmarks.wide" } else { "obc.benchmarks.scalar" };
                let mut section = Section::new("Ingest");
                for (index, ()) in std::iter::repeat_n((), $plots).enumerate() {
                    let mut plot = Plot::time_series(format!("Value {index}"));
                    plot.id(format!("ingest.value.{index}"));
                    plot.y(dokime::expressions::telemetry(stream, "value"));
                    section.plot(plot);
                }
                let mut document = Document::new("default", "Stream Ingest");
                document.section(section);
                document
            }

            #[dokime::step(label = "Publish Rows")]
            async fn publish(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
                for index in 0..ctx.parameters().rows {
                    let value = index as f64;
                    if $wide {
                        ctx.stream::<Wide>()
                            .open()
                            .push(Wide {
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
                            })
                            .await;
                    } else {
                        ctx.stream::<Scalar>().open().push(Scalar { value }).await;
                    }
                }
                Ok(())
            }

            #[dokime::step(label = "Inspect Saved Rows")]
            async fn inspect(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
                ctx.ask(SteadyCheck::default()).await?;
                Ok(())
            }
        }
    };
}

fixture!(ScalarZero, "obc.benchmarks.ingest_scalar0", false, 0);
fixture!(ScalarOne, "obc.benchmarks.ingest_scalar1", false, 1);
fixture!(ScalarMany, "obc.benchmarks.ingest_scalar64", false, 64);
fixture!(WideZero, "obc.benchmarks.ingest_wide0", true, 0);
fixture!(WideOne, "obc.benchmarks.ingest_wide1", true, 1);
fixture!(WideMany, "obc.benchmarks.ingest_wide64", true, 64);

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
