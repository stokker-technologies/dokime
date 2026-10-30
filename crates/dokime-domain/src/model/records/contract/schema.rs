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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::identity::SuiteId;
use crate::model::identity::TemplateId;
use crate::model::records::RecordSeverity;
use crate::model::validation::ValidationMode;
use crate::model::values::MeasurementUnit;
use crate::model::values::Value;
use crate::model::values::ValueFormat;
use crate::model::values::ValueType;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordContract {
    pub record_type: Identifier,
    pub schema_version: u32,
    pub metadata: Metadata,

    #[serde(default)]
    pub display: RecordDisplay,

    pub template_id: Option<TemplateId>,
    pub owner: Option<SuiteId>,
    pub default_severity: RecordSeverity,
    pub fields: Vec<RecordField>,
    pub validation_mode: Option<ValidationMode>,
    pub allow_extra_fields: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordField {
    pub metadata: Metadata,
    pub value_type: ValueType,
    pub required: bool,
    pub nullable: bool,
    pub unit: Option<MeasurementUnit>,
    pub choices: Vec<Value>,
    pub default: Option<Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<ValueFormat>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordDisplay {
    #[serde(default)]
    pub icon: RecordIconKey,

    #[serde(default)]
    pub tone: RecordTone,
}

/* === Enums === */

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecordIconKey {
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

    #[default]
    Clock,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecordTone {
    Info,
    Success,
    Warning,
    Danger,
    Accent,
    Muted,

    #[default]
    Neutral,
}

/* === Traits === */

/* === Implementations === */

impl harmos::MemorySize for RecordContract {
    fn memory_size(&self) -> usize {
        retained_memory(
            std::mem::size_of::<Self>(),
            [
                (harmos::MemorySize::memory_size(&self.record_type), std::mem::size_of_val(&self.record_type)),
                (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
                (harmos::MemorySize::memory_size(&self.display), std::mem::size_of_val(&self.display)),
                (harmos::MemorySize::memory_size(&self.template_id), std::mem::size_of_val(&self.template_id)),
                (harmos::MemorySize::memory_size(&self.owner), std::mem::size_of_val(&self.owner)),
                (harmos::MemorySize::memory_size(&self.fields), std::mem::size_of_val(&self.fields)),
            ],
        )
    }
}

impl harmos::MemorySize for RecordField {
    fn memory_size(&self) -> usize {
        retained_memory(
            std::mem::size_of::<Self>(),
            [
                (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
                (harmos::MemorySize::memory_size(&self.unit), std::mem::size_of_val(&self.unit)),
                (harmos::MemorySize::memory_size(&self.choices), std::mem::size_of_val(&self.choices)),
                (harmos::MemorySize::memory_size(&self.default), std::mem::size_of_val(&self.default)),
                (harmos::MemorySize::memory_size(&self.format), std::mem::size_of_val(&self.format)),
            ],
        )
    }
}

impl harmos::MemorySize for RecordDisplay {
    fn memory_size(&self) -> usize {
        // All Icon Keys And Tones Are Closed Unit Variants With No Owned Heap.
        std::mem::size_of::<Self>()
    }
}

impl RecordContract {
    pub fn declared(record_type: impl Into<String>, label: impl Into<String>) -> Self {
        let record_type = Identifier::declared(record_type, "record");
        Self {
            metadata: Metadata::new(record_type.clone(), label, None),
            record_type,
            schema_version: 1,
            display: RecordDisplay::default(),
            template_id: None,
            owner: None,
            default_severity: RecordSeverity::Info,
            fields: Vec::new(),
            validation_mode: None,
            allow_extra_fields: true,
        }
    }

    pub fn described(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = Some(description.into());
        self
    }

    pub fn displayed(mut self, icon: RecordIconKey, tone: RecordTone) -> Self {
        self.display = RecordDisplay::new(icon, tone);
        self
    }

    pub fn severity(mut self, severity: RecordSeverity) -> Self {
        self.default_severity = severity;
        self
    }

    pub fn for_template(mut self, template_id: TemplateId) -> Self {
        self.template_id = Some(template_id);
        self
    }

    pub fn record_schema(&self) -> Self {
        let mut schema = self.clone();
        schema.template_id = None;
        schema
    }

    pub fn field(mut self, field: RecordField) -> Self {
        self.fields.push(field);
        self
    }

    pub fn strict(mut self) -> Self {
        self.validation_mode = Some(ValidationMode::Strict);
        self.allow_extra_fields = false;
        self
    }
}

impl RecordDisplay {
    pub fn new(icon: RecordIconKey, tone: RecordTone) -> Self {
        Self { icon, tone }
    }
}

impl RecordField {
    pub fn new(metadata: Metadata, value_type: ValueType) -> Self {
        Self {
            metadata,
            value_type,
            required: true,
            nullable: false,
            unit: None,
            choices: Vec::new(),
            default: None,
            format: None,
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self.required = false;
        self
    }

    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }

    pub fn with_choices(mut self, choices: impl IntoIterator<Item = Value>) -> Self {
        self.choices = choices.into_iter().collect();
        self
    }

    pub fn with_unit(mut self, unit: MeasurementUnit) -> Self {
        self.unit = Some(unit);
        self
    }

    pub fn with_format(mut self, format: ValueFormat) -> Self {
        self.format = Some(format);
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn retained_memory<const N: usize>(inline: usize, fields: [(usize, usize); N]) -> usize {
    fields.into_iter().fold(
        inline,
        |bytes, (owned, embedded)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(embedded)) }
        },
    )
}

/* === Tests === */
