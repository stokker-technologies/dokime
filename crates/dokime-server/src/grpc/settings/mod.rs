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

use dokime::RuntimeHandle;

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    preferences::register(runtime, maximum).await?;
    sources::register(runtime, maximum).await?;
    plugins::register(runtime, maximum).await
}
