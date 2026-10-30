/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub(crate) mod native;
pub(crate) mod read;
pub(crate) mod source;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports
use base64::Engine;
use cssparser::{Parser, Token};
use serde::{Deserialize, Serialize};

// Local Imports
use crate::definition::TestCase;
use crate::document::DocumentDefinition;
use crate::installed_plugins::RuntimeAppearanceFrame;
use crate::prompts::PromptInstance;
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::health::HealthItem;
use dokime_domain::model::execution::notes::OperatorNote;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::traceability::TraceabilitySnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionContext {
    pub appearance_policy: AppearancePolicy,
    pub scope: RecordScope,
    pub session_id: SessionId,
    pub cycle: ExecutionCycleContext,
    pub execution_documents: BTreeMap<DocumentId, DocumentDefinition>,
    pub prompts: Vec<PromptInstance>,
    pub health: Vec<HealthItem>,
    pub notes: Vec<OperatorNote>,
    pub traceability: Vec<TraceabilitySnapshot>,
    pub appearance: RuntimeAppearanceFrame,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppearancePolicy {
    pub logo_bytes: u64,
    pub css_depth: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionCycleContext {
    pub cycle: CycleView,
    pub cases: Vec<TestCase>,
    pub execution_order: Vec<TestExecutionId>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl AppearancePolicy {
    pub fn capture() -> Self {
        Self { logo_bytes: 1_048_576, css_depth: 64 }
    }

    fn validate(&self) -> Result<()> {
        if self.logo_bytes == 0
            || self.css_depth == 0
            || usize::try_from(self.logo_bytes).is_err()
            || usize::try_from(self.css_depth).is_err()
        {
            return Err(invalid("captured appearance policy is invalid"));
        }
        Ok(())
    }
}

impl ExecutionContext {
    pub fn validate(&self, execution: &TestExecution) -> Result<()> {
        if self.scope != execution.scope || self.cycle.cycle.test_cycle_id != execution.test_cycle_id {
            return Err(invalid("captured context differs from its execution owner"));
        }
        if self.prompts.iter().any(|item| !owns_projection(&item.scope, &self.scope))
            || self.health.iter().any(|item| !owns_annotation(&item.scope, &self.scope))
            || self.notes.iter().any(|item| !owns_annotation(&item.scope, &self.scope))
        {
            return Err(invalid("captured annotation scope differs"));
        }
        let notes = self.notes.iter().map(|item| &item.note_id).collect::<BTreeSet<_>>();
        let prompts = self.prompts.iter().map(|item| &item.prompt_id).collect::<BTreeSet<_>>();
        let health = self.health.iter().map(|item| &item.health_item_id).collect::<BTreeSet<_>>();
        if notes.len() != self.notes.len() || prompts.len() != self.prompts.len() || health.len() != self.health.len() {
            return Err(invalid("captured annotation identity is duplicated"));
        }
        if self.execution_documents.iter().any(|(id, document)| id != &document.document_id)
            || self.traceability.iter().any(|item| {
                item.scope.test_cycle_id.as_ref().is_some_and(|id| id != &execution.test_cycle_id)
                    || item.scope.test_execution_id.as_ref().is_some_and(|id| id != &execution.test_execution_id)
                    || item.scope.test_case_id.as_ref().is_some_and(|id| id != &execution.test_case_id)
            })
        {
            return Err(invalid("captured definitions or traceability owner differs"));
        }
        let members = self.cycle.execution_order.iter().collect::<BTreeSet<_>>();
        let cases = self.cycle.cases.iter().map(|case| &case.test_case_id).collect::<BTreeSet<_>>();
        if !self.cycle.cycle.test_executions.is_empty()
            || members.len() != self.cycle.execution_order.len()
            || cases.len() != self.cycle.cases.len()
            || !members.contains(&execution.test_execution_id)
            || !self.cycle.cases.iter().any(|case| case == execution.snapshot.case())
        {
            return Err(invalid("captured navigation differs from frozen membership"));
        }
        validate_appearance(&self.appearance, &self.appearance_policy)
    }
}

/* === Public Functions === */

// Each execution owns its own projections and copies only its cycle/case ancestors.
// Global items are deliberately outside the cycle export selection.
pub(crate) fn owns_projection(scope: &RecordScope, execution: &RecordScope) -> bool {
    (scope.test_cycle_id.is_some() || scope.test_execution_id.is_some())
        && scope.test_cycle_id.as_ref().is_none_or(|id| Some(id) == execution.test_cycle_id.as_ref())
        && scope.test_case_id.as_ref().is_none_or(|id| Some(id) == execution.test_case_id.as_ref())
        && scope.test_execution_id.as_ref().is_none_or(|id| Some(id) == execution.test_execution_id.as_ref())
}

// Annotation context also retains session/global observations. Export selection
// separately decides whether these belong to an explicitly cycle-scoped bundle.
pub(crate) fn owns_annotation(scope: &RecordScope, execution: &RecordScope) -> bool {
    if (scope.step_id.is_some() || scope.criterion_id.is_some()) && scope.test_execution_id.is_none() {
        return false;
    }
    (scope.test_cycle_id.is_none() && scope.test_case_id.is_none() && scope.test_execution_id.is_none())
        || owns_projection(scope, execution)
}

pub(crate) fn validate_appearance(appearance: &RuntimeAppearanceFrame, policy: &AppearancePolicy) -> Result<()> {
    policy.validate()?;
    validate_css_appearance(appearance, policy)?;
    for (index, logo) in appearance.logos.iter().enumerate() {
        if appearance.logos[..index].iter().any(|other| other.variant == logo.variant) {
            return Err(invalid("captured appearance has duplicate logo categories"));
        }
        validate_logo(&logo.url, policy.logo_bytes)?;
    }
    Ok(())
}

pub(crate) fn validate_css_appearance(appearance: &RuntimeAppearanceFrame, policy: &AppearancePolicy) -> Result<()> {
    policy.validate()?;
    for theme in &appearance.themes {
        for value in css_values(theme) {
            validate_css(value, policy.css_depth).map_err(|reason| invalid(&format!("captured theme {}: {reason}", theme.theme_id)))?;
        }
    }
    Ok(())
}

pub(crate) fn css_values(theme: &crate::installed_plugins::RuntimeThemeFamily) -> impl Iterator<Item = &str> {
    let radii = &theme.corner_radii;
    theme
        .stylesheet
        .iter()
        .chain(theme.variables.values())
        .chain(theme.light.variables.values())
        .chain(theme.dark.variables.values())
        .chain(theme.light.plot_palette.iter())
        .chain(theme.dark.plot_palette.iter())
        .map(String::as_str)
        .chain(
            [
                radii.base.as_deref(),
                radii.sm.as_deref(),
                radii.md.as_deref(),
                radii.lg.as_deref(),
                radii.xl.as_deref(),
                radii.two_xl.as_deref(),
                radii.three_xl.as_deref(),
                radii.four_xl.as_deref(),
            ]
            .into_iter()
            .flatten(),
        )
}
/* === Private Functions === */

fn validate_css(css: &str, maximum: u64) -> std::result::Result<(), &'static str> {
    let mut input = Parser::new(css);
    scan(&mut input, 0, maximum).map_err(|_| "CSS must use embedded data URLs or document fragments; imports, dynamic resource functions and malformed tokens are unsupported")
}

fn scan<'i>(input: &mut Parser<'i>, depth: u64, maximum: u64) -> std::result::Result<(), cssparser::ParseError<()>> {
    if depth > maximum {
        return Err(cssparser::ParseError::custom(()));
    }
    while !input.is_exhausted() {
        let token = input.next()?.clone();
        match token {
            Token::AtKeyword(name) if name.eq_ignore_ascii_case("import") => {
                return Err(cssparser::ParseError::custom(()));
            }
            Token::UnquotedUrl(url) => {
                if !portable_url(&url) {
                    return Err(cssparser::ParseError::custom(()));
                }
            }
            Token::Function(name) if name.eq_ignore_ascii_case("url") => {
                input.parse_nested_block(|input| {
                    let url = input.expect_string()?;
                    if !portable_url(url) {
                        return Err(cssparser::ParseError::custom(()));
                    }
                    input.expect_exhausted()?;
                    Ok(())
                })?;
            }
            // These can turn a string or a substituted variable into a resource URL.
            // Use literal url(data:...) instead, which has a captured byte source.
            Token::Function(name)
                if ["image-set", "-webkit-image-set", "image", "src", "attr", "local"]
                    .iter()
                    .any(|candidate| name.eq_ignore_ascii_case(candidate)) =>
            {
                return Err(cssparser::ParseError::custom(()));
            }
            Token::Function(_) | Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock => {
                input.parse_nested_block(|input| scan(input, depth.saturating_add(1), maximum))?;
            }
            Token::BadUrl(_) | Token::BadString(_) | Token::CloseParenthesis | Token::CloseSquareBracket | Token::CloseCurlyBracket => {
                return Err(cssparser::ParseError::custom(()));
            }
            _ => {}
        }
    }
    Ok(())
}

fn portable_url(url: &str) -> bool {
    url.starts_with('#') || url.get(..5).is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
}

fn validate_logo(url: &str, maximum: u64) -> Result<()> {
    let (media, payload) = url
        .strip_prefix("data:")
        .and_then(|value| value.split_once(";base64,"))
        .ok_or_else(|| invalid("captured logo requires an embedded image data URL"))?;
    if !matches!(
        media,
        "image/png" | "image/jpeg" | "image/gif" | "image/webp" | "image/svg+xml" | "image/x-icon" | "image/vnd.microsoft.icon"
    ) {
        return Err(invalid("captured logo MIME type is unsupported"));
    }
    if payload.is_empty() || payload.len() % 4 != 0 || payload.len() as u128 > u128::from(maximum).div_ceil(3) * 4 {
        return Err(invalid("captured logo exceeds its original byte limit"));
    }
    let mut length = 0usize;
    let mut decoded = [0u8; 3072];
    let chunks = payload.as_bytes().chunks(4096);
    let count = chunks.len();
    for (index, chunk) in chunks.enumerate() {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode_slice(chunk, &mut decoded)
            .map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
        if bytes != chunk.len() / 4 * 3 && index + 1 != count {
            return Err(invalid("captured logo base64 padding precedes the end"));
        }
        length += bytes;
    }
    if length == 0 || length as u64 > maximum {
        return Err(invalid("captured logo exceeds its original byte limit"));
    }
    Ok(())
}

fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{validate_css, validate_logo};

    #[test]
    fn logo_admission_uses_captured_bytes() {
        let logo = "data:image/png;base64,YWJj";
        assert!(validate_logo(logo, 2).is_err());
        assert!(validate_logo(logo, 3).is_ok());
        assert!(validate_logo("data:image/png;base64,YQ==", 1).is_ok());
    }

    #[test]
    fn logo_validation_keeps_chunk_and_padding_bounds() {
        use base64::Engine;
        let bytes = vec![42; 3073];
        let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let logo = format!("data:image/png;base64,{encoded}");
        assert!(validate_logo(&logo, 3073).is_ok());
        assert!(validate_logo(&logo, 3072).is_err());
        let invalid = format!("data:image/png;base64,{}YQ==YWJj", "YWJj".repeat(1023));
        assert!(validate_logo(&invalid, 4096).is_err());
        assert!(validate_logo("data:image/png;base64,YR==", 1).is_err());
    }

    #[test]
    fn css_admission_uses_captured_depth() {
        assert!(validate_css("calc(calc(1))", 1).is_err());
        assert!(validate_css("calc(calc(1))", 2).is_ok());
        assert!(validate_css("url(https://example.invalid/image)", 2).is_err());
    }
}
