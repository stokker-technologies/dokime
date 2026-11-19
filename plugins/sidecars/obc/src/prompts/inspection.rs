/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operator Checkpoints

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;

// Third Party Imports

// Local Imports
use dokime::Metadata;
use dokime::plugin::Prompt;
use dokime::templates::Context;
use dokime::{PromptArchetype, PromptField, PromptFieldKind, Result, Value, ValueType, value};

/* === Constants === */

pub const CHECK: &str = "operator_inspection";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// What the Operator Decided About an Inspection.
pub struct Inspection {
    pub accepted: bool,
    pub action: String,
    pub note: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn catalog() -> [Prompt; 1] {
    [inspection(CHECK, "Inspect Powered Charger")]
}

/// Ask an Inspection With a Prefilled Note; Acceptance Needs the Passed Box and an Accepted Answer.
pub async fn inspect(ctx: &Context, prompt: &str, note: &str) -> Result<Inspection> {
    let values = HashMap::from([(
        "operator_note".into(),
        Value {
            kind: Some(value::Kind::Text(note.into())),
        },
    )]);
    let answer = ctx.prompt_answer(Prompt::new(instance(prompt), "Charger Inspection", "Inspect the Charger"), values).await?;
    let passed = matches!(answer.values.get("inspection_passed").and_then(|value| value.kind.as_ref()), Some(value::Kind::Boolean(true)));
    let note = match answer.values.get("operator_note").and_then(|value| value.kind.as_ref()) {
        Some(value::Kind::Text(text)) => text.clone(),
        _ => note.into(),
    };
    let accepted = answer.accepted && passed;
    Ok(Inspection {
        accepted,
        action: if accepted { "continue" } else { "decline" }.into(),
        note,
    })
}

/* === Private Functions === */

fn instance(prompt: &str) -> String {
    prompt.rsplit('.').next().unwrap_or(prompt).into()
}

fn inspection(id: &str, label: &str) -> Prompt {
    let note = PromptField {
        metadata: Some(Metadata::new("operator_note", "Operator Note")),
        r#type: ValueType::Text as i32,
        kind: PromptFieldKind::Text as i32,
        default_value: Some(Value {
            kind: Some(value::Kind::Text(String::new())),
        }),
        ..Default::default()
    };
    let passed = PromptField {
        metadata: Some(Metadata::new("inspection_passed", "Charger Inspection Passed")),
        r#type: ValueType::Boolean as i32,
        kind: PromptFieldKind::Boolean as i32,
        default_value: Some(Value { kind: Some(value::Kind::Boolean(false)) }),
        ..Default::default()
    };
    Prompt::new(id, label, "Check That the Charger Status Indicator Is Steady and No Fault Is Shown, Then Record Your Observation.")
        .archetype(PromptArchetype::Form)
        .field(note)
        .field(passed)
        .action("submit", "Submit")
}

/* === Tests === */
