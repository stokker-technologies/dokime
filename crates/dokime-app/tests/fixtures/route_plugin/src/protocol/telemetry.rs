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
use dokime::core::{TemplateId, ValueType};
use dokime::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole};
use dokime::{Refusal, Supervisor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Reading {
    value: f64,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Publish {
    context: String,
    values: Vec<f64>,
}
pub(crate) struct Readings;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl dokime::StreamSpec for Readings {
    type Row = Reading;
    const ID: &'static str = "fixture-sidecar.reading";
    const VERSION: u32 = 1;
    const WINDOW: usize = 32;
}

impl Resources {
    pub(super) async fn publish_telemetry(
        &self,
        host: &Supervisor,
        request: Publish,
    ) -> Result<Empty, Refusal> {
        let token = if request.context == "current" {
            self.state.lock().unwrap().token.clone()
        } else {
            request.context
        };
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
    let mut contract = TelemetryContract::declared("fixture-sidecar.reading", "Reading")
        .columns([TelemetryColumn::declared(
            "value",
            "Value",
            ValueType::Number,
            TelemetryRole::Value,
        )])
        .strict();
    contract.template_id = Some(template);
    contract
}

/* === Private Functions === */

/* === Tests === */
