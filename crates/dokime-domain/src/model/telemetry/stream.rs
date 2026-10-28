/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::model::identity::Identifier;
use crate::model::telemetry::TelemetryContract;
use crate::model::telemetry::TelemetryRow;

#[diagnostic::on_unimplemented(
    message = "`{Self}` declares no stream of its own",
    label = "this type has no stream for a plot or a publication to resolve",
    note = "declare a `#[dokime_runtime::stream(id = …, label = …, window = …)]`, or explicitly select a stream with `ctx.stream::<{Self}>().into::<Stream>()`"
)]
pub trait TelemetryStream: harmos_domain::StreamSpec<Row: TelemetryRow> {
    const PUBLISHED: bool = true;

    fn contract() -> TelemetryContract;

    fn stream_id() -> Identifier {
        Identifier::declared(Self::ID, "telemetry stream")
    }
}
