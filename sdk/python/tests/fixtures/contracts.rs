/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Export Valid Foreign Operation Fixtures Through The Rust Contract Owner

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, time::Duration};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use dokime::harmos::{
    self, Portable,
    operations::{Codec, Contract, Grpc, OperationDefinition, OperationPolicy, ReplaySafety, RetryPolicy, Visibility},
    schema::{Described, graph::Schema},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::message]
#[derive(Clone, Serialize, Deserialize)]
struct Wide {
    wide: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destination = std::env::args_os().nth(1).ok_or("supply the operation fixture JSON path")?;
    let mut contracts = BTreeMap::new();
    for (name, input, output) in
        [("unary", false, false), ("client_stream", true, false), ("server_stream", false, true), ("bidi", true, true)]
    {
        let mut declarations = vec![("request", Wide::DEFINITION), ("response", Wide::DEFINITION)];
        if input {
            declarations.push(("input", Wide::DEFINITION));
        }
        if output {
            declarations.push(("output", Wide::DEFINITION));
        }
        let fingerprint = harmos::schema::identity::operation(
            "external.fixture.invoke",
            Wide::fingerprint(),
            Wide::fingerprint(),
            input.then_some(Wide::fingerprint()),
            output.then_some(Wide::fingerprint()),
        );
        let operation = OperationDefinition {
            id: harmos::Identifier::new("external.fixture.invoke")?,
            description: "External Fixture".to_owned(),
            policy: OperationPolicy {
                timeout: Duration::from_secs(30),
                attempt_timeout: Duration::from_secs(30),
                cancellation_timeout: Duration::ZERO,
                retries: RetryPolicy::Never,
            },
            visibility: Visibility::Public,
            replay: ReplaySafety::Idempotent,
            fingerprint,
        };
        let schema = Schema::from_declarations(&declarations, &Default::default())?;
        let contract = Contract::new(operation, fingerprint, Codec::Postcard, schema)?.with_grpc(Some(Grpc::new(
            "external.v1",
            "Plugin",
            "Invoke",
        )))?;
        contracts.insert(name, contract);
    }
    std::fs::write(destination, serde_json::to_vec_pretty(&contracts)?)?;
    Ok(())
}

/* === Tests === */
