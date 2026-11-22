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
use prost_reflect::ReflectMessage;

// Local Imports
use dokime::Value;
use dokime_client_internal::{CasesCreateOptions, RegisteredSelection, RegisteredSelectionParametersEntry, SelectionPlan, SelectionValues};
use dokime_protocol::host::CasesCreateRequest;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, help = "Reuse This Identity When Retrying the Same Mutation")]
    request_id: Option<String>,

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
    let client = settings.sdk(endpoint).await?;
    let mut parameters: HashMap<String, Vec<Value>> = HashMap::new();
    if let Some(path) = args.parameters {
        let input: HashMap<String, Vec<serde_json::Value>> = serde_json::from_slice(&std::fs::read(path)?)?;
        let descriptor = Value::default().descriptor();
        for (name, values) in input {
            let mut converted = Vec::new();
            for value in values {
                let message = prost_reflect::DynamicMessage::deserialize(descriptor.clone(), value)?;
                converted.push(message.transcode_to::<Value>()?);
            }
            parameters.insert(name, converted);
        }
    }
    let request = CasesCreateRequest {
        request_id: settings.request_id(args.request_id)?,
        validate_only: false,
        plan: Some(SelectionPlan {
            selections: vec![RegisteredSelection {
                suite_id: args.suite,
                template_id: args.template,
                parameters: parameters
                    .into_iter()
                    .map(|(key, values)| RegisteredSelectionParametersEntry {
                        key,
                        value: Some(SelectionValues { values }),
                    })
                    .collect(),
                ..Default::default()
            }],
            ..Default::default()
        }),
    };
    let value = client
        .cases
        .create_with(
            request.plan.unwrap_or_default(),
            request.validate_only,
            CasesCreateOptions {
                context: settings.context()?,
                request_id: Some(request.request_id),
            },
        )
        .await?;
    let bytes = value.encode_to_vec();
    if bytes.len() > settings.retained_bytes.get() {
        return Err(crate::error::Error::Capacity("Materialize Cases: Increase retained_bytes or Select Fewer Cases".into()));
    }
    std::fs::write(args.out, bytes)?;
    if args.json {
        return crate::output::print(&value, true);
    }
    for case in value.cases {
        println!("{}\t{}\t{}", case.case_id, case.suite_id, case.template_id);
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
