/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Discover and Invoke Host Operations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io::Read, path::PathBuf};

// Third Party Imports
use prost::Message;
use prost::bytes::{Buf, BufMut};

// Local Imports


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The SDK Admits the Descriptor-Bound Reply Before This Lossless Wire Container Decodes It.
#[derive(Default, Debug)]
struct WireMessage(Vec<u8>);

struct Recording<'a, B> {
    input: &'a mut B,
    bytes: Vec<u8>,
}

/* === Enums === */

#[derive(clap::Subcommand)]
pub(crate) enum Command {
    Call {
        operation: String,

        #[arg(long, required_unless_present = "input_file", conflicts_with = "input_file", help = "Typed Request as Protobuf JSON")]
        json: Option<String>,

        #[arg(long, required_unless_present = "json")]
        input_file: Option<PathBuf>,
    },

    Show {
        operation: String,

        #[arg(long)]
        json: bool,
    },

    List {
        #[arg(long)]
        json: bool,
    },
}

/* === Traits === */

/* === Implementations === */

impl<B: Buf> Buf for Recording<'_, B> {
    fn remaining(&self) -> usize { self.input.remaining() }
    fn chunk(&self) -> &[u8] { self.input.chunk() }
    fn advance(&mut self, count: usize) {
        let bytes = self.input.copy_to_bytes(count);
        self.bytes.extend_from_slice(&bytes);
    }
}

impl Message for WireMessage {
    fn encode_raw(&self, buffer: &mut impl BufMut) { buffer.put_slice(&self.0); }
    fn merge_field(&mut self, tag: u32, wire: prost::encoding::WireType, buffer: &mut impl Buf,
        context: prost::encoding::DecodeContext) -> Result<(), prost::DecodeError> {
        let mut recording = Recording { input: buffer, bytes: Vec::new() };
        prost::encoding::skip_field(wire, tag, &mut recording, context)?;
        prost::encoding::encode_key(tag, wire, &mut self.0);
        self.0.extend_from_slice(&recording.bytes);
        Ok(())
    }
    fn encoded_len(&self) -> usize { self.0.len() }
    fn clear(&mut self) { self.0.clear(); }
}


impl Command {
    pub fn action(&self) -> &'static str {
        match self {
            Self::Call { .. } => "Call Operation",
            Self::List { .. } => "List Operations",
            Self::Show { .. } => "Show Operation",
        }
    }
}

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let channel = settings.channel(endpoint, false).await?;
    let mut client = dokime::Client::from_channel(channel, settings.client())?;
    if let Some(pairing) = &settings.pairing { client = client.authorization(pairing.authorization()?); }
    match command {
        Command::Call { operation, json, input_file } => {
            let discovered = find(&client, &operation).await?;
            let contract = discovered.contract.ok_or("Operation Has No Contract")?;
            if contract.visibility != dokime::client::v1::Visibility::Public as i32 {
                return Err(tonic::Status::permission_denied(format!("{operation}: Public Visibility Required")).into());
            }
            let protobuf = contract.protobuf.ok_or_else(|| format!("{operation}: No Protobuf Contract Is Available"))?;
            if protobuf.input.is_some() || protobuf.output.is_some() {
                return Err(crate::error::Error::Invalid(format!("{operation}: Use a Typed Streaming Client for This Contract")));
            }
            let pool = prost_reflect::DescriptorPool::decode(protobuf.descriptors.as_slice())
                .map_err(|error| format!("Decode {operation} Contract: {error}"))?;
            let descriptor = pool.get_message_by_name(&protobuf.request).ok_or("Operation Has No Request Descriptor")?;
            let response = pool.get_message_by_name(&protobuf.response).ok_or("Operation Has No Response Descriptor")?;
            let mut bytes = json.unwrap_or_default().into_bytes();
            if let Some(path) = input_file {
                std::fs::File::open(path)?.take(settings.retained_bytes.get() as u64 + 1).read_to_end(&mut bytes)?;
            }
            if bytes.len() > settings.retained_bytes.get() {
                return Err(crate::error::Error::Capacity(format!("{operation}: Select a Smaller JSON Request or Raise retained_bytes")));
            }
            let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
            let request = prost_reflect::DynamicMessage::deserialize(descriptor, &mut deserializer)
                .map_err(|error| crate::error::Error::Invalid(format!("{operation}: Correct the Typed JSON Request: {error}")))?;
            deserializer.end().map_err(|error| crate::error::Error::Invalid(format!("{operation}: Correct the Typed JSON Request: {error}")))?;
            let grpc = contract.grpc.filter(|_| discovered.grpc_available)
                .ok_or_else(|| format!("{operation}: Host Must Expose This Public Operation Through Its Generic Invocation Service"))?;
            if grpc.managed {
                return Err(crate::error::Error::Invalid(format!("{operation}: Use a Typed Managed Client for This Contract")));
            }
            let path = format!("/{}.{}/{}", grpc.package, grpc.service, grpc.method).parse()
                .map_err(|error| format!("{operation}: Invalid gRPC Mapping: {error}"))?;
            let reply: WireMessage = client.transport_handle().unary(path, settings.request(WireMessage(request.encode_to_vec())), response.clone())
                .await?.into_inner();
            if reply.0.len() > settings.retained_bytes.get() {
                return Err(crate::error::Error::Capacity(format!("{operation}: Response Exceeds retained_bytes")));
            }
            let reply = prost_reflect::DynamicMessage::decode(response, reply.0.as_slice())?;
            crate::output::print(&reply, true)
        }
        Command::Show { operation, json } => {
            let discovered = find(&client, &operation).await?;
            let contract = discovered.contract.as_ref().ok_or("Operation Has No Contract")?;
            let mut value = crate::output::value(&discovered)?;
            if let Some(protobuf) = &contract.protobuf {
                let pool = prost_reflect::DescriptorPool::decode(protobuf.descriptors.as_slice())
                    .map_err(|error| format!("Decode {operation} Contract: {error}"))?;
                let request = pool.get_message_by_name(&protobuf.request).ok_or("Operation Has No Request Descriptor")?;
                let response = pool.get_message_by_name(&protobuf.response).ok_or("Operation Has No Response Descriptor")?;
                value["request_shape"] = shape(request);
                value["response_shape"] = shape(response);
            }
            if json { println!("{}", serde_json::to_string(&value)?); }
            else { println!("{}", serde_json::to_string_pretty(&value)?); }
            Ok(())
        }
        Command::List { json } => {
            let request = dokime::client::CatalogEntriesListRequest {
                kind: dokime::client::CatalogKind::Operations as i32,
                ..Default::default()
            };
            let reply = client.catalog(request).await?.into_inner();
            crate::output::print(&reply, json)
        }
    }
}

/* === Private Functions === */

async fn find(client: &dokime::Client, operation: &str) -> crate::Result<dokime::client::DiscoveredOperation> {
    let reply = client.catalog(dokime::client::CatalogEntriesListRequest {
        kind: dokime::client::CatalogKind::Operations as i32,
        ..Default::default()
    }).await?.into_inner();
    reply.operations.into_iter().find(|entry| entry.contract.as_ref().is_some_and(|contract| contract.id == operation))
        .ok_or_else(|| tonic::Status::not_found(format!("{operation}: Discover an Admitted Operation With operation list")).into())
}

fn shape(message: prost_reflect::MessageDescriptor) -> serde_json::Value {
    let fields = message.fields().map(|field| {
        let kind = match field.kind() {
            prost_reflect::Kind::Message(message) => message.full_name().to_owned(),
            prost_reflect::Kind::Enum(value) => value.full_name().to_owned(),
            scalar => format!("{scalar:?}").to_lowercase(),
        };
        serde_json::json!({"name": field.name(), "json_name": field.json_name(), "number": field.number(),
            "type": kind, "repeated": field.is_list(), "map": field.is_map(), "presence": field.supports_presence()})
    }).collect::<Vec<_>>();
    serde_json::json!({"message": message.full_name(), "fields": fields})
}


/* === Tests === */
