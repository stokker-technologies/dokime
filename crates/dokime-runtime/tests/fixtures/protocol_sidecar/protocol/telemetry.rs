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
use super::{Empty, Protocol, Resources};
use dokime_runtime::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole};
use dokime_runtime::{Refusal, Supervisor};
use dokime_runtime::{TemplateId, ValueType};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::row]
#[derive(Clone, PartialEq)]
pub(crate) struct Reading {
    value: f64,
}

#[dokime_runtime::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Publish {
    context: String,
    values: Vec<f64>,
}
pub(crate) struct Readings;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl dokime_runtime::StreamSpec for Readings {
    type Row = Reading;
    const ID: &'static str = "fixture-sidecar.reading";
    const VERSION: u32 = 1;
    const WINDOW: usize = 32;
}

impl dokime_runtime::telemetry::TelemetryStream for Readings {
    fn contract() -> TelemetryContract {
        TelemetryContract::declared("fixture-sidecar.reading", "Reading")
            .typed::<Reading>()
            .columns([TelemetryColumn::declared("value", "Value", ValueType::Number, TelemetryRole::Value)])
            .strict()
    }
}

impl Resources {
    pub(super) async fn publish_telemetry(&self, host: &Supervisor, request: Publish) -> Result<Empty, Refusal> {
        let token = if request.context == "current" { self.state.lock().unwrap().token.clone() } else { request.context };
        for value in request.values {
            let row = Reading { value };
            let context = (!token.is_empty()).then_some(token.as_str());
            host.publisher::<Protocol>()
                .publish_on_context::<Readings>(&row, context)
                .await
                .map_err(|error| Refusal::of(error.to_string()))?;
        }
        Ok(Empty {})
    }
}

/* === Public Functions === */
pub(crate) fn contract(template: TemplateId) -> TelemetryContract {
    <Readings as dokime_runtime::telemetry::TelemetryStream>::contract().for_template(template)
}

/* === Private Functions === */

/* === Tests === */
