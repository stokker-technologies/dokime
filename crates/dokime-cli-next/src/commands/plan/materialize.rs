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
use std::{collections::HashMap, path::PathBuf};

// Third Party Imports
use prost::Message;

// Local Imports
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long)]
    suite: String,

    #[arg(long)]
    template: String,

    #[arg(long, help = "JSON Object Mapping Parameters to Arrays of Dokime Values")]
    parameters: Option<PathBuf>,

    #[arg(long)]
    out: PathBuf,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let channel = settings.channel(endpoint, false).await?;
    let mut client = v1::planning_client::PlanningClient::with_interceptor(channel, settings.interceptor());
    let mut parameters: HashMap<String, Vec<v1::Value>> = HashMap::new();
    if let Some(path) = args.parameters {
        let input: HashMap<String, Vec<serde_json::Value>> = serde_json::from_slice(&std::fs::read(path)?)?;
        let descriptor = dokime_protocol::descriptor_pool().get_message_by_name("dokime.v1.Value").unwrap();
        for (name, values) in input {
            let mut converted = Vec::new();
            for value in values {
                let message = prost_reflect::DynamicMessage::deserialize(descriptor.clone(), value)?;
                converted.push(message.transcode_to::<v1::Value>()?);
            }
            parameters.insert(name, converted);
        }
    }
    let request = v1::PlanningMaterializeTemplateRequest {
        selection: Some(v1::RegisteredSelection {
            suite_id: args.suite, template_id: args.template,
            parameters: parameters.into_iter().map(|(key, values)| v1::RegisteredSelectionParametersEntry {
                key, value: Some(v1::SelectionValues { values }),
            }).collect(), ..Default::default()
        }),
    };
    let cases = client.materialize_template(request).await?.into_inner().cases;
    let value = v1::PreviewSelectionReply { cases, ..Default::default() };
    let bytes = value.encode_to_vec();
    if bytes.len() > settings.retained_bytes.get() {
        return Err(crate::error::Error::Capacity("Materialize Cases: Increase retained_bytes or Select Fewer Cases".into()));
    }
    std::fs::write(args.out, bytes)?;
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
