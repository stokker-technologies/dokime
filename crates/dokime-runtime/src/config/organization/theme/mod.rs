/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod error;
mod source;

pub use error::{Error, Result};
pub use source::{Bundled, Directory, Source};

use std::{
    collections::BTreeMap,
    fmt::Write as _,
    path::{Component, Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, Parser, ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser,
    StyleSheetParser,
};

use crate::domains::{ThemeCornerRadii, ThemeVariant};

pub const TYPOGRAPHY_CSS_FILE: &str = "theme/typography.css";
pub const BASE_CSS_FILE: &str = "theme/base.css";
pub const LIGHT_CSS_FILE: &str = "theme/light.css";
pub const DARK_CSS_FILE: &str = "theme/dark.css";
const REQUIRED_TYPOGRAPHY_PROPERTIES: &[&str] = &[
    "brand-font-sans",
    "brand-font-heading",
    "brand-font-mono",
    "brand-text-xs",
    "brand-text-xl",
    "brand-text-2xl",
    "brand-text-3xl",
    "brand-text-4xl",
    "brand-text-5xl",
    "brand-text-6xl",
    "brand-text-7xl",
];
const REQUIRED_PALETTE_PROPERTIES: &[&str] = &[
    "background",
    "foreground",
    "card",
    "card-foreground",
    "popover",
    "popover-foreground",
    "primary",
    "primary-foreground",
    "secondary",
    "secondary-foreground",
    "muted",
    "muted-foreground",
    "accent",
    "accent-foreground",
    "destructive",
    "success",
    "success-foreground",
    "warning",
    "warning-foreground",
    "info",
    "info-foreground",
    "border",
    "input",
    "ring",
];
const REQUIRED_CHART_COLORS: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Authored {
    pub typography_css: String,
    pub base_css: String,
    pub light_css: String,
    pub dark_css: String,
    pub variables: BTreeMap<String, String>,
    pub light: ThemeVariant,
    pub dark: ThemeVariant,
}

struct StylesheetRuleParser;

struct CustomPropertyParser;

#[derive(Debug)]
enum ParsedRule {
    Import,
    AtRule,
    Style { selector: String, variables: BTreeMap<String, String> },
}

impl Authored {
    pub fn combined_css(&self, corner_radii: &ThemeCornerRadii) -> String {
        let mut output = String::from("/* GENERATED from the authored `organization/theme/` CSS.\n * Do not edit by hand. */\n");
        append_section(&mut output, TYPOGRAPHY_CSS_FILE, &self.typography_css);
        append_section(&mut output, BASE_CSS_FILE, &self.base_css);
        output.push_str("\n/* BEGIN generated corner radii from organization/organization.toml */\n:root {\n");
        render_corner_radii(&mut output, corner_radii);
        output.push_str("}\n/* END generated corner radii */\n");
        append_section(&mut output, LIGHT_CSS_FILE, &self.light_css);
        append_section(&mut output, DARK_CSS_FILE, &self.dark_css);
        output
    }
}

impl<'i> AtRuleParser<'i> for StylesheetRuleParser {
    type Prelude = bool;
    type AtRule = ParsedRule;
    type Error = String;

    fn parse_prelude(&mut self, name: CowRcStr<'i>, input: &mut Parser<'i>) -> Result<Self::Prelude, cssparser::ParseError<Self::Error>> {
        while input.next_including_whitespace_and_comments().is_ok() {}
        Ok(name.eq_ignore_ascii_case("import"))
    }

    fn rule_without_block(&mut self, is_import: Self::Prelude, _start: &ParserState) -> Result<Self::AtRule, ()> {
        Ok(if is_import { ParsedRule::Import } else { ParsedRule::AtRule })
    }

    fn parse_block(
        &mut self, is_import: Self::Prelude, _start: &ParserState, input: &mut Parser<'i>,
    ) -> Result<Self::AtRule, cssparser::ParseError<Self::Error>> {
        while input.next_including_whitespace_and_comments().is_ok() {}
        Ok(if is_import { ParsedRule::Import } else { ParsedRule::AtRule })
    }
}

impl<'i> QualifiedRuleParser<'i> for StylesheetRuleParser {
    type Prelude = String;
    type QualifiedRule = ParsedRule;
    type Error = String;

    fn parse_prelude(&mut self, input: &mut Parser<'i>) -> Result<Self::Prelude, cssparser::ParseError<Self::Error>> {
        let start = input.position();
        while input.next_including_whitespace_and_comments().is_ok() {}
        Ok(input.slice_from(start).trim().to_owned())
    }

    fn parse_block(
        &mut self, selector: Self::Prelude, _start: &ParserState, input: &mut Parser<'i>,
    ) -> Result<Self::QualifiedRule, cssparser::ParseError<Self::Error>> {
        let mut declaration_parser = CustomPropertyParser;
        let mut variables = BTreeMap::new();
        for result in RuleBodyParser::new(input, &mut declaration_parser) {
            match result {
                Ok(Some((name, value))) => {
                    if variables.insert(name.clone(), value).is_some() {
                        return Err(cssparser::ParseError::custom(format!("duplicate custom property --{name}")));
                    }
                }
                Ok(None) => {}
                Err((error, _, _)) => return Err(error),
            }
        }
        Ok(ParsedRule::Style { selector, variables })
    }
}

impl<'i> DeclarationParser<'i> for CustomPropertyParser {
    type Declaration = Option<(String, String)>;
    type Error = String;

    fn parse_value(
        &mut self, name: CowRcStr<'i>, input: &mut Parser<'i>, _start: &ParserState,
    ) -> Result<Self::Declaration, cssparser::ParseError<Self::Error>> {
        let start = input.position();
        while input.next_including_whitespace_and_comments().is_ok() {}
        let value = input.slice_from(start).trim().to_owned();
        if !name.starts_with("--") {
            return Ok(None);
        }
        let name = name.trim_start_matches("--").to_owned();
        if value.is_empty() || value.to_ascii_lowercase().ends_with("!important") {
            return Err(cssparser::ParseError::custom(format!("invalid custom property --{name}")));
        }
        Ok(Some((name, value)))
    }
}

impl<'i> AtRuleParser<'i> for CustomPropertyParser {
    type Prelude = ();
    type AtRule = Option<(String, String)>;
    type Error = String;
}

impl<'i> QualifiedRuleParser<'i> for CustomPropertyParser {
    type Prelude = ();
    type QualifiedRule = Option<(String, String)>;
    type Error = String;
}

impl<'i> RuleBodyItemParser<'i, Option<(String, String)>, String> for CustomPropertyParser {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        false
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn load(dir: &Path) -> Result<Authored, Error> {
    let source = Directory::new(dir).map_err(|source| Error::Read { path: dir.to_path_buf(), source })?;
    load_from(&source)
}

pub fn load_from(source: &impl Source) -> Result<Authored, Error> {
    let typography = inline_local_font_urls(source, &read_css(source, TYPOGRAPHY_CSS_FILE)?)?;
    let base = read_css(source, BASE_CSS_FILE)?;
    let light = read_css(source, LIGHT_CSS_FILE)?;
    let dark = read_css(source, DARK_CSS_FILE)?;
    parse(typography, base, light, dark)
}

pub fn parse(typography_css: String, base_css: String, light_css: String, dark_css: String) -> Result<Authored, Error> {
    let typography_rules = parse_stylesheet(TYPOGRAPHY_CSS_FILE, &typography_css)?;
    reject_imports(TYPOGRAPHY_CSS_FILE, &typography_rules)?;
    let base_rules = parse_stylesheet(BASE_CSS_FILE, &base_css)?;
    reject_imports(BASE_CSS_FILE, &base_rules)?;
    let light_rules = parse_stylesheet(LIGHT_CSS_FILE, &light_css)?;
    reject_imports(LIGHT_CSS_FILE, &light_rules)?;
    let dark_rules = parse_stylesheet(DARK_CSS_FILE, &dark_css)?;
    reject_imports(DARK_CSS_FILE, &dark_rules)?;

    let variables = required_rule_variables(TYPOGRAPHY_CSS_FILE, ":root", &typography_rules, REQUIRED_TYPOGRAPHY_PROPERTIES)?;
    let light_variables = required_rule_variables(LIGHT_CSS_FILE, ":root", &light_rules, REQUIRED_PALETTE_PROPERTIES)?;
    let dark_variables = required_rule_variables(DARK_CSS_FILE, ".dark", &dark_rules, REQUIRED_PALETTE_PROPERTIES)?;

    let light = theme_variant(LIGHT_CSS_FILE, ":root", light_variables)?;
    let dark = theme_variant(DARK_CSS_FILE, ".dark", dark_variables)?;
    if light.variables.keys().ne(dark.variables.keys()) {
        return Err(Error::ModePropertyMismatch);
    }

    Ok(Authored {
        typography_css: normalize_css(typography_css),
        base_css: normalize_css(base_css),
        light_css: normalize_css(light_css),
        dark_css: normalize_css(dark_css),
        variables,
        light,
        dark,
    })
}

fn reject_imports(file: &'static str, rules: &[ParsedRule]) -> Result<(), Error> {
    if rules.iter().any(|rule| matches!(rule, ParsedRule::Import)) {
        return Err(Error::Import { file });
    }
    Ok(())
}

fn read_css(source: &impl Source, relative: &'static str) -> Result<String, Error> {
    let error = |source| Error::Read { path: PathBuf::from(relative), source };
    let bytes = source.read(relative).map_err(error)?;
    String::from_utf8(bytes).map_err(|utf8| error(std::io::Error::new(std::io::ErrorKind::InvalidData, utf8)))
}

fn inline_local_font_urls(source: &impl Source, css: &str) -> Result<String, Error> {
    let mut output = String::with_capacity(css.len());
    let mut cursor = 0;
    while let Some(relative_start) = find_url_function(&css[cursor..]) {
        let start = cursor + relative_start;
        output.push_str(&css[cursor..start]);
        let value_start = start + 4;
        let end = find_url_end(css, value_start).ok_or_else(|| Error::Parse {
            file: TYPOGRAPHY_CSS_FILE,
            near: css[start..].chars().take(120).collect(),
            message: "unterminated url()".to_owned(),
        })?;
        let raw = css[value_start..end].trim();
        let url = raw.strip_prefix(['\'', '"']).and_then(|value| value.strip_suffix(['\'', '"'])).unwrap_or(raw).trim();
        if url.starts_with("data:") {
            output.push_str(&css[start..=end]);
        } else {
            let relative = Path::new(url);
            if url.starts_with("http:")
                || url.starts_with("https:")
                || relative.is_absolute()
                || relative
                    .components()
                    .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
            {
                return Err(Error::UnsafeUrl {
                    file: TYPOGRAPHY_CSS_FILE,
                    url: url.to_owned(),
                });
            }
            let mime = match relative.extension().and_then(|extension| extension.to_str()) {
                Some("woff2") => "font/woff2",
                Some("woff") => "font/woff",
                Some("ttf") => "font/ttf",
                Some("otf") => "font/otf",
                _ => {
                    return Err(Error::UnsupportedFont {
                        file: TYPOGRAPHY_CSS_FILE,
                        url: url.to_owned(),
                    });
                }
            };
            let bytes = source.read(url).map_err(|error| {
                if error.kind() == std::io::ErrorKind::PermissionDenied {
                    Error::UnsafeUrl {
                        file: TYPOGRAPHY_CSS_FILE,
                        url: url.to_owned(),
                    }
                } else {
                    Error::FontRead {
                        path: relative.to_path_buf(),
                        source: error,
                    }
                }
            })?;
            write!(output, "url(\"data:{mime};base64,{}\")", BASE64.encode(bytes)).expect("writing to a String cannot fail");
        }
        cursor = end + 1;
    }
    output.push_str(&css[cursor..]);
    Ok(output)
}

fn find_url_function(css: &str) -> Option<usize> {
    let bytes = css.as_bytes();
    let mut index = 0;
    let mut quote = None;
    while index + 4 <= bytes.len() {
        if quote.is_none() && bytes[index..].starts_with(b"/*") {
            index += 2;
            while index + 2 <= bytes.len() && !bytes[index..].starts_with(b"*/") {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
            continue;
        }
        match quote {
            Some(_) if bytes[index] == b'\\' => index += 2,
            Some(active) if bytes[index] == active => {
                quote = None;
                index += 1;
            }
            Some(_) => index += 1,
            None if matches!(bytes[index], b'\'' | b'"') => {
                quote = Some(bytes[index]);
                index += 1;
            }
            None if bytes[index..index + 4].eq_ignore_ascii_case(b"url(") => return Some(index),
            None => index += 1,
        }
    }
    None
}

fn find_url_end(css: &str, start: usize) -> Option<usize> {
    let bytes = css.as_bytes();
    let mut quote = None;
    let mut index = start;
    while index < bytes.len() {
        match quote {
            Some(_) if bytes[index] == b'\\' => index += 2,
            Some(active) if bytes[index] == active => {
                quote = None;
                index += 1;
            }
            Some(_) => index += 1,
            None if matches!(bytes[index], b'\'' | b'"') => {
                quote = Some(bytes[index]);
                index += 1;
            }
            None if bytes[index] == b')' => return Some(index),
            None => index += 1,
        }
    }
    None
}

fn normalize_css(mut css: String) -> String {
    while css.ends_with(['\n', '\r']) {
        css.pop();
    }
    css.push('\n');
    css
}

fn append_section(output: &mut String, path: &str, css: &str) {
    write!(output, "\n/* BEGIN authored {path} */\n{css}").expect("writing to a String cannot fail");
    if !css.ends_with('\n') {
        output.push('\n');
    }
    writeln!(output, "/* END authored {path} */").expect("writing to a String cannot fail");
}

fn render_corner_radii(output: &mut String, radii: &ThemeCornerRadii) {
    let entries = [
        ("", radii.base.as_deref()),
        ("-sm", radii.sm.as_deref()),
        ("-md", radii.md.as_deref()),
        ("-lg", radii.lg.as_deref()),
        ("-xl", radii.xl.as_deref()),
        ("-2xl", radii.two_xl.as_deref()),
        ("-3xl", radii.three_xl.as_deref()),
        ("-4xl", radii.four_xl.as_deref()),
    ];
    for (suffix, value) in entries {
        if let Some(value) = value {
            writeln!(output, "  --brand-radius{suffix}: {value};").expect("writing to a String cannot fail");
        }
    }
}

fn parse_stylesheet(file: &'static str, source: &str) -> Result<Vec<ParsedRule>, Error> {
    let mut parser = Parser::new(source);
    let mut rule_parser = StylesheetRuleParser;
    StyleSheetParser::new(&mut parser, &mut rule_parser)
        .map(|result| {
            result.map_err(|(error, near, _)| Error::Parse {
                file,
                near: near.chars().take(120).collect(),
                message: error.to_string(),
            })
        })
        .collect()
}

fn required_rule_variables(
    file: &'static str, selector: &'static str, rules: &[ParsedRule], required: &[&str],
) -> Result<BTreeMap<String, String>, Error> {
    let matches = rules
        .iter()
        .filter_map(|rule| match rule {
            ParsedRule::Style { selector: actual, variables } if actual == selector => Some(variables),
            _ => None,
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(Error::Selector { file, selector, count: matches.len() });
    }
    let variables = matches[0].clone();
    for property in required {
        if !variables.contains_key(*property) {
            return Err(Error::MissingProperty {
                file,
                selector,
                property: (*property).to_owned(),
            });
        }
    }
    Ok(variables)
}

fn theme_variant(file: &'static str, selector: &'static str, mut variables: BTreeMap<String, String>) -> Result<ThemeVariant, Error> {
    let mut plot_palette = Vec::with_capacity(REQUIRED_CHART_COLORS);
    for index in 1..=REQUIRED_CHART_COLORS {
        let property = format!("chart-{index}");
        let value = variables.remove(&property).ok_or(Error::MissingProperty { file, selector, property })?;
        plot_palette.push(value);
    }
    if let Some(property) = variables.keys().find(|name| name.starts_with("chart-")).cloned() {
        return Err(Error::UnsupportedChartProperty { file, selector, property });
    }
    Ok(ThemeVariant { variables, plot_palette })
}
