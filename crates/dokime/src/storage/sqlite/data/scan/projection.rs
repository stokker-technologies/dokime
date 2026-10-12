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
use std::{mem::size_of, num::NonZeroUsize, sync::Arc, time::UNIX_EPOCH};
// Third Party Imports
use harmos::{
    MemorySize,
    storage::{frame::*, reduce::*, *},
};
// Local Imports
use super::native;
use crate::storage::sqlite::{Error, memory};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
pub(super) enum Projection<'a> {
    Native {
        fields: &'a [ScalarField],
        kind: native::Kind,
    },
    Postcard {
        projection: ScalarProjection,
        fields: usize,
    },
}

/* === Traits === */

/* === Implementations === */
impl<'a> Projection<'a> {
    pub(super) fn new(
        schema: &StorageSchema,
        fields: &'a [ScalarField],
        bytes: usize,
    ) -> Result<Self, Error> {
        harmos::schema::validate(&schema.fields).map_err(|error| {
            harmos::Error::backend(
                "validate scan schema",
                harmos::BackendFailure::Corrupt,
                error,
            )
        })?;
        if let Some(kind) = native::Kind::for_schema(&schema.key) {
            kind.check_schema(&schema.fields)?;
            for field in fields {
                kind.check_field(field, &schema.fields)?;
            }
            return Ok(Self::Native { fields, kind });
        }
        Ok(Self::Postcard {
            projection: ScalarProjection::new(&schema.fields, fields.to_vec(), bound(bytes)?)?,
            fields: fields.len(),
        })
    }
    pub(super) fn output_memory(
        &self,
        record: &StoredRecord,
        stream: Option<&harmos::Identifier>,
    ) -> Result<usize, Error> {
        let fields = match self {
            Self::Native { fields, .. } => fields.len(),
            Self::Postcard { fields, .. } => *fields,
        };
        let metadata = record
            .item
            .as_str()
            .len()
            .saturating_add(
                record
                    .scope
                    .as_ref()
                    .map_or(0, |scope| scope.as_str().len()),
            )
            .saturating_add(stream.map_or(0, |stream| stream.as_str().len()));
        record
            .payload
            .len()
            .checked_add(metadata)
            .and_then(|bytes| bytes.checked_mul(fields))
            .and_then(|bytes| {
                bytes.checked_add(fields * size_of::<ScalarValue>() + size_of::<Vec<ScalarValue>>())
            })
            .ok_or_else(|| exhausted("SQLite scalar projection output"))
    }
    pub(super) fn project(
        &self,
        record: &StoredRecord,
        stream: Option<&harmos::Identifier>,
        memory: &Arc<memory::Budget>,
        bytes: NonZeroUsize,
        maximum: NonZeroUsize,
    ) -> Result<Vec<ScalarValue>, Error> {
        match self {
            Self::Postcard { projection, .. } => Ok(projection.project(
                &record.payload,
                |path, allowance| metadata(record, stream, path, allowance).map_err(runtime),
                bytes,
            )?),
            Self::Native { fields, kind } => {
                let row = kind.decode(&record.payload, memory, maximum)?;
                let mut values = Vec::new();
                values
                    .try_reserve_exact(fields.len())
                    .map_err(|_| exhausted("SQLite native projection slots"))?;
                if values.capacity() > fields.len() {
                    return Err(exhausted("SQLite native projection capacity"));
                }
                let mut used =
                    size_of::<Vec<ScalarValue>>() + fields.len() * size_of::<ScalarValue>();
                for field in *fields {
                    let allowance = bytes
                        .get()
                        .checked_sub(used)
                        .and_then(NonZeroUsize::new)
                        .ok_or_else(|| exhausted("SQLite native scalar output"))?;
                    let value = match &field.path {
                        ScalarPath::Payload(path) => row.scalar(path, field.kind, allowance)?,
                        path => metadata(record, stream, path, allowance)?,
                    };
                    value.check()?;
                    if value.kind().is_some_and(|kind| kind != field.kind) {
                        return Err(Error::Data(
                            "native scalar kind differs from requested field",
                        ));
                    }
                    used = used
                        .checked_add(value.memory_size().saturating_sub(size_of::<ScalarValue>()))
                        .filter(|used| *used <= bytes.get())
                        .ok_or_else(|| exhausted("SQLite native scalar output"))?;
                    values.push(value);
                }
                Ok(values)
            }
        }
    }
}

/* === Public Functions === */
pub(super) fn admission(selection: &ReadSelection, fields: &[ScalarField]) -> harmos::Result<()> {
    let key = selection
        .schema
        .as_ref()
        .ok_or(harmos::Error::InvalidInput {
            field: "scan schema",
            reason: "schema is required",
        })?;
    if let Some(kind) = native::Kind::for_schema(key) {
        for field in fields {
            kind.check_field(field, kind.fields()).map_err(runtime)?;
        }
        return Ok(());
    }
    if key.payload_encoding != PayloadEncoding::PostcardV1 {
        return Err(harmos::Error::InvalidInput {
            field: "scan schema",
            reason: "unsupported native codec; register a typed storage projection",
        });
    }
    Ok(())
}

pub(super) fn construction_memory(
    schema: &StorageSchema,
    fields: &[ScalarField],
) -> Result<usize, Error> {
    let validation = harmos::schema::validation_memory_bytes(&schema.fields).map_err(|error| {
        harmos::Error::backend("budget scan schema", harmos::BackendFailure::Corrupt, error)
    })?;
    let decoder = if native::Kind::for_schema(&schema.key).is_some() {
        0
    } else {
        harmos::schema::Validator::construction_memory_bytes(&schema.fields).map_err(|error| {
            harmos::Error::backend(
                "budget scan projection",
                harmos::BackendFailure::Corrupt,
                error,
            )
        })?
    };
    validation
        .checked_add(decoder)
        .and_then(|bytes| {
            bytes.checked_add(
                fields
                    .iter()
                    .fold(size_of::<Vec<ScalarField>>(), |bytes, field| {
                        bytes.saturating_add(field.memory_size())
                    }),
            )
        })
        .and_then(|bytes| bytes.checked_add(size_of::<Projection<'_>>() + 1024))
        .ok_or_else(|| exhausted("SQLite scalar projection construction"))
}

/* === Private Functions === */
fn metadata(
    record: &StoredRecord,
    stream: Option<&harmos::Identifier>,
    path: &ScalarPath,
    allowance: NonZeroUsize,
) -> Result<ScalarValue, Error> {
    Ok(match path {
        ScalarPath::SourceTime => match record.source_time {
            Some(time) => ScalarValue::Datetime(match time.duration_since(UNIX_EPOCH) {
                Ok(duration) => duration.as_nanos() as i128,
                Err(error) => -(error.duration().as_nanos() as i128),
            }),
            None => ScalarValue::Missing,
        },
        ScalarPath::CaptureOrder => ScalarValue::Unsigned(record.position.sequence),
        ScalarPath::Scope => match &record.scope {
            Some(scope) => text(scope.as_str(), allowance)?,
            None => ScalarValue::Missing,
        },
        ScalarPath::Stream => match stream {
            Some(stream) => text(stream.as_str(), allowance)?,
            None => ScalarValue::Missing,
        },
        ScalarPath::Item => text(record.item.as_str(), allowance)?,
        ScalarPath::Payload(_) => {
            return Err(Error::Data("metadata projection received payload path"));
        }
    })
}
fn text(value: &str, allowance: NonZeroUsize) -> Result<ScalarValue, Error> {
    if value.len() > allowance.get() {
        return Err(exhausted("SQLite metadata text"));
    }
    let mut text = String::new();
    text.try_reserve_exact(value.len())
        .map_err(|_| exhausted("SQLite metadata text allocation"))?;
    if text.capacity() > value.len() {
        return Err(exhausted("SQLite metadata text capacity"));
    }
    text.push_str(value);
    Ok(ScalarValue::Text(text))
}
fn bound(bytes: usize) -> Result<NonZeroUsize, Error> {
    NonZeroUsize::new(bytes)
        .filter(|bytes| bytes.get() != usize::MAX)
        .ok_or_else(|| exhausted("SQLite scalar projection memory"))
}
fn exhausted(resource: &'static str) -> Error {
    harmos::Error::ResourceExhausted { resource }.into()
}
fn runtime(error: Error) -> harmos::Error {
    match error {
        Error::Runtime(error) => error,
        other => harmos::Error::backend(
            "project SQLite scalar",
            harmos::BackendFailure::Permanent,
            other,
        ),
    }
}

/* === Tests === */
