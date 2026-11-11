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
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::LazyLock;
use std::sync::RwLock;

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

// A Field's Contribution to the Storage Schema: Identifier, Value Type, Nullable, Required.
type FieldShape = (Identifier, ValueType, bool, bool);
type CachedRevision = (Vec<FieldShape>, u32);
type RevisionCache = HashMap<u64, Vec<CachedRevision>>;

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordContract {
    pub record_type: Identifier,
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
    pub fn storage_schema(&self) -> harmos::schema::graph::Schema {
        use harmos::schema::{
            Cardinality, Described,
            graph::{Field, Kind, Schema},
        };

        let settings = harmos::config::SchemaSettings::default();
        let dynamic = Schema::from_declaration(Value::DEFINITION, &settings).expect("finite dynamic value schema");
        let mut definitions = vec![Vec::new()];
        let mut representations = std::collections::BTreeMap::new();
        let mut fields: Vec<_> = self.fields.iter().collect();
        fields.sort_by(|left, right| left.metadata.identifier.cmp(&right.metadata.identifier));
        for (index, field) in fields.into_iter().enumerate() {
            let kind = match field.value_type {
                ValueType::Boolean => Kind::Bool,
                ValueType::Integer => Kind::I64,
                ValueType::Number => Kind::F64,
                ValueType::Text | ValueType::File | ValueType::Datetime => Kind::String,
                ValueType::Object | ValueType::Array => {
                    let offset = definitions.len();
                    representations.extend(dynamic.representations.iter().map(|(index, value)| (index + offset, value.clone())));
                    for fields in &dynamic.definitions {
                        definitions.push(
                            fields
                                .iter()
                                .cloned()
                                .map(|mut field| {
                                    if let Kind::Message { definition } = &mut field.kind {
                                        *definition += offset;
                                    }
                                    field
                                })
                                .collect(),
                        );
                    }
                    Kind::Message { definition: offset + dynamic.root }
                }
            };
            let definition = definitions.len();
            let value_type = serde_json::to_value(&field.value_type).expect("record value type serializes");
            definitions.push(vec![Field {
                name: value_type.as_str().expect("record value type is named").to_owned().into(),
                tag: 1,
                kind,
                cardinality: if field.nullable { Cardinality::Optional } else { Cardinality::Required },
            }]);
            let kind = Kind::Message { definition };
            definitions[0].push(Field {
                name: field.metadata.identifier.to_string().into(),
                tag: u32::try_from(index + 1).expect("record field count fits u32"),
                kind,
                cardinality: if field.required { Cardinality::Required } else { Cardinality::Optional },
            });
        }
        Schema { root: 0, definitions, representations }
    }

    // Derives The Revision Of This Contract's Storage Schema Once Per Distinct Field Shape.
    pub fn schema_version(&self) -> u32 {
        static REVISIONS: LazyLock<RwLock<RevisionCache>> = LazyLock::new(Default::default);

        let key = self.shape_hash();
        let cached = REVISIONS
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&key)
            .and_then(|shapes| shapes.iter().find(|(shape, _)| self.has_shape(shape)).map(|(_, revision)| *revision));
        if let Some(revision) = cached {
            return revision;
        }
        let revision = self.derived_schema_version();
        let shape = self
            .fields
            .iter()
            .map(|field| (field.metadata.identifier.clone(), field.value_type.clone(), field.nullable, field.required))
            .collect();
        let mut revisions = REVISIONS.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        let shapes = revisions.entry(key).or_default();
        if !shapes.iter().any(|(shape, _)| self.has_shape(shape)) {
            shapes.push((shape, revision));
        }
        revision
    }

    fn derived_schema_version(&self) -> u32 {
        harmos::schema::identity::revision(&self.storage_schema()).expect("finite record schema")
    }

    fn shape_hash(&self) -> u64 {
        let mut hasher = std::hash::DefaultHasher::new();
        self.fields.len().hash(&mut hasher);
        for field in &self.fields {
            field.metadata.identifier.hash(&mut hasher);
            std::mem::discriminant(&field.value_type).hash(&mut hasher);
            field.nullable.hash(&mut hasher);
            field.required.hash(&mut hasher);
        }
        hasher.finish()
    }

    fn has_shape(&self, shape: &[FieldShape]) -> bool {
        shape.len() == self.fields.len()
            && shape.iter().zip(&self.fields).all(|((identifier, value_type, nullable, required), field)| {
                *identifier == field.metadata.identifier
                    && *value_type == field.value_type
                    && *nullable == field.nullable
                    && *required == field.required
            })
    }

    pub fn declared(record_type: impl Into<String>, label: impl Into<String>) -> Self {
        let record_type = Identifier::declared(record_type, "record");
        Self {
            metadata: Metadata::new(record_type.clone(), label, None),
            record_type,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_shape_identity_tracks_fields_without_presentation_or_order() {
        let original = RecordContract::declared("fixture.reading", "Reading")
            .field(RecordField::new(Metadata::new(Identifier::new("value").unwrap(), "Value", None), ValueType::Number));
        let mut relabeled = original.clone();
        relabeled.metadata.label = "Changed Label".into();
        relabeled.fields[0].metadata.label = "Changed Field Label".into();
        assert_eq!(original.schema_version(), relabeled.schema_version());
        let changed =
            original.clone().field(RecordField::new(Metadata::new(Identifier::new("added").unwrap(), "Added", None), ValueType::Integer));
        assert_ne!(original.schema_version(), changed.schema_version());
        let mut reordered = changed.clone();
        reordered.fields.reverse();
        assert_eq!(changed.schema_version(), reordered.schema_version());
        assert_eq!(crate::model::records::BuiltinRecordType::ToolConnectionAcquired.schema_version(), 3948733078);
    }

    #[test]
    fn cached_schema_version_equals_the_derived_revision_and_follows_shape_changes() {
        let base = RecordContract::declared("fixture.cached", "Cached")
            .field(RecordField::new(Metadata::new(Identifier::new("value").unwrap(), "Value", None), ValueType::Number));
        let mut retyped = base.clone();
        retyped.fields[0].value_type = ValueType::Integer;
        let mut nullable = base.clone();
        nullable.fields[0].nullable = !nullable.fields[0].nullable;
        let mut optional = base.clone();
        optional.fields[0].required = !optional.fields[0].required;
        let extended =
            base.clone().field(RecordField::new(Metadata::new(Identifier::new("label").unwrap(), "Label", None), ValueType::Text));
        let contracts = [
            base.clone(),
            retyped,
            nullable,
            optional,
            extended,
            crate::model::records::BuiltinRecordType::ToolConnectionAcquired.contract(),
        ];
        for contract in &contracts {
            assert_eq!(contract.schema_version(), contract.derived_schema_version());
            assert_eq!(contract.schema_version(), contract.derived_schema_version());
        }
        for (index, contract) in contracts[1..5].iter().enumerate() {
            assert_ne!(contract.schema_version(), base.schema_version(), "shape change {index} keeps the base revision");
        }
    }
}
