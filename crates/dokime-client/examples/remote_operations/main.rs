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
use std::{env, time::Duration};
// Third Party Imports
use zeroize::Zeroizing;
// Local Imports
use dokime_client::{Client, Target, dynamic::Event};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = env::var("DOKIME_ENDPOINT")?;
    let token = env::var("DOKIME_TOKEN").ok().map(Zeroizing::new);
    let expected = env::var("DOKIME_BACKEND_ID").ok();
    let ca = env::var("DOKIME_CA_FILE")
        .ok()
        .map(std::fs::read_to_string)
        .transpose()?;
    let identifier = env::args()
        .nth(1)
        .ok_or("provide an operation identifier")?;
    let owner = env::args().nth(2);
    let request = env::var("DOKIME_REQUEST_JSON").unwrap_or_else(|_| "{}".into());
    let target = Target::new(endpoint, ca)?;
    let client = Client::connect(
        target,
        token.as_deref().map(String::as_str),
        expected.as_deref(),
    )
    .await?
    .with_timeout(Duration::from_secs(30));
    let result = async {
        let stream = client
            .call_operation(
                &identifier,
                owner.as_deref(),
                request.as_bytes(),
                Vec::new(),
            )
            .await?;
        let read = async {
            let interrupt = tokio::signal::ctrl_c();
            tokio::pin!(interrupt);
            loop {
                let event = tokio::select! {
                    biased;
                    result = &mut interrupt => { result?; break; }
                    event = stream.message() => event?,
                };
                match event {
                    Some(Event::Item(value)) => {
                        println!("{}", serde_json::json!({"event": "item", "value": value}))
                    }
                    Some(Event::Completion(value)) => println!(
                        "{}",
                        serde_json::json!({"event": "completion", "value": value})
                    ),
                    None => break,
                }
            }
            Ok::<_, Box<dyn std::error::Error>>(())
        }
        .await;
        stream.close().await;
        read
    }
    .await;
    client.close_and_wait().await;
    result
}

/* === Tests === */
