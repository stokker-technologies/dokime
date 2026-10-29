/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Benchmark-Only High-Volume Evidence And Prompt Fixture

mod capture;

pub struct Suite;

#[dokime::catalog]
pub enum Templates {
    Capture(capture::Capture),
}

#[dokime::suite(
    id = "obc.benchmarks", label = "Benchmark Fixtures", templates = Templates,
    description = "Generated high-volume evidence for system benchmarks; excluded from product builds."
)]
impl Suite {}
