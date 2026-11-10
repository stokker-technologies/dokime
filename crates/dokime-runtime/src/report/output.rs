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
use std::{
    io::{self, Write},
    sync::LazyLock,
};

// Third Party Imports
use serde::Serialize;

// Local Imports
use dokime_domain::model::plugins::LogoVariant;

/* === Constants === */

const REPORT_DATA_MARKER: &str = r#"<script id="hub-report-data" type="application/json">"#;
const REPORT_DATA_END: &str = "</script>";
static REPORT_PARTS: LazyLock<(&str, &str)> = LazyLock::new(|| split_template(super::asset::REPORT_APP_TEMPLATE));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct ScriptFormatter;

pub(super) struct Object<'a> {
    writer: &'a mut dyn Write,
    first: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl serde_json::ser::Formatter for ScriptFormatter {
    fn write_string_fragment<W: ?Sized + Write>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()> {
        let mut start = 0;
        for (at, character) in fragment.char_indices() {
            let escaped = match character {
                '<' => b"\\u003c".as_slice(),
                '>' => b"\\u003e".as_slice(),
                '&' => b"\\u0026".as_slice(),
                '\u{2028}' => b"\\u2028".as_slice(),
                '\u{2029}' => b"\\u2029".as_slice(),
                _ => continue,
            };
            writer.write_all(&fragment.as_bytes()[start..at])?;
            writer.write_all(escaped)?;
            start = at + character.len_utf8();
        }
        writer.write_all(&fragment.as_bytes()[start..])
    }
}

impl<'a> Object<'a> {
    pub(super) fn new(writer: &'a mut dyn Write) -> io::Result<Self> {
        writer.write_all(b"{")?;
        Ok(Self { writer, first: true })
    }

    pub(super) fn field(&mut self, name: &str, value: &impl Serialize) -> io::Result<()> {
        self.raw(name, |writer| write_json(writer, value))
    }

    pub(super) fn raw(&mut self, name: &str, value: impl FnOnce(&mut dyn Write) -> io::Result<()>) -> io::Result<()> {
        if !self.first {
            self.writer.write_all(b",")?;
        }
        self.first = false;
        write_json(self.writer, &name)?;
        self.writer.write_all(b":")?;
        value(self.writer)
    }

    pub(super) fn finish(self) -> io::Result<()> {
        self.writer.write_all(b"}")
    }
}

/* === Public Functions === */

pub(super) fn write_html(
    writer: &mut dyn Write, payload: &impl Serialize, template: &str, appearance: &crate::installed_plugins::RuntimeAppearanceFrame,
) -> io::Result<()> {
    write_html_with(writer, template, appearance, |writer| write_json(writer, payload))
}

pub(super) fn write_json(writer: &mut dyn Write, payload: &impl Serialize) -> io::Result<()> {
    let mut serializer = serde_json::Serializer::with_formatter(writer, ScriptFormatter);
    payload.serialize(&mut serializer).map_err(io::Error::other)
}

pub(super) fn write_html_with(
    writer: &mut dyn Write, template: &str, appearance: &crate::installed_plugins::RuntimeAppearanceFrame,
    payload: impl FnOnce(&mut dyn Write) -> io::Result<()>,
) -> io::Result<()> {
    // Reuse The Fixed Embedded Template; Other Inputs Keep Their Own Split.
    let (prefix, suffix) = if std::ptr::eq(template, super::asset::REPORT_APP_TEMPLATE) { *REPORT_PARTS } else { split_template(template) };
    write_head(writer, prefix, appearance)?;
    payload(writer)?;
    writer.write_all(suffix.as_bytes())
}

/* === Private Functions === */

fn write_head(writer: &mut dyn Write, prefix: &str, appearance: &crate::installed_plugins::RuntimeAppearanceFrame) -> io::Result<()> {
    let generated = prefix.find("<!-- BEGIN GENERATED BRAND HEAD -->").and_then(|start| {
        let end = prefix[start..].find("<!-- END GENERATED BRAND HEAD -->")? + start + "<!-- END GENERATED BRAND HEAD -->".len();
        Some((start, end))
    });
    let (start, end) = generated.unwrap_or_else(|| {
        let end = prefix.find("</head>").unwrap_or(0);
        (end, end)
    });
    writer.write_all(prefix[..start].as_bytes())?;
    let label = html_attribute(&appearance.app_name);
    write!(writer, "<title>{label} Report</title>")?;
    for (contained, transparent, media) in [
        (LogoVariant::ContainedOnLight, LogoVariant::TransparentOnLight, "(prefers-color-scheme: light)"),
        (LogoVariant::ContainedOnDark, LogoVariant::TransparentOnDark, "(prefers-color-scheme: dark)"),
    ] {
        let logo = appearance
            .logos
            .iter()
            .find(|logo| logo.variant == contained)
            .or_else(|| appearance.logos.iter().find(|logo| logo.variant == transparent))
            .or_else(|| appearance.logos.first());
        if let Some(logo) = logo.filter(|logo| logo.url.starts_with("data:image/")) {
            let url = html_attribute(&logo.url);
            write!(writer, "<link rel=\"icon\" media=\"{media}\" href=\"{url}\">")?;
        }
    }
    writer.write_all(prefix[end..].as_bytes())
}

fn html_attribute(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('\"', "&quot;").replace('\'', "&#39;")
}

fn split_template(template: &str) -> (&str, &str) {
    template.find(REPORT_DATA_MARKER)
        .and_then(|marker_start| {
            let content_start = marker_start + REPORT_DATA_MARKER.len();
            let script_end = template[content_start..].find(REPORT_DATA_END)?;
            Some((&template[..content_start], &template[content_start + script_end..]))
        })
        .unwrap_or((
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"></head><body><div id=\"root\"></div><script id=\"hub-report-data\" type=\"application/json\">",
            "</script></body></html>",
        ))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn appearance() -> crate::installed_plugins::RuntimeAppearanceFrame {
        let fixture: serde_json::Value = serde_json::from_str(include_str!("fixtures/execution-report-input.json")).unwrap();
        let mut appearance: crate::installed_plugins::RuntimeAppearanceFrame =
            serde_json::from_value(fixture["input"]["appearance"].clone()).unwrap();
        appearance.app_name = r#"Test <Console> & "Lab""#.into();
        appearance.logos = serde_json::from_value(serde_json::json!([{
            "variant": "contained_on_light", "plugin_id": "organization", "asset_id": "brand.logo",
            "url": "data:image/svg+xml;base64,PHN2Zy8+", "alt": null
        }]))
        .unwrap();
        appearance
    }

    #[test]
    fn captured_head_precedes_payload() {
        let appearance = appearance();
        let template = "<html><head><!-- BEGIN GENERATED BRAND HEAD --><title>Old</title><link rel=\"icon\" href=\"old.svg\"><!-- END GENERATED BRAND HEAD --></head><body><script id=\"hub-report-data\" type=\"application/json\">{}</script></body></html>";
        for template in [template, super::super::asset::REPORT_APP_TEMPLATE] {
            let mut bytes = Vec::new();
            write_html(&mut bytes, &serde_json::json!({"captured": true}), template, &appearance).unwrap();
            let html = String::from_utf8(bytes).unwrap();
            let head = html.split_once("</head>").unwrap().0;
            assert!(head.contains("<title>Test &lt;Console&gt; &amp; &quot;Lab&quot; Report</title>"));
            assert!(head.contains(&appearance.logos[0].url));
            assert!(!head.contains("old.svg"));
            assert_eq!(head.matches("<title>").count(), 1);
            assert!(html.contains("{\"captured\":true}"));
        }
    }

    #[test]
    fn fallback_head_uses_captured_appearance() {
        let mut appearance = appearance();
        appearance.logos.clear();
        let mut bytes = Vec::new();
        write_html(&mut bytes, &serde_json::json!({}), "", &appearance).unwrap();
        let html = String::from_utf8(bytes).unwrap();
        assert!(html.contains("<title>Test &lt;Console&gt; &amp; &quot;Lab&quot; Report</title>"));
        assert!(!html.contains("rel=\"icon\""));
        assert!(html.starts_with("<!doctype html>"));
    }
}
