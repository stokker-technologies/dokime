/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plots

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::model::{Metadata, Plot, PlotKind};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Plot {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>, kind: PlotKind) -> Self {
        Self {
            metadata: Some(Metadata::new(identifier, label)),
            kind: kind as i32,
            ..Default::default()
        }
    }

    pub fn line(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(identifier, label, PlotKind::TimeSeries)
    }

    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }
}

impl PlotKind {
    /// Parse a declared kind, refusing retired spellings with authoring guidance.
    pub fn parse_name(name: &str) -> Result<Self, String> {
        let short = name.strip_prefix("PLOT_KIND_").unwrap_or(name);
        if let Some(replacement) = retired_plot_kind(short) {
            return Err(format!("{short}: use {replacement}"));
        }
        let declared_name = format!("PLOT_KIND_{short}");
        let kind = Self::from_str_name(&declared_name).filter(|kind| *kind != Self::Unspecified);
        kind.ok_or_else(|| format!("use a declared plot kind instead of {name}"))
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
        let kind = Self::try_from(number).ok().filter(|kind| *kind != Self::Unspecified);
        kind.ok_or_else(|| format!("use a declared plot kind number instead of {number}"))
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
