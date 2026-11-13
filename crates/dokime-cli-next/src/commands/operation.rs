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

// Third Party Imports

// Local Imports


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(clap::Subcommand)]
pub(crate) enum Command {
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

impl Command {
    pub fn action(&self) -> &'static str {
        match self {
            Self::List { .. } => "List Operations",
            Self::Show { .. } => "Show Operation",
        }
    }
}

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let channel = settings.channel(endpoint, false).await?;
    let client = dokime::Client::from_channel(channel, settings.client())?;
    match command {
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
