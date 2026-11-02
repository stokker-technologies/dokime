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
use dokime_runtime::execution::{CriterionContext, InitializationContext, StepContext};
use dokime_runtime::plugin::testing::Harness;
use dokime_runtime::prompts::{DeclaredPrompt, FormPrompt, PromptDefinition};
use dokime_runtime::records::RecordSeverity;

/* === Constants === */
const SUITE: &str = "prompts.checks";
const TEMPLATE: &str = "prompts.checks.ask";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::prompt(
    id = "prompts.checks.signoff",
    label = "Sign off",
    submit = "Sign off",
    level = warning,
    description = "The form: its fields are the question's fields and the answer's fields.\nRecord what you saw."
)]
#[derive(Debug)]
pub struct SignOff {
    #[prompt(label = "Operator note", default = "steady", max_length = 200)]
    pub note: String,
    #[prompt(default = true)]
    pub accepted: bool,
    #[prompt(label = "Sweep", default = Sweep::Fast)]
    pub sweep: Sweep,
    #[prompt(default = Some(3))]
    pub retries: Option<i64>,
    #[prompt(label = "Sweeps to repeat", default = vec![Sweep::Slow], min_items = 1)]
    pub repeats: Vec<Sweep>,
}

#[dokime_runtime::prompt(
    id = "prompts.checks.go",
    label = "Continue?",
    description = "The confirmation: a unit struct that answers yes or no."
)]
pub struct Go;

struct Checks;

struct Ask {
    heard: Vec<String>,
}

/* === Enums === */
#[dokime_runtime::choice]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sweep {
    Fast,
    #[choice(label = "Slow and careful")]
    Slow,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("dokime sdk: {0}")]
    Sdk(#[from] dokime_runtime::Error),
}

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::catalog]
enum PluginSuites {
    Checks(Checks),
}

#[dokime_runtime::catalog]
enum ChecksTemplates {
    Ask(Ask),
}

#[dokime_runtime::suite(id = SUITE, label = "Prompts", templates = ChecksTemplates)]
impl Checks {}

#[dokime_runtime::catalog]
enum AskPrompts {
    SignOff(SignOff),
    Go(Go),
}

#[dokime_runtime::template(id = TEMPLATE, label = "Ask", prompts = AskPrompts)]
impl Ask {
    async fn initialize(_ctx: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self { heard: Vec::new() })
    }

    #[dokime_runtime::step(label = "Ask by type")]
    async fn declared(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // === Ask The Form === //

        let answer = ctx.ask(SignOff::default()).await?;
        self.heard.push(format!(
            "form note={} accepted={} sweep={:?} retries={:?} repeats={:?}",
            answer.note, answer.accepted, answer.sweep, answer.retries, answer.repeats
        ));

        // === Ask The Confirmation === //

        let go = ctx.ask(Go).await?;
        self.heard.push(format!("go={go}"));

        // === Ask With Custom Initial Values === //

        let seen = format!("{} samples looked right", 3);
        let again = ctx
            .ask(SignOff {
                note: seen,
                retries: None,
                ..Default::default()
            })
            .await?;
        self.heard.push(format!("custom note={} retries={:?}", again.note, again.retries));
        for line in &self.heard {
            dokime_runtime::info!(ctx, "{line}");
        }

        Ok(())
    }

    #[dokime_runtime::criterion(id = "prompts.checks.answered", label = "Answered")]
    fn answered(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        Ok(ctx.steps_reached().len() == 1)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn logged(report: &dokime_runtime::plugin::testing::Report) -> Vec<String> {
    report.log().iter().map(|entry| entry.message.clone()).collect()
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn declared_prompt_declaration() {
    let report = Harness::template(TEMPLATE).run::<PluginSuites>().await.expect("the run completes");
    assert!(report.passed(), "{}", report.diagnostics());

    // Unanswered, the harness continues every asking with the declaration's own
    // first non-cancel action and the declared defaults: the form's `submit`
    // with `steady`/`true`/`fast`, the confirmation's `yes`, and the instance's
    // runtime default in place of the declared one.
    let logged = logged(&report);
    assert!(logged.contains(&"form note=steady accepted=true sweep=Fast retries=Some(3) repeats=[Slow]".to_owned()), "{logged:?}");
    assert!(logged.contains(&"go=true".to_owned()), "{logged:?}");
    assert!(logged.contains(&"custom note=3 samples looked right retries=None".to_owned()), "{logged:?}");
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn answers_declared_constant() {
    let report = Harness::template(TEMPLATE)
        .answer(
            SignOff::PROMPT_TYPE,
            "submit",
            [
                ("note", dokime_runtime::Value::from("wobbly")),
                ("accepted", dokime_runtime::Value::from(false)),
                ("sweep", dokime_runtime::Value::from("slow")),
                ("retries", dokime_runtime::Value::from(2_i64)),
                ("repeats", dokime_runtime::Value::Array(vec![dokime_runtime::Value::from("slow"), dokime_runtime::Value::from("fast")])),
            ],
        )
        .answer(Go::PROMPT_TYPE, "no", std::iter::empty::<(&str, dokime_runtime::Value)>())
        .run::<PluginSuites>()
        .await
        .expect("the run completes");
    assert!(report.passed(), "{}", report.diagnostics());

    let logged = logged(&report);
    assert!(logged.contains(&"form note=wobbly accepted=false sweep=Slow retries=Some(2) repeats=[Slow, Fast]".to_owned()), "{logged:?}");
    assert!(logged.contains(&"go=false".to_owned()), "{logged:?}");
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[test]
fn declaration_what_registers() {
    let definition = SignOff::definition();
    assert_eq!(definition.prompt_type.as_str(), SignOff::PROMPT_TYPE);
    assert_eq!(
        definition.message.as_deref(),
        Some(
            "The form: its fields are the question's fields and the answer's fields.\n\
             Record what you saw."
        )
    );
    assert_eq!(definition.actions[0].label, "Sign off");
    let names: Vec<&str> = definition.fields.iter().map(|field| field.metadata.identifier.as_str()).collect();
    assert_eq!(names, ["note", "accepted", "sweep", "retries", "repeats"]);
    assert_eq!(definition.fields[2].choices, ["fast", "slow"]);
    assert!(!definition.fields[3].required);
    assert_eq!(definition.fields[0].validation.max_length, Some(200));
    // A `Vec<Choice>` is a multiple choice over the same variants, and its
    // declared default travels as a list.
    assert!(definition.fields[4].multiple);
    assert_eq!(definition.fields[4].choices, ["fast", "slow"]);
    assert_eq!(definition.fields[4].validation.min_items, Some(1));
    assert_eq!(definition.fields[4].default, Some(dokime_runtime::Value::Array(vec![dokime_runtime::Value::from("slow")])));
}

#[dokime_tracing::trace_test]
#[test]
fn prompt_level_roundtrips() {
    // === Declared And Built Levels === //

    let declared = SignOff::definition();
    let unraised = Go::definition();
    let built = dokime_runtime::prompts::Prompt::definition(
        &FormPrompt::<std::collections::BTreeMap<String, String>>::new("prompts.checks.built", "Built").level(RecordSeverity::Error),
    );

    // === Encode And Decode === //

    let encoded = serde_json::to_value(&declared).expect("a prompt definition encodes");
    let decoded: PromptDefinition = serde_json::from_value(encoded.clone()).expect("a prompt definition decodes");
    let mut baseline = encoded;
    baseline.as_object_mut().expect("a prompt definition encodes as an object").remove("level");
    let defaulted: PromptDefinition = serde_json::from_value(baseline).expect("a definition without a level decodes");

    // === Compare === //

    assert_eq!(declared.level, RecordSeverity::Warning);
    assert_eq!(unraised.level, RecordSeverity::Info);
    assert_eq!(built.level, RecordSeverity::Error);
    assert_eq!(decoded, declared);
    assert_eq!(defaulted.level, RecordSeverity::Info);
}
