/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Inspect Host-Owned State Through the SDK

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use dokime::client::FrozenSchemaCache;
use dokime_protocol::client;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(required = true)]
    execution: Vec<String>,

    #[arg(long, help = "Fetch Frozen Descriptors Once per Digest")]
    schemas: bool,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let client = settings.sdk(endpoint, false).await?;
    let mut schemas = FrozenSchemaCache::default();
    for name in args.execution {
        let request = client::ExecutionsGetRequest { name, at: None };
        let value = client.executions.get(settings.request(request)).await?.into_inner();
        crate::output::print(&value, args.json)?;
        if args.schemas {
            let detail = value.execution.as_ref().ok_or("Read Execution: Missing Execution Detail")?;
            let types = schemas.get(&client, detail).await?;
            let descriptor = dokime_protocol::descriptor_pool()
                .get_message_by_name("google.protobuf.FileDescriptorSet")
                .ok_or("Read Schemas: Missing Descriptor Set Message")?;
            let types = prost_reflect::DynamicMessage::decode(descriptor, types.encode_to_vec().as_slice())?;
            crate::output::print(&types, args.json)?;
        }
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
