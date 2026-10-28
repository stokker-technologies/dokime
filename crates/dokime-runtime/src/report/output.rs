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
pub(super) static REPORT_PARTS: LazyLock<(&str, &str)> = LazyLock::new(|| split_template(super::asset::REPORT_APP_TEMPLATE));

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

pub(super) fn write_html(writer: &mut dyn Write, payload: &impl Serialize, template: &str) -> io::Result<()> {
    write_html_with(writer, template, |writer| write_json(writer, payload))
}

pub(super) fn write_json(writer: &mut dyn Write, payload: &impl Serialize) -> io::Result<()> {
    let mut serializer = serde_json::Serializer::with_formatter(writer, ScriptFormatter);
    payload.serialize(&mut serializer).map_err(io::Error::other)
}

pub(super) fn write_html_with(
    writer: &mut dyn Write, template: &str, payload: impl FnOnce(&mut dyn Write) -> io::Result<()>,
) -> io::Result<()> {
    // Reuse The Fixed Embedded Template; Other Inputs Keep Their Own Split.
    let (prefix, suffix) = if std::ptr::eq(template, super::asset::REPORT_APP_TEMPLATE) { *REPORT_PARTS } else { split_template(template) };
    writer.write_all(prefix.as_bytes())?;
    payload(writer)?;
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
