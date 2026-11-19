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
use std::collections::{HashMap, HashSet};

// Third Party Imports
use prost::Message;
use prost_reflect::{DynamicMessage, Value};
use sha2::{Digest, Sha256};

// Local Imports
use dokime::PageRequest;
use dokime_client_internal::{ExecutionsGetOptions, ExecutionsSchemasGetOptions};

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
    let client = settings.sdk(endpoint).await?;
    let mut schemas = HashMap::new();
    for name in args.execution {
        let value = client
            .executions
            .get_with(
                &name,
                ExecutionsGetOptions {
                    context: settings.context()?,
                    ..Default::default()
                },
            )
            .await?;
        crate::output::print(&value, args.json)?;
        if args.schemas {
            let detail = value.execution.as_ref().ok_or("Read Execution: Missing Execution Detail")?;
            let identities = detail.schemas.as_ref().ok_or("Read Schemas: Missing Frozen Schema Identities")?;
            if !schemas.contains_key(&identities.digest) {
                let pool = dokime_protocol::descriptor_pool();
                let descriptor =
                    pool.get_message_by_name("google.protobuf.FileDescriptorSet").ok_or("Read Schemas: Missing Descriptor Set Message")?;
                let file_descriptor = pool
                    .get_message_by_name("google.protobuf.FileDescriptorProto")
                    .ok_or("Read Schemas: Missing File Descriptor Message")?;
                let mut at = detail.resolved_at.clone();
                let mut page_token = String::new();
                let mut tokens = HashSet::new();
                let mut files = Vec::new();
                loop {
                    let options = ExecutionsSchemasGetOptions {
                        context: settings.context()?,
                        at: at.clone(),
                        page: Some(PageRequest {
                            page_size: 1,
                            page_token,
                            ..Default::default()
                        }),
                    };
                    let page = client.executions.schemas.get_with(&name, options).await?;
                    if page.digest != identities.digest || page.resolved_at.is_none() || (at.is_some() && page.resolved_at != at) {
                        return Err("Read Schemas: Frozen Schema Identity or Retained Cut Changed".into());
                    }
                    for file in page.types.unwrap_or_default().file {
                        let file = DynamicMessage::decode(file_descriptor.clone(), file.encode_to_vec().as_slice())?;
                        files.push(Value::Message(file));
                    }
                    if page.next_page_token.is_empty() {
                        break;
                    }
                    if !tokens.insert(page.next_page_token.clone()) {
                        return Err("Read Schemas: Repeated Page Token".into());
                    }
                    at = page.resolved_at;
                    page_token = page.next_page_token;
                }
                let mut types = DynamicMessage::new(descriptor);
                types.set_field_by_name("file", Value::List(files));
                if Sha256::digest(types.encode_to_vec()).as_slice() != identities.digest {
                    return Err("Read Schemas: Frozen Descriptor Digest Does Not Match".into());
                }
                schemas.insert(identities.digest.clone(), types);
            }
            let types = schemas.get(&identities.digest).ok_or("Read Schemas: Missing Assembled Descriptor Set")?;
            crate::output::print(types, args.json)?;
        }
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
