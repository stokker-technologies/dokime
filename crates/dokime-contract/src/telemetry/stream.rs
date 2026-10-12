/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::core::Identifier;
use crate::telemetry::{TelemetryContract, TelemetryRow};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */
#[diagnostic::on_unimplemented(
    message = "`{Self}` declares no stream of its own",
    label = "this type has no stream for a plot or a publication to resolve",
    note = "declare a `#[dokime::stream(id = …, label = …, window = …)]`, or explicitly select a stream with `ctx.stream::<{Self}>().into::<Stream>()`"
)]
pub trait TelemetryStream: harmos::StreamSpec<Row: TelemetryRow> {
    fn contract() -> TelemetryContract;

    fn stream_id() -> Identifier {
        Identifier::declared(Self::ID, "telemetry stream")
    }
}

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
