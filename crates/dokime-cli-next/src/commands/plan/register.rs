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
use dokime_protocol::v1;

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
    let descriptor = v1::PlanningRegisterSelectionRequest::default().descriptor();
    let mut json = serde_json::Deserializer::from_slice(&bytes);
    let message = prost_reflect::DynamicMessage::deserialize(descriptor, &mut json)?;
    json.end()?;
    let request = message.transcode_to::<v1::PlanningRegisterSelectionRequest>()?;

    // Let the Host Own Validation and Registration
    let channel = settings.channel(endpoint, false).await?;
    let reply = v1::planning_client::PlanningClient::with_interceptor(channel, settings.interceptor())
        .register_selection(request)
        .await?
        .into_inner();
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
