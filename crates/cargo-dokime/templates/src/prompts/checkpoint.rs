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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// The struct is the question and the answer at once: its fields are the
// form's fields, the attribute is the title and the actions, and what the
// operator submits decodes into this type. The template names it once in
// `prompts = Prompts` and the step asks it with
// `ctx.ask(SoakCheckpoint::default())`, so no id or field is spelled anywhere
// else. A submitted `accepted: false` is an answer, not a cancellation.
//
// A step whose default depends on what it just measured asks through the
// struct instance: `SoakCheckpoint { note: seen, ..Default::default() }`.
#[dokime::prompt(
    id = "{{owner}}.soak.checkpoint",
    label = "Sign off the soak",
    submit = "Sign off"
, description = "Check the chamber and the unit, then record what you saw.")]
pub struct SoakCheckpoint {
    #[prompt(label = "Operator note", default = "chamber steady")]
    pub note: String,
    #[prompt(label = "The soak looked right", default = true)]
    pub accepted: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
