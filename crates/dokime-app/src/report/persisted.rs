/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Persisted Materialized Report Inputs

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::io::{self, Write};
use std::num::NonZeroUsize;
// Third Party Imports
use harmos::{
    MemorySize,
    storage::{FileReference, QueryValue, StorageCursor, StorageView},
};
use serde::{Deserialize, Serialize};
// Local Imports
use super::CapturedExecutionReportInput;
use crate::{Error, Result};
use dokime::core::TestExecutionId;
use dokime::execution::saved::{
    ExecutionSeal, MATERIALIZED_REPORT_INPUT_MEDIA_TYPE, MATERIALIZED_REPORT_INPUT_SCHEMA_VERSION,
    MATERIALIZED_REPORT_INPUT_SUFFIX,
};
use dokime::storage::Manager;

/* === Constants === */
pub(crate) const MEDIA_TYPE: &str = MATERIALIZED_REPORT_INPUT_MEDIA_TYPE;
pub(crate) const SCHEMA_VERSION: u32 = MATERIALIZED_REPORT_INPUT_SCHEMA_VERSION;
// A Format Constraint, Independent Of Consumer Memory Policy.
const MAX_DEPTH: usize = 128;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Serialize)]
struct EnvelopeRef<'a> {
    version: u32,
    captured: &'a CapturedExecutionReportInput,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u32,
    captured: CapturedExecutionReportInput,
}
struct Count(usize);
struct Exact<'a> {
    bytes: &'a mut Vec<u8>,
    length: usize,
}
struct Preflight<'a> {
    bytes: &'a [u8],
    offset: usize,
    memory: usize,
    typed_cell: usize,
}

/* === Enums === */
#[derive(Clone, Copy)]
enum Scope {
    Envelope,
    Captured,
    Input,
    Cases,
    Case,
    Typed,
    Dynamic,
}

/* === Traits === */

/* === Implementations === */
impl Write for Count {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("materialized input length overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Write for Exact<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let end = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .filter(|end| *end <= self.length && *end <= self.bytes.capacity())
            .ok_or_else(|| io::Error::other("materialized input exceeds counted length"))?;
        self.bytes.extend_from_slice(bytes);
        debug_assert_eq!(end, self.bytes.len());
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> Preflight<'a> {
    fn claim(&mut self, bytes: usize) -> Result<()> {
        self.memory = self
            .memory
            .checked_add(bytes)
            .ok_or_else(|| invalid("materialized input memory bound overflows"))?;
        Ok(())
    }

    fn value(&mut self, scope: Scope, depth: usize) -> Result<()> {
        if depth > MAX_DEPTH {
            return Err(invalid(
                "materialized input JSON nesting exceeds format bound",
            ));
        }
        self.space();
        self.claim(if matches!(scope, Scope::Dynamic) {
            128
        } else {
            self.typed_cell
        })?;
        match self.peek() {
            Some(b'{') => self.object(scope, depth + 1),
            Some(b'[') => self.array(scope, depth + 1),
            Some(b'"') => {
                self.string()?;
                Ok(())
            }
            Some(b't') => self.literal(b"true"),
            Some(b'f') => self.literal(b"false"),
            Some(b'n') => self.literal(b"null"),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(invalid("invalid materialized input JSON value")),
        }
    }

    fn object(&mut self, scope: Scope, depth: usize) -> Result<()> {
        self.offset += 1;
        self.space();
        if self.consume(b'}') {
            return Ok(());
        }
        loop {
            self.space();
            let key = self.string()?;
            self.space();
            if !self.consume(b':') {
                return Err(invalid("materialized input JSON object lacks a colon"));
            }
            // A Dynamic BTreeMap Node Can Allocate Its Full Key/Value Fanout On
            // The First Insertion. Charge That Capacity Before Serde Runs.
            if matches!(scope, Scope::Dynamic) {
                self.claim(1024)?;
            }
            let child = match (scope, key) {
                (Scope::Envelope, b"captured") => Scope::Captured,
                (Scope::Captured, b"input") => Scope::Input,
                (Scope::Input | Scope::Case, b"plots") | (Scope::Input, b"appearance") => {
                    Scope::Dynamic
                }
                (Scope::Input, b"case_documents") => Scope::Cases,
                (Scope::Dynamic, _) => Scope::Dynamic,
                _ => Scope::Typed,
            };
            self.value(child, depth)?;
            self.space();
            if self.consume(b'}') {
                return Ok(());
            }
            if !self.consume(b',') {
                return Err(invalid("materialized input JSON object lacks a separator"));
            }
        }
    }

    fn array(&mut self, scope: Scope, depth: usize) -> Result<()> {
        self.offset += 1;
        self.space();
        if self.consume(b']') {
            return Ok(());
        }
        let child = match scope {
            Scope::Cases => Scope::Case,
            scope => scope,
        };
        loop {
            self.value(child, depth)?;
            self.space();
            if self.consume(b']') {
                return Ok(());
            }
            if !self.consume(b',') {
                return Err(invalid("materialized input JSON array lacks a separator"));
            }
        }
    }

    fn string(&mut self) -> Result<&'a [u8]> {
        if !self.consume(b'"') {
            return Err(invalid("materialized input JSON key is not a string"));
        }
        let start = self.offset;
        loop {
            let byte = self
                .peek()
                .ok_or_else(|| invalid("truncated materialized input JSON string"))?;
            self.offset += 1;
            match byte {
                b'"' => {
                    let end = self.offset - 1;
                    self.claim(
                        (end - start)
                            .checked_mul(4)
                            .ok_or_else(|| invalid("materialized string bound overflows"))?,
                    )?;
                    return Ok(&self.bytes[start..end]);
                }
                b'\\' => {
                    let escape = self
                        .peek()
                        .ok_or_else(|| invalid("truncated materialized input JSON escape"))?;
                    self.offset += 1;
                    match escape {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {}
                        b'u' => {
                            for _ in 0..4 {
                                let digit = self.peek().ok_or_else(|| {
                                    invalid("truncated materialized Unicode escape")
                                })?;
                                if !digit.is_ascii_hexdigit() {
                                    return Err(invalid("invalid materialized Unicode escape"));
                                }
                                self.offset += 1;
                            }
                        }
                        _ => return Err(invalid("invalid materialized input JSON escape")),
                    }
                }
                0..=31 => return Err(invalid("control byte in materialized input JSON string")),
                _ => {}
            }
        }
    }

    fn number(&mut self) -> Result<()> {
        self.consume(b'-');
        if !self.consume(b'0') {
            if !matches!(self.peek(), Some(b'1'..=b'9')) {
                return Err(invalid("invalid materialized input JSON number"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.offset += 1;
            }
        }
        if self.consume(b'.') {
            self.digits()?;
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.offset += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.offset += 1;
            }
            self.digits()?;
        }
        Ok(())
    }

    fn digits(&mut self) -> Result<()> {
        let start = self.offset;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.offset += 1;
        }
        if self.offset == start {
            return Err(invalid("materialized input JSON number lacks digits"));
        }
        Ok(())
    }

    fn literal(&mut self, literal: &[u8]) -> Result<()> {
        let end = self
            .offset
            .checked_add(literal.len())
            .ok_or_else(|| invalid("materialized input length overflows"))?;
        if self.bytes.get(self.offset..end) != Some(literal) {
            return Err(invalid("invalid materialized input JSON literal"));
        }
        self.offset = end;
        Ok(())
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.offset).copied()
    }
    fn consume(&mut self, byte: u8) -> bool {
        if self.peek() != Some(byte) {
            return false;
        }
        self.offset += 1;
        true
    }
    fn space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.offset += 1;
        }
    }
}

/* === Public Functions === */
/// Count Output Before Its Owner Reserves The Exact Destination Buffer.
/// The Caller Holds Validation Scratch For The Already Admitted Capture.
pub(crate) fn encoded_length(captured: &CapturedExecutionReportInput) -> Result<usize> {
    captured.validate()?;
    let mut count = Count(0);
    serde_json::to_writer(
        &mut count,
        &EnvelopeRef {
            version: SCHEMA_VERSION,
            captured,
        },
    )
    .map_err(|error| invalid(&format!("cannot encode materialized report input: {error}")))?;
    Ok(count.0)
}

/// The Caller Holds Its Normal Storage Production Reservation For `length`.
pub(crate) fn encode(captured: &CapturedExecutionReportInput, length: usize) -> Result<Vec<u8>> {
    if encoded_length(captured)? != length {
        return Err(invalid("materialized report input changed after counting"));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| exhausted("materialized input encoding allocation"))?;
    if bytes.capacity() > length {
        return Err(invalid(
            "materialized input allocation exceeds exact preclaim",
        ));
    }
    let mut output = Exact {
        bytes: &mut bytes,
        length,
    };
    serde_json::to_writer(
        &mut output,
        &EnvelopeRef {
            version: SCHEMA_VERSION,
            captured,
        },
    )
    .map_err(|error| invalid(&format!("cannot encode materialized report input: {error}")))?;
    if bytes.len() != length {
        return Err(invalid("materialized report input encoded length differs"));
    }
    Ok(bytes)
}

pub(crate) async fn load(
    storage: &Manager,
    view: &StorageView,
    seal: &ExecutionSeal,
) -> Result<QueryValue<CapturedExecutionReportInput>> {
    if !seal
        .dependencies_saved
        .covers(&seal.materialized_input.position)?
    {
        return Err(invalid(
            "materialized report input exceeds sealed dependencies",
        ));
    }
    let captured = load_current(
        storage,
        view,
        &seal.materialized_input,
        &seal.source.execution.execution_id,
        &seal.source.evidence_through,
    )
    .await?;
    let source = crate::execution::context::read::read_file(view, &seal.source.context).await?;
    source.check_origin(storage)?;
    let memory = captured
        .memory_size()
        .checked_add(captured.validation_memory())
        .and_then(|bytes| bytes.checked_add(source.validation_memory()))
        .ok_or_else(|| invalid("materialized input validation memory overflows"))?;
    Ok(view.project_owned(captured, bound(memory)?, |captured, _| {
        source.validate_header().map_err(corrupt)?;
        if captured.owner_node_id != seal.source.owner_node_id
            || captured.owner_node_id != source.owner_node_id
            || captured.execution_id != source.execution_id
            || captured.evidence_through != source.evidence_through
            || captured.lifecycle_id != source.lifecycle_id
            || captured.input.generated_at != source.captured_at
            || captured.execution != source.execution
            || captured.execution().map_err(corrupt)? != &source.lifecycle
            || captured.context != source.context
            || captured.record_contracts != source.record_contracts
            || captured.telemetry_contracts != source.telemetry_contracts
        {
            return Err(corrupt(invalid(
                "materialized report input differs from frozen lifecycle source",
            )));
        }
        if let Some(anchor) = &captured.execution
            && !seal.source.execution.position.covers(&anchor.position)?
        {
            return Err(corrupt(invalid(
                "materialized primary anchor follows published revision",
            )));
        }
        captured.validate_final().map_err(corrupt)?;
        Ok(captured)
    })?)
}

pub(crate) async fn load_current(
    storage: &Manager,
    view: &StorageView,
    file: &FileReference,
    execution: &TestExecutionId,
    at: &StorageCursor,
) -> Result<QueryValue<CapturedExecutionReportInput>> {
    verify_reference(view, file, execution, at)?;
    let bytes = read_bytes(view, file).await?;
    bytes.check_origin(storage)?;
    let computation = view.begin_computation()?;
    let memory = decoding_memory(&bytes)?;
    computation.check()?;
    drop(computation);
    let captured = view.project_owned(bytes, bound(memory)?, |bytes, _| {
        let envelope: Envelope = serde_json::from_slice(&bytes).map_err(|_| {
            corrupt(invalid(
                "materialized report input JSON does not match its contract",
            ))
        })?;
        if envelope.version != SCHEMA_VERSION {
            return Err(corrupt(invalid(
                "unsupported materialized report input version",
            )));
        }
        Ok(envelope.captured)
    })?;
    let memory = captured
        .memory_size()
        .checked_add(captured.validation_memory())
        .ok_or_else(|| invalid("materialized input validation memory overflows"))?;
    Ok(view.project_owned(captured, bound(memory)?, |captured, _| {
        if captured.execution_id != *execution || captured.evidence_through != *at {
            return Err(corrupt(invalid(
                "materialized report input execution or evidence cursor differs",
            )));
        }
        captured.validate().map_err(corrupt)?;
        Ok(captured)
    })?)
}

/* === Private Functions === */
fn decoding_memory(bytes: &[u8]) -> Result<usize> {
    std::str::from_utf8(bytes).map_err(|_| invalid("materialized report input is not UTF-8"))?;
    // Inline Typed Elements, Vec Growth And BTreeMap Fanout Are Admitted Before
    // Serde Can Allocate. Plot JSON Uses Its Smaller Closed Value Representation.
    let inline = [
        std::mem::size_of::<CapturedExecutionReportInput>(),
        std::mem::size_of::<crate::execution::context::ExecutionContext>(),
        std::mem::size_of::<dokime::execution::TestExecution>(),
        std::mem::size_of::<dokime::definition::TestCase>(),
        std::mem::size_of::<crate::document::PlotDescriptor>(),
        std::mem::size_of::<crate::document::DocumentDefinition>(),
    ]
    .into_iter()
    .max()
    .expect("nonempty inline type inventory");
    let typed_cell = inline
        .checked_mul(16)
        .and_then(|bytes| bytes.checked_add(4096))
        .ok_or_else(|| invalid("materialized inline bound overflows"))?;
    let memory = bytes
        .len()
        .checked_mul(8)
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Envelope>()))
        .ok_or_else(|| invalid("materialized wire staging bound overflows"))?;
    let mut scan = Preflight {
        bytes,
        offset: 0,
        memory,
        typed_cell,
    };
    scan.value(Scope::Envelope, 0)?;
    scan.space();
    if scan.offset != bytes.len() {
        return Err(invalid("trailing materialized report input JSON"));
    }
    Ok(scan.memory)
}

async fn read_bytes(view: &StorageView, file: &FileReference) -> Result<QueryValue<Vec<u8>>> {
    let length = usize::try_from(file.length)
        .map_err(|_| invalid("materialized report input exceeds address space"))?;
    let maximum = view.decoded_bytes()?;
    let capacity = length
        .checked_add(std::mem::size_of::<Vec<u8>>())
        .ok_or_else(|| invalid("materialized file buffer overflows"))?;
    if length == 0 || capacity > maximum.get() {
        return Err(exhausted("materialized input file buffer").into());
    }
    let mut reader = view.read_file(file).await?;
    let step = bound(length)?;
    let first = reader
        .next_chunk(step)
        .await?
        .ok_or_else(|| invalid("materialized input file has no bytes"))?
        .into_query_value();
    let mut bytes = if first.len() == length {
        first
    } else {
        view.project_owned(first, bound(capacity)?, |first, _| {
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(length)
                .map_err(|_| exhausted("materialized input read allocation"))?;
            if bytes.capacity() > length {
                return Err(corrupt(invalid(
                    "materialized input read capacity exceeds preclaim",
                )));
            }
            bytes.extend_from_slice(&first);
            Ok(bytes)
        })?
    };
    while let Some(chunk) = reader.next_chunk(step).await? {
        bytes = view.project_owned(bytes, bound(capacity)?, |mut bytes, _| {
            let end = bytes
                .len()
                .checked_add(chunk.len())
                .filter(|end| *end <= length && *end <= bytes.capacity())
                .ok_or_else(|| {
                    corrupt(invalid("materialized input exceeds immutable file length"))
                })?;
            bytes.extend_from_slice(&chunk);
            debug_assert_eq!(bytes.len(), end);
            Ok(bytes)
        })?;
    }
    // The Standard Finished-File Reader Verifies Its Hash Before Its Last Batch.
    if bytes.len() != length {
        return Err(invalid("materialized input file length differs"));
    }
    Ok(bytes)
}

fn verify_reference(
    view: &StorageView,
    file: &FileReference,
    execution: &TestExecutionId,
    at: &StorageCursor,
) -> Result<()> {
    let token = file
        .metadata
        .id
        .as_str()
        .strip_prefix(execution.as_str())
        .and_then(|suffix| suffix.strip_prefix('.'))
        .and_then(|suffix| suffix.strip_suffix(MATERIALIZED_REPORT_INPUT_SUFFIX))
        .ok_or_else(|| invalid("materialized input logical identity differs"))?;
    if token.len() != 36
        || token.bytes().any(|byte| byte.is_ascii_uppercase())
        || TestExecutionId::from_uuid(token).is_err()
        || file.metadata.scope.as_ref().map(|scope| scope.as_str()) != Some(execution.as_str())
        || file.metadata.media_type != MEDIA_TYPE
        || file.length == 0
        || file.metadata.expected_length != Some(file.length)
        || file.metadata.expected_sha256 != Some(file.sha256)
    {
        return Err(invalid(
            "materialized input file metadata differs from immutable contract",
        ));
    }
    file.position.history.check(&at.history)?;
    let saved = view.saved()?;
    if !saved.covers(&file.position)? || !saved.covers(at)? {
        return Err(invalid("materialized input is outside the durable view"));
    }
    Ok(())
}

fn bound(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes)
        .filter(|_| bytes != usize::MAX)
        .ok_or_else(|| invalid("materialized input memory bound overflows"))
}
fn exhausted(resource: &'static str) -> harmos::Error {
    harmos::Error::ResourceExhausted { resource }
}
fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}
fn corrupt(source: Error) -> harmos::Error {
    harmos::Error::Backend {
        operation: "load materialized report input",
        kind: harmos::BackendFailure::Corrupt,
        source: Box::new(source),
    }
}

/* === Tests === */
