/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[cfg(feature = "host")]
pub(crate) mod admission;
mod authoring;
mod declared;
mod validation;

pub use authoring::{ChoicePrompt, ConfirmationPrompt, FormPrompt, MultipleChoicePrompt, NumberPrompt, Prompt, PromptNumber, TextPrompt};
pub use declared::{DeclaredPrompt, PromptFieldValue};
pub use declared::{
    answered, confirmation_answer, declared_confirmation, declared_default, declared_field, declared_form, declared_metadata, form_answer,
};
pub use dokime_domain::model::prompts::*;
pub use validation::{validate_answers, validate_definition, validate_fields};
