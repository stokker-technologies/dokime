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

const MAX_LOGO_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionContext {
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
pub struct ExecutionCycleContext {
    pub cycle: CycleView,
    pub cases: Vec<TestCase>,
    pub execution_order: Vec<TestExecutionId>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos::MemorySize for ExecutionContext {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.scope), std::mem::size_of_val(&self.scope)),
            (harmos::MemorySize::memory_size(&self.session_id), std::mem::size_of_val(&self.session_id)),
            (harmos::MemorySize::memory_size(&self.cycle), std::mem::size_of_val(&self.cycle)),
            (harmos::MemorySize::memory_size(&self.execution_documents), std::mem::size_of_val(&self.execution_documents)),
            (harmos::MemorySize::memory_size(&self.prompts), std::mem::size_of_val(&self.prompts)),
            (harmos::MemorySize::memory_size(&self.health), std::mem::size_of_val(&self.health)),
            (harmos::MemorySize::memory_size(&self.notes), std::mem::size_of_val(&self.notes)),
            (harmos::MemorySize::memory_size(&self.traceability), std::mem::size_of_val(&self.traceability)),
            (harmos::MemorySize::memory_size(&self.appearance), std::mem::size_of_val(&self.appearance)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for ExecutionCycleContext {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.cycle), std::mem::size_of_val(&self.cycle)),
            (harmos::MemorySize::memory_size(&self.cases), std::mem::size_of_val(&self.cases)),
            (harmos::MemorySize::memory_size(&self.execution_order), std::mem::size_of_val(&self.execution_order)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl ExecutionContext {
    pub fn validation_memory(&self) -> usize {
        harmos::btree_map_memory::<&dokime_domain::model::identity::NoteId, ()>(self.notes.len())
            .saturating_add(harmos::btree_map_memory::<&dokime_domain::model::identity::PromptId, ()>(self.prompts.len()))
            .saturating_add(harmos::btree_map_memory::<&dokime_domain::model::identity::HealthItemId, ()>(self.health.len()))
            .saturating_add(harmos::btree_map_memory::<&TestExecutionId, ()>(self.cycle.execution_order.len()))
            .saturating_add(harmos::btree_map_memory::<&dokime_domain::model::identity::CaseId, ()>(self.cycle.cases.len()))
            .saturating_add(appearance_validation_memory(
                self.appearance.themes.iter().flat_map(css_values),
                self.appearance.themes.iter().map(|theme| theme.theme_id.as_str().len()).max().unwrap_or(0),
            ))
    }

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
        validate_appearance(&self.appearance)
    }
}

/* === Public Functions === */

pub(crate) fn owns_projection(scope: &RecordScope, execution: &RecordScope) -> bool {
    (scope.test_cycle_id.is_some() || scope.test_execution_id.is_some())
        && scope.test_cycle_id.as_ref().is_none_or(|id| Some(id) == execution.test_cycle_id.as_ref())
        && scope.test_case_id.as_ref().is_none_or(|id| Some(id) == execution.test_case_id.as_ref())
        && scope.test_execution_id.as_ref().is_none_or(|id| Some(id) == execution.test_execution_id.as_ref())
}

pub(crate) fn owns_annotation(scope: &RecordScope, execution: &RecordScope) -> bool {
    if (scope.step_id.is_some() || scope.criterion_id.is_some()) && scope.test_execution_id.is_none() {
        return false;
    }
    (scope.test_cycle_id.is_none() && scope.test_case_id.is_none() && scope.test_execution_id.is_none())
        || owns_projection(scope, execution)
}

pub(crate) fn validate_appearance(appearance: &RuntimeAppearanceFrame) -> Result<()> {
    for theme in &appearance.themes {
        for value in css_values(theme) {
            validate_css(value).map_err(|reason| invalid(&format!("captured theme {}: {reason}", theme.theme_id)))?;
        }
    }
    for (index, logo) in appearance.logos.iter().enumerate() {
        if appearance.logos[..index].iter().any(|other| other.variant == logo.variant) {
            return Err(invalid("captured appearance has duplicate logo categories"));
        }
        validate_logo(&logo.url)?;
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

pub(crate) fn appearance_validation_memory<'a>(values: impl Iterator<Item = &'a str>, max_label_bytes: usize) -> usize {
    // Lexer Escape Buffers Grow Geometrically; Retained Nested Token Text Uses
    // Disjoint Source Slices. NUL And Zero Escapes Can Emit Three-Byte Replacements.
    let text = values
        .map(|value| {
            value.len().saturating_add(value.chars().filter(|c| *c == '\0' || *c == '0').count().saturating_mul(2)).saturating_mul(3).max(8)
        })
        .max()
        .unwrap_or(0);
    // The Scanner Refuses Depth Above 64. Rc<String> Stores Two Counts Plus Text.
    let tokens = 66usize.saturating_mul(std::mem::size_of::<String>() + 2 * std::mem::size_of::<usize>());
    let parsers = 66usize.saturating_mul(std::mem::size_of::<Parser<'_>>());
    let error = max_label_bytes.saturating_add(256).saturating_mul(3);
    text.saturating_add(tokens).saturating_add(parsers).saturating_add(error)
}

/* === Private Functions === */

fn validate_css(css: &str) -> std::result::Result<(), &'static str> {
    let mut input = Parser::new(css);
    scan(&mut input, 0).map_err(|_| "CSS must use embedded data URLs or document fragments; imports, dynamic resource functions and malformed tokens are unsupported")
}

fn scan<'i>(input: &mut Parser<'i>, depth: usize) -> std::result::Result<(), cssparser::ParseError<()>> {
    if depth > 64 {
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
                input.parse_nested_block(|input| scan(input, depth + 1))?;
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

fn validate_logo(url: &str) -> Result<()> {
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
    if payload.is_empty() || payload.len() % 4 != 0 || payload.len() > MAX_LOGO_BYTES.div_ceil(3) * 4 {
        return Err(invalid("captured logo exceeds its original byte limit"));
    }
    let mut length = 0usize;
    let mut decoded = [0u8; 3];
    let chunks = payload.as_bytes().as_chunks::<4>().0.iter();
    let count = chunks.len();
    for (index, chunk) in chunks.enumerate() {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode_slice(chunk, &mut decoded)
            .map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
        if bytes != 3 && index + 1 != count {
            return Err(invalid("captured logo base64 padding precedes the end"));
        }
        length += bytes;
    }
    if length == 0 || length > MAX_LOGO_BYTES {
        return Err(invalid("captured logo exceeds its original byte limit"));
    }
    Ok(())
}

fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

/* === Tests === */
