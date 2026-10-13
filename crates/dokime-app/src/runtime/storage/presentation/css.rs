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
use cssparser::{Parser, Token};
// Local Imports
use super::error;
use crate::{Result, installed_plugins::RuntimeAppearanceFrame};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn validate(appearance: &RuntimeAppearanceFrame) -> Result<()> {
    for theme in &appearance.themes {
        let radii = &theme.corner_radii;
        let values = theme
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
            );
        for value in values {
            validate_css(value)
                .map_err(|reason| error(format!("captured theme {}: {reason}", theme.theme_id)))?;
        }
    }
    Ok(())
}

/* === Private Functions === */
fn validate_css(css: &str) -> std::result::Result<(), &'static str> {
    let mut input = Parser::new(css);
    scan(&mut input, 0).map_err(|_| "CSS must use embedded data URLs or document fragments; imports, dynamic resource functions and malformed tokens are unsupported")
}

fn scan<'i>(
    input: &mut Parser<'i>,
    depth: usize,
) -> std::result::Result<(), cssparser::ParseError<()>> {
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
                if [
                    "image-set",
                    "-webkit-image-set",
                    "image",
                    "src",
                    "attr",
                    "local",
                ]
                .iter()
                .any(|candidate| name.eq_ignore_ascii_case(candidate)) =>
            {
                return Err(cssparser::ParseError::custom(()));
            }
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                input.parse_nested_block(|input| scan(input, depth + 1))?;
            }
            Token::BadUrl(_)
            | Token::BadString(_)
            | Token::CloseParenthesis
            | Token::CloseSquareBracket
            | Token::CloseCurlyBracket => return Err(cssparser::ParseError::custom(())),
            _ => {}
        }
    }
    Ok(())
}

fn portable_url(url: &str) -> bool {
    url.starts_with('#')
        || url
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
}

/* === Tests === */
