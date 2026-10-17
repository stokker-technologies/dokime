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
use std::fmt;
// Third Party Imports
use serde::Deserialize;
use serde::Serialize;
use serde::de;
// Local Imports

/* === Constants === */
const DOKIME_ID_NAMESPACE: [u8; 16] = [0xd0, 0x11, 0xed, 0x1d, 0x5e, 0xed, 0x4d, 0xa7, 0x9c, 0x0f, 0xe1, 0xd5, 0xea, 0x11, 0xab, 0x1e];

/* === Macros === */
macro_rules! typed_uuid {
    ($name:ident) => {
        typed_uuid!(@shape $name, preserve_case, from_uuid);
    };
    ($name:ident, lowercase) => {
        typed_uuid!(@shape $name, lower_uuid, from_uuid);
    };
    // An id whose wire form is either a UUID or one alternate spelling the
    // named validator accepts. `from_wire` tries both, and it is what
    // `Deserialize` routes through, so a stored alternate form decodes.
    ($name:ident, alt = $alternate:ident via $validator:ident, $refusal:expr) => {
        typed_uuid!(@shape $name, preserve_case, from_wire);

        impl $name {
            pub fn $alternate(value: impl Into<String>) -> crate::Result<Self> {
                let value = value.into();
                $validator(&value)?;
                Ok(Self(value))
            }

            pub fn from_wire(value: impl Into<String>) -> crate::Result<Self> {
                let value = value.into();
                if validate_uuid(stringify!($name), &value).is_ok() || $validator(&value).is_ok() {
                    return Ok(Self(value));
                }

                Err($refusal.into())
            }
        }
    };
    (@shape $name:ident, $format:ident, $wire:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl harmos_domain::MemorySize for $name {
            fn memory_size(&self) -> usize {
                harmos_domain::MemorySize::memory_size(&self.0)
            }
        }

        impl $name {
            pub fn from_seed(seed: impl AsRef<str>) -> Self {
                Self($format(seeded_uuid(stringify!($name), seed.as_ref())))
            }

            pub fn from_uuid(value: impl Into<String>) -> crate::Result<Self> {
                let value = value.into();
                validate_uuid(stringify!($name), &value)?;
                Ok(Self($format(value)))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = crate::Error;

            fn try_from(value: String) -> crate::Result<Self> {
                Self::$wire(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;

                Self::$wire(value).map_err(de::Error::custom)
            }
        }
    };
}

typed_uuid!(ArtifactId);

typed_uuid!(ActorId);

typed_uuid!(CommandId);

typed_uuid!(ContractId);

typed_uuid!(CorrelationId);

typed_uuid!(CriterionId);

typed_uuid!(RecordId);

typed_uuid!(TestExecutionId, lowercase);

typed_uuid!(HealthItemId);

typed_uuid!(JobRunId);

typed_uuid!(NoteId);

typed_uuid!(NodeId);

typed_uuid!(ParameterId);

typed_uuid!(SuiteId);

typed_uuid!(PromptId);

typed_uuid!(SessionId);

typed_uuid!(TestCycleId, lowercase);

typed_uuid!(StepId);

typed_uuid!(StepAttemptId);

typed_uuid!(TelemetryBatchId);

typed_uuid!(TelemetryStreamId);

typed_uuid!(TelemetrySampleId);

typed_uuid!(TemplateId);

typed_uuid!(TraceId);

typed_uuid!(TraceabilityId);

typed_uuid!(UploadId);

typed_uuid!(
    DocumentId,
    alt = from_identifier via validate_document_id,
    HubIdError::InvalidDocumentId
);

typed_uuid!(
    CaseId,
    alt = from_materialized via validate_materialized_case_id,
    HubIdError::InvalidCaseId
);

product_error!(HubIdError, formatter {
    Self::InvalidUuid { type_name } => write!(formatter, "{type_name} must be a valid UUID"),
    Self::InvalidCaseId => formatter.write_str("CaseId must be a valid UUID or materialized case id"),
    Self::InvalidDocumentId => formatter.write_str("DocumentId must be a valid UUID or document identifier"),
});

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HubIdError {
    InvalidUuid { type_name: &'static str },
    InvalidCaseId,
    InvalidDocumentId,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn seeded_uuid(type_name: &str, seed: &str) -> String {
    let mut left = fnv1a64(&DOKIME_ID_NAMESPACE, 0xcbf29ce484222325);
    left = fnv1a64(type_name.as_bytes(), left);
    left = fnv1a64(seed.as_bytes(), left);
    let mut right = fnv1a64(&DOKIME_ID_NAMESPACE, 0x84222325cbf29ce4);
    right = fnv1a64(seed.as_bytes(), right);
    right = fnv1a64(type_name.as_bytes(), right);
    format_uuid(left, right, 5)
}

/* === Private Functions === */
fn validate_uuid(type_name: &'static str, value: &str) -> Result<(), HubIdError> {
    let valid = value.len() == 36
        && value.bytes().enumerate().all(
            |(index, byte)| {
                if matches!(index, 8 | 13 | 18 | 23) { byte == b'-' } else { byte.is_ascii_hexdigit() }
            },
        );
    if valid { Ok(()) } else { Err(HubIdError::InvalidUuid { type_name }) }
}

fn fnv1a64(bytes: &[u8], seed: u64) -> u64 {
    bytes.iter().fold(seed, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3))
}

fn format_uuid(left: u64, right: u64, version: u8) -> String {
    let mut bytes = [0_u8; 16];
    bytes[..8].copy_from_slice(&left.to_be_bytes());
    bytes[8..].copy_from_slice(&right.to_be_bytes());
    bytes[6] = (bytes[6] & 0x0f) | (version << 4);
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let mut identifier = String::with_capacity(36);
    for (index, byte) in bytes.into_iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            identifier.push('-');
        }
        identifier.push(char::from(b"0123456789abcdef"[usize::from(byte >> 4)]));
        identifier.push(char::from(b"0123456789abcdef"[usize::from(byte & 0x0f)]));
    }
    identifier
}

fn preserve_case(value: String) -> String {
    value
}

fn lower_uuid(mut value: String) -> String {
    value.make_ascii_lowercase();
    value
}

fn validate_materialized_case_id(value: &str) -> Result<(), HubIdError> {
    let Some((template, digest)) = value.rsplit_once(':') else {
        return Err(HubIdError::InvalidCaseId);
    };
    let template_valid =
        !template.is_empty() && template.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'));
    let digest_valid = digest.len() == 10 && digest.chars().all(|character| character.is_ascii_hexdigit());

    if template_valid && digest_valid { Ok(()) } else { Err(HubIdError::InvalidCaseId) }
}

fn validate_document_id(value: &str) -> Result<(), HubIdError> {
    let valid =
        !value.is_empty() && value.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'));

    if valid { Ok(()) } else { Err(HubIdError::InvalidDocumentId) }
}

/* === Tests === */
