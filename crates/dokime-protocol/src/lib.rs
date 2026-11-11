/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! Dokime contracts and the vendored Harmos transport, independent of the host runtime.

/* === Modules === */

// Checked-In Output of the Generator Exercised by the Integration Drift Check.
#[rustfmt::skip]
pub mod v1;

pub mod harmos {
    #[rustfmt::skip]
    pub mod v1;
}

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl v1::PlotKind {
    /// Parse a declared kind, refusing retired spellings with authoring guidance.
    pub fn parse_name(name: &str) -> Result<Self, String> {
        let short = name.strip_prefix("PLOT_KIND_").unwrap_or(name);
        if let Some(replacement) = retired_plot_kind(short) {
            return Err(format!("{short}: use {replacement}"));
        }
        Self::from_str_name(&format!("PLOT_KIND_{short}"))
            .filter(|kind| *kind != Self::Unspecified)
            .ok_or_else(|| format!("unknown plot kind {name}"))
    }

    /// Validate wire numbers without reinterpreting a retired enum tag.
    pub fn from_declared_number(number: i32) -> Result<Self, String> {
        let retired = match number {
            3 => Some("GROUPED_BAR"),
            4 => Some("STACKED_BAR"),
            6 => Some("DONUT"),
            7 => Some("PARETO"),
            9 => Some("XY_INTERVAL"),
            11 => Some("HEATMAP"),
            12 => Some("XY_VALUE_HEATMAP"),
            13 => Some("WIFI_CHANNEL_OCCUPANCY"),
            16 => Some("BOX_SUMMARY"),
            17 => Some("AGGREGATE_TABLE"),
            _ => None,
        };
        if let Some(name) = retired {
            return Err(format!("{name}: use {}", retired_plot_kind(name).expect("retired kind has a replacement")));
        }
        Self::try_from(number).ok().filter(|kind| *kind != Self::Unspecified).ok_or_else(|| format!("unknown plot kind number {number}"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn retired_plot_kind(name: &str) -> Option<&'static str> {
    Some(match name {
        "GROUPED_BAR" => "BAR with arrangement=grouped",
        "STACKED_BAR" => "BAR with arrangement=stacked",
        "DONUT" => "PIE with hole=0.5",
        "PARETO" => "BAR with pareto=true",
        "XY_INTERVAL" => "XY with interval=sigma",
        "HEATMAP" | "XY_VALUE_HEATMAP" => "CONTOUR",
        "WIFI_CHANNEL_OCCUPANCY" => "GRID_CHART with declared rows, columns and frames",
        "BOX_SUMMARY" => "BOX_PLOT",
        "AGGREGATE_TABLE" => "a document Summary card",
        _ => return None,
    })
}

/* === Tests === */
