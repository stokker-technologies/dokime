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
use std::path::PathBuf;
// Third Party Imports

// Local Imports
use dokime::plugin::Secret;

use crate::HOME_VARIABLE;
use crate::operations::choices::ListChoices;
use crate::operations::status::GetStatus;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::settings]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RoutePluginSettings {
    pub fixture: Fixture,
}

#[dokime::section(label = "Fixture", status = GetStatus, description = "What this fixture is admitted with, in one section.\n`home` is an `Option<Secret>` because the fixture reads whether the host bound it rather than depending on one: the field's shape is what says whether an operator must fill it in.")]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fixture {
    #[setting(label = "Images Directory", default = "fixture-images")]
    pub images_directory: PathBuf,

    #[setting(label = "Host Home Fixture", from_environment = HOME_VARIABLE)]
    pub home: Option<Secret>,

    #[setting(label = "Access Token")]
    pub token: Option<Secret>,

    #[setting(label = "Offered Choice", options = ListChoices, description = "Which of the choices `list_choices` offers this fixture is bound to.")]
    pub offered: Offered,
}

/* === Enums === */
#[dokime::choice]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Offered {
    #[default]
    One,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
