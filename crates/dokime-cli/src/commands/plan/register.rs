/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Materialize Frozen Cases Through the SDK

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io::Read, path::PathBuf};

// Third Party Imports
use prost::Message;
use prost_reflect::ReflectMessage;

// Local Imports
use crate::client::CasesCreateOptions;
use dokime_protocol::host::CasesCreateRequest;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, help = "Canonical Protobuf JSON Request")]
    input_file: PathBuf,

    #[arg(long, help = "Save the Protobuf Reply for Cycle Creation")]
    out: Option<PathBuf>,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    // Read One Bounded Canonical Request
    let input = std::fs::File::open(args.input_file)?;
    let mut bytes = Vec::new();
    input.take(settings.retained_bytes.get() as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > settings.retained_bytes.get() {
        return Err(crate::error::Error::Capacity("Register Selection: Select a Smaller Request".into()));
    }
    let descriptor = CasesCreateRequest::default().descriptor();
    let mut json = serde_json::Deserializer::from_slice(&bytes);
    let message = prost_reflect::DynamicMessage::deserialize(descriptor, &mut json)?;
    json.end()?;
    let mut request = message.transcode_to::<CasesCreateRequest>()?;
    request.validate_only = false;

    // Let the Host Own Validation and Registration
    let client = settings.sdk(endpoint).await?;
    let reply = client
        .cases
        .create_with(
            request.plan.ok_or("Register Selection: Supply a Selection Plan")?,
            request.validate_only,
            CasesCreateOptions {
                context: settings.context()?,
                request_id: (!request.request_id.is_empty()).then_some(request.request_id),
            },
        )
        .await?;
    if let Some(path) = args.out {
        let bytes = reply.encode_to_vec();
        if bytes.len() > settings.retained_bytes.get() {
            return Err(crate::error::Error::Capacity("Register Selection: Select Fewer Cases".into()));
        }
        std::fs::write(path, bytes)?;
    }
    crate::output::print(&reply, args.json)
}

/* === Private Functions === */

/* === Tests === */
