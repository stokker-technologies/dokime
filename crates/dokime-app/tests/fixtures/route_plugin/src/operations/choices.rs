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
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::Resources;
use dokime::domains::{OptionProviderOption, OptionsResponse};
use dokime::{Refusal, Supervisor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct ListChoices;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime::operation(
    id = "route-fixture.list_choices",
    version = 1,
    description = "Offers the choices the settings card's `offered` field resolves through.",
    policy = crate::POLICY
)]
impl ListChoices {
    pub(crate) async fn initialize(_: Arc<Resources>, _: &Supervisor) -> Self {
        Self
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<OptionsResponse, Refusal> {
        Ok(OptionsResponse::of([OptionProviderOption {
            value: "one".into(),
            label: "One".into(),
            description: None,
            disabled: false,
            metadata: None,
        }]))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
