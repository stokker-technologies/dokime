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
use serde::{Deserialize, Serialize};

// Local Imports
use super::common::Metadata;
#[allow(unused_imports)]
use super::common::*;
use super::value::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementUnit {
    pub metadata: Option<Metadata>,
    pub family: UnitFamily,
    pub symbol: String,
}

// Native catalogs currently describe object/array fields only at the top
// level. Do not infer a nested schema when none was declared.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordField {
    pub metadata: Option<Metadata>,
    pub value_type: ValueType,
    pub required: bool,
    pub nullable: bool,
    pub unit: Option<MeasurementUnit>,
    pub choices: Vec<Value>,
    // Absent means no default; a present Value may explicitly be null.
    #[row(description = "Absent means no default; a present Value may explicitly be null.")]
    pub default_value: Option<Value>,
    pub format: Option<ValueFormat>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordSchema {
    pub record_type: String,
    pub schema_version: u32,
    pub metadata: Option<Metadata>,
    pub template_identifier: Option<String>,
    pub owner_suite_id: Option<String>,
    pub default_severity: Severity,
    pub fields: Vec<RecordField>,
    pub validation_mode: Option<ValidationMode>,
    pub allow_extra_fields: bool,
    pub display: Option<RecordDisplay>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordDisplay {
    pub icon: RecordIcon,
    pub tone: RecordTone,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryColumn {
    pub metadata: Option<Metadata>,
    pub value_type: ValueType,
    pub role: TelemetryRole,
    pub unit: Option<MeasurementUnit>,
    pub display_unit: Option<String>,
    pub required: bool,
    pub nullable: bool,
    pub default_value: Option<Value>,
    pub format: Option<ValueFormat>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySchema {
    pub stream: String,
    pub schema_version: u32,
    pub storage_schema: Option<harmos::schema::graph::Schema>,
    pub metadata: Option<Metadata>,
    pub template_identifier: Option<String>,
    pub owner_suite_id: Option<String>,
    pub columns: Vec<TelemetryColumn>,
    pub default_x: Option<String>,
    pub plot_window_ms: Option<u64>,
    pub allow_extra_fields: bool,
    pub scalar: bool,
    pub validation_mode: Option<ValidationMode>,
    // A declaration that reads a stream another plugin owns. The host checks
    // the declared columns are a subset of the owner's contract at admission.
    #[row(
        description = "A declaration that reads a stream another plugin owns. The host checks the declared columns are a subset of the owner's contract at admission."
    )]
    pub foreign: bool,
}

// How a declared value's number or datetime is rendered. The first five
// `kind`s format a number; the last three format a datetime. A number
// pattern is deliberately not offered: `normal`/`fixed`/`scientific`/
// `engineering`/`si_prefix` already cover every number a plot, a table cell
// or the report draws.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueFormat {
    pub kind: Option<ValueFormatKind>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalFormat {
    pub max_fraction_digits: Option<u32>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FixedFormat {
    pub decimals: u32,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ScientificFormat {
    pub significant_digits: u32,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineeringFormat {
    pub significant_digits: u32,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SiPrefixFormat {
    pub significant_digits: u32,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsoluteTimeFormat {
    pub pattern: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ElapsedFormat {
    pub unit: ElapsedUnit,
}

/* === Enums === */
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    Unspecified,
    Boolean,
    Integer,
    Number,
    Text,
    File,
    Object,
    Array,
    Datetime,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationMode {
    Unspecified,
    Strict,
    Prototype,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitFamily {
    Unspecified,
    Count,
    Distance,
    Duration,
    Frequency,
    Power,
    Temperature,
    Voltage,
    Ratio,
    Custom,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordIcon {
    Unspecified,
    Activity,
    AlertTriangle,
    Archive,
    Bell,
    CheckCircle,
    CircleDashed,
    ClipboardCheck,
    Cpu,
    FileText,
    Flag,
    Gauge,
    HeartPulse,
    HelpCircle,
    MessageSquareText,
    Package,
    Plug,
    RadioTower,
    RotateCcw,
    ShieldAlert,
    Target,
    TimerReset,
    Wrench,
    Zap,
    Clock,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordTone {
    Unspecified,
    Info,
    Success,
    Warning,
    Danger,
    Accent,
    Muted,
    Neutral,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetryRole {
    Unspecified,
    Value,
    Counter,
    State,
    Distance,
    Latency,
    Throughput,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElapsedUnit {
    Unspecified,
    Seconds,
    Milliseconds,
    Minutes,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueFormatKind {
    Normal(NormalFormat),
    Fixed(FixedFormat),
    Scientific(ScientificFormat),
    Engineering(EngineeringFormat),
    SiPrefix(SiPrefixFormat),
    AbsoluteTime(AbsoluteTimeFormat),
    ElapsedSinceRun(ElapsedFormat),
    ElapsedSinceCycle(ElapsedFormat),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
