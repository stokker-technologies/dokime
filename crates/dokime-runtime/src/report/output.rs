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

/* === Constants === */

const REPORT_DATA_MARKER: &str = r#"<script id="hub-report-data" type="application/json">"#;
const REPORT_DATA_END: &str = "</script>";
static REPORT_PARTS: LazyLock<(&str, &str)> = LazyLock::new(|| split_template(super::asset::REPORT_APP_TEMPLATE));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct ScriptFormatter;

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

/* === Public Functions === */

pub(super) fn write_html(writer: &mut dyn Write, payload: &impl Serialize, template: &str) -> io::Result<()> {
    // Reuse The Fixed Embedded Template; Other Inputs Keep Their Own Split.
    let (prefix, suffix) = if std::ptr::eq(template, super::asset::REPORT_APP_TEMPLATE) { *REPORT_PARTS } else { split_template(template) };
    writer.write_all(prefix.as_bytes())?;
    let mut serializer = serde_json::Serializer::with_formatter(&mut *writer, ScriptFormatter);
    payload.serialize(&mut serializer).map_err(io::Error::other)?;
    writer.write_all(suffix.as_bytes())
}

/* === Private Functions === */

fn split_template(template: &str) -> (&str, &str) {
    template.find(REPORT_DATA_MARKER)
        .and_then(|marker_start| {
            let content_start = marker_start + REPORT_DATA_MARKER.len();
            let script_end = template[content_start..].find(REPORT_DATA_END)?;
            Some((&template[..content_start], &template[content_start + script_end..]))
        })
        .unwrap_or((
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Dokime Report</title></head><body><div id=\"root\"></div><script id=\"hub-report-data\" type=\"application/json\">",
            "</script></body></html>",
        ))
}

/* === Tests === */
