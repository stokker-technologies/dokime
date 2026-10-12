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
use std::fmt::Write;
use std::path::PathBuf;

// Third Party Imports
use harmos::catalog::{Catalog, Projection};
use prost::Message;
use tonic_prost_build::manual::{Builder, Method, Service};

// Local Imports
use dokime_contract::operations::{Endpoints, Operations, runtime};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = format!("/{}.{}/", runtime::PACKAGE, runtime::SERVICE);
    let mut declarations = Vec::new();
    <Operations as Catalog<Endpoints>>::ENTRIES.try_for_each(|entry| {
        let method = entry
            .path
            .strip_prefix(&prefix)
            .ok_or("public operation is outside the mounted Runtime service")?;
        declarations.push((method, entry.wire.contract()?, (entry.rust_type)()));
        Ok::<_, Box<dyn std::error::Error>>(())
    })?;
    let methods: Vec<_> = declarations
        .iter()
        .map(|(name, contract, declaration)| harmos::grpc::Method {
            name,
            contract,
            declaration,
        })
        .collect();
    let generated =
        harmos::grpc::generate_server(runtime::PACKAGE, runtime::SERVICE, "harmos", &methods)?;
    let output = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("OUT_DIR is missing")?);
    std::fs::write(output.join("runtime-operations.rs"), generated.source)?;
    let descriptor = prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?;
    let mut served = descriptor.clone();
    let operations = prost_types::FileDescriptorSet::decode(
        generated
            .descriptor
            .parent_pool()
            .encode_to_vec()
            .as_slice(),
    )?;
    for file in operations.file {
        if let Some(existing) = served
            .file
            .iter()
            .find(|existing| existing.name == file.name)
        {
            if existing != &file {
                return Err("conflicting generated descriptor dependency".into());
            }
        } else {
            served.file.push(file);
        }
    }
    std::fs::write(output.join("served-descriptor.bin"), served.encode_to_vec())?;
    let mut inputs = String::new();
    for service_name in ["Evidence", "Execution", "History", "Content", "Query"] {
        let file = descriptor
            .file
            .iter()
            .find(|file| {
                file.package.as_deref() == Some(dokime_protocol::API_PACKAGE)
                    && file
                        .service
                        .iter()
                        .any(|service| service.name() == service_name)
            })
            .ok_or("canonical service is missing")?;
        let declared = file
            .service
            .iter()
            .find(|service| service.name() == service_name)
            .ok_or("canonical service is missing")?;
        let mut service = Service::builder()
            .name(declared.name())
            .package(file.package());
        for method in &declared.method {
            let output = rust_type(method.output_type())?;
            let (output_type, codec) = match method.output_type() {
                ".dokime.v1.RecordPage" if service_name == "Evidence" => {
                    let tag = field_number(&descriptor, method.input_type(), "selection")?;
                    writeln!(
                        inputs,
                        "impl super::Input for {} {{ const SELECTION: u32 = {tag}; }}",
                        rust_type(method.input_type())?
                    )?;
                    (
                        "crate::grpc::evidence::codec::Page",
                        "crate::grpc::evidence::codec::Codec",
                    )
                }
                ".dokime.v1.ExecutionBrowsePage"
                | ".dokime.v1.CheckpointRecordPage"
                | ".dokime.v1.CheckpointTelemetryPage"
                | ".dokime.v1.ExecutionCheckpoint"
                | ".dokime.v1.CheckpointDocument"
                | ".dokime.v1.CheckpointContext"
                | ".dokime.v1.CheckpointEvidence"
                | ".dokime.v1.CheckpointFileWindow"
                    if service_name == "History" =>
                {
                    (
                        "crate::grpc::history::codec::Page",
                        "crate::grpc::history::codec::Codec",
                    )
                }
                ".dokime.v1.AggregateReply" if service_name == "Query" => {
                    ("crate::grpc::charged::Bytes", "crate::grpc::charged::Codec")
                }
                ".dokime.v1.ExecutionFileWindow"
                | ".dokime.v1.ListExecutionFilesReply"
                | ".dokime.v1.ExecutionFileDownloadFrame"
                    if service_name == "Content" =>
                {
                    ("crate::grpc::charged::Bytes", "crate::grpc::charged::Codec")
                }
                ".dokime.v1.RunRequestReceipt" if service_name == "Execution" => (
                    "crate::grpc::execution::codec::RunReceipt",
                    "crate::grpc::execution::codec::RunCodec",
                ),
                ".dokime.v1.ExecutionCommandReceipt" if service_name == "Execution" => (
                    "crate::grpc::execution::codec::CommandReceipt",
                    "crate::grpc::execution::codec::CommandCodec",
                ),
                _ => (output.as_str(), "tonic_prost::ProstCodec"),
            };
            let mut generated = Method::builder()
                .name(rust_method(method.name()))
                .route_name(method.name())
                .input_type(rust_type(method.input_type())?)
                .output_type(output_type)
                .codec_path(codec);
            if method.client_streaming() {
                generated = generated.client_streaming();
            }
            if method.server_streaming() {
                generated = generated.server_streaming();
            }
            service = service.method(generated.build());
        }
        Builder::new()
            .build_client(false)
            .build_server(true)
            .compile(&[service.build()]);
    }
    for (name, field) in [
        ("RECORD_TYPES", "record_types"),
        ("SOURCES", "sources"),
        ("SEVERITIES", "severities"),
    ] {
        let tag = field_number(&descriptor, ".dokime.v1.CapturedRecordSelection", field)?;
        writeln!(inputs, "pub(super) const {name}: u32 = {tag};")?;
    }
    let output = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("OUT_DIR is missing")?);
    std::fs::write(output.join("evidence-inputs.rs"), inputs)?;
    let mut fields = String::new();
    for (name, field) in [
        ("PATH", "relative_path"),
        ("MEDIA_TYPE", "content_type"),
        ("DATA", "data"),
    ] {
        let tag = field_number(&descriptor, ".dokime.v1.ExecutionFileWindow", field)?;
        writeln!(fields, "const {name}: u32 = {tag};")?;
    }
    std::fs::write(output.join("content-output-fields.rs"), fields)?;
    let mut fields = String::new();
    for (name, message, field) in [
        ("TEXT", ".dokime.v1.Value", "text_value"),
        ("DATETIME", ".dokime.v1.Value", "datetime_value"),
        ("ARRAY", ".dokime.v1.Value", "array_value"),
        ("OBJECT", ".dokime.v1.Value", "object_value"),
        ("LIST_VALUES", ".dokime.v1.ValueList", "values"),
        ("OBJECT_FIELDS", ".dokime.v1.ValueObject", "fields"),
        ("METADATA_ID", ".dokime.v1.Metadata", "identifier"),
        ("METADATA_LABEL", ".dokime.v1.Metadata", "label"),
        ("METADATA_DESCRIPTION", ".dokime.v1.Metadata", "description"),
        ("METADATA_TAGS", ".dokime.v1.Metadata", "tags"),
        ("METADATA_ATTRIBUTES", ".dokime.v1.Metadata", "attributes"),
        ("METADATA_CREATED", ".dokime.v1.Metadata", "created_at"),
        ("METADATA_UPDATED", ".dokime.v1.Metadata", "updated_at"),
    ] {
        let tag = field_number(&descriptor, message, field)?;
        writeln!(fields, "const {name}: u32 = {tag};")?;
    }
    std::fs::write(output.join("value-output-fields.rs"), fields)?;
    let mut fields = String::new();
    for (name, message, field) in [
        ("COLUMNS", ".dokime.v1.AggregateReply", "columns"),
        ("ROWS", ".dokime.v1.AggregateReply", "rows"),
        ("VALUES", ".dokime.v1.QueryRow", "values"),
    ] {
        let tag = field_number(&descriptor, message, field)?;
        writeln!(fields, "const {name}: u32 = {tag};")?;
    }
    std::fs::write(output.join("aggregate-output-fields.rs"), fields)?;
    let mut fields = String::new();
    for (name, message, field) in [
        ("FILES", ".dokime.v1.ListExecutionFilesReply", "files"),
        (
            "NEXT",
            ".dokime.v1.ListExecutionFilesReply",
            "next_after_path",
        ),
        ("CURSOR", ".dokime.v1.ListExecutionFilesReply", "cursor"),
        ("PATH", ".dokime.v1.ExecutionFileMetadata", "relative_path"),
        (
            "MEDIA_TYPE",
            ".dokime.v1.ExecutionFileMetadata",
            "content_type",
        ),
        ("EPOCH", ".dokime.v1.ExecutionFilesCursor", "epoch"),
        (
            "EXECUTION",
            ".dokime.v1.ExecutionFilesCursor",
            "execution_id",
        ),
        (
            "DIGEST",
            ".dokime.v1.ExecutionFilesCursor",
            "metadata_sha256",
        ),
    ] {
        let tag = field_number(&descriptor, message, field)?;
        writeln!(fields, "const {name}: u32 = {tag};")?;
    }
    std::fs::write(output.join("inventory-output-fields.rs"), fields)?;
    let mut fields = String::new();
    for (name, message, field) in [
        (
            "METADATA",
            ".dokime.v1.ExecutionFileDownloadFrame",
            "metadata",
        ),
        ("CHUNK", ".dokime.v1.ExecutionFileDownloadFrame", "chunk"),
        (
            "COMPLETE",
            ".dokime.v1.ExecutionFileDownloadFrame",
            "complete",
        ),
        (
            "PATH",
            ".dokime.v1.ExecutionFileDownloadMetadata",
            "relative_path",
        ),
        (
            "MEDIA_TYPE",
            ".dokime.v1.ExecutionFileDownloadMetadata",
            "content_type",
        ),
        ("DATA", ".dokime.v1.ArtifactChunk", "data"),
        ("DIGEST", ".dokime.v1.ArtifactCompletion", "sha256"),
    ] {
        let tag = field_number(&descriptor, message, field)?;
        writeln!(fields, "const {name}: u32 = {tag};")?;
    }
    std::fs::write(output.join("download-output-fields.rs"), fields)?;
    Ok(())
}

fn field_number(
    descriptor: &prost_types::FileDescriptorSet,
    message: &str,
    field: &str,
) -> Result<u32, Box<dyn std::error::Error>> {
    let (package, name) = message
        .trim_start_matches('.')
        .rsplit_once('.')
        .ok_or("invalid message identity")?;
    let selected = descriptor
        .file
        .iter()
        .filter(|file| file.package() == package)
        .flat_map(|file| &file.message_type)
        .find(|message| message.name() == name)
        .and_then(|message| {
            message
                .field
                .iter()
                .find(|candidate| candidate.name() == field)
        })
        .ok_or("canonical selection field is missing")?;
    Ok(u32::try_from(selected.number())?)
}

fn rust_type(wire: &str) -> Result<String, Box<dyn std::error::Error>> {
    let name = wire
        .strip_prefix(".dokime.v1.")
        .ok_or("unsupported Evidence message package")?;
    if name.is_empty()
        || !name
            .chars()
            .all(|letter| letter.is_ascii_alphanumeric() || letter == '_')
    {
        return Err("unsupported Evidence message name".into());
    }
    Ok(format!("dokime_protocol::v1::{name}"))
}

fn rust_method(name: &str) -> String {
    let letters = name.as_bytes();
    let mut output = String::with_capacity(name.len() * 2);
    for (index, letter) in letters.iter().copied().enumerate() {
        let boundary = index > 0
            && letter.is_ascii_uppercase()
            && (letters[index - 1].is_ascii_lowercase()
                || letters[index - 1].is_ascii_digit()
                || letters.get(index + 1).is_some_and(u8::is_ascii_lowercase));
        if boundary {
            output.push('_');
        }
        output.push(char::from(letter.to_ascii_lowercase()));
    }
    output
}

/* === Tests === */
