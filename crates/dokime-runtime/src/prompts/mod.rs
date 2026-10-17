/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod authoring;
mod declared;
mod validation;

/* === Re-Exports === */
pub use authoring::{
    ChoicePrompt, ConfirmationPrompt, FormPrompt, MultipleChoicePrompt, NumberPrompt, Prompt,
    PromptNumber, TextPrompt,
};
pub use declared::{DeclaredPrompt, PromptFieldValue};
pub use dokime_domain::model::prompts::*;
pub use validation::{validate_answers, validate_definition, validate_fields};

pub use declared::{
    answered, confirmation_answer, declared_confirmation, declared_default, declared_field,
    declared_form, declared_metadata, form_answer,
};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
