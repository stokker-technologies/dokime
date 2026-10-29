/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod plugins;
mod preferences;
mod sources;
mod wire;

use dokime::app::LiveRuntime;

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    preferences::prepare(setup, runtime, maximum)?;
    sources::prepare(setup, runtime, maximum)?;
    plugins::prepare(setup, runtime, maximum)
}
