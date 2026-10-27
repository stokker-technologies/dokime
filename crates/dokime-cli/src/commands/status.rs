/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Backend Authentication And Connection Inspection

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

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Emit The Result As JSON Without Secret Values.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn run(mut args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    args.connection = args.connection.with_settings(*settings);
    check(args.connection, args.json).await
}

pub(super) async fn check(connection: crate::connection::Args, json: bool) -> crate::Result<()> {
    let resolved = connection.resolve()?;
    let endpoint = resolved.selection.target.endpoint();
    let result = connection.connect_selected(&resolved, resolved.token.clone()).await;
    match result {
        Ok(client) => {
            let backend = client.backend().clone();
            client.close_and_wait().await;
            if json {
                println!("{}", serde_json::json!({"status": "ready", "endpoint": endpoint, "backend": backend}));
            } else {
                println!("Ready: {} ({endpoint})", backend.display_name);
                println!("Backend: {}", backend.backend_id);
            }
            Ok(())
        }
        Err(error) => {
            let status = match &error {
                crate::Error::Client(error) => serde_json::to_value(error.connection_failure())?,
                crate::Error::Secrets(_) | crate::Error::SecretTimeout => {
                    serde_json::json!("secrets_unavailable")
                }
                crate::Error::Io(error) if error.kind() == std::io::ErrorKind::TimedOut => {
                    serde_json::json!("timed_out")
                }
                _ => serde_json::json!("failed"),
            };
            if json {
                println!("{}", serde_json::json!({"status": status, "endpoint": endpoint, "error": error.to_string()}));
            } else {
                let label = match status.as_str() {
                    Some("configuration") => "Invalid Configuration",
                    Some("authentication") => "Authentication Required",
                    Some("permission_denied") => "Permission Denied",
                    Some("unreachable") => "Unreachable",
                    Some("timed_out") => "Timed Out",
                    Some("incompatible") => "Incompatible",
                    Some("not_ready") => "Not Ready",
                    Some("secrets_unavailable") => "Secrets Unavailable",
                    _ => "Failed",
                };
                println!("{label}: {endpoint}");
            }
            Err(crate::Error::Check(Box::new(error)))
        }
    }
}

/* === Private Functions === */

/* === Tests === */
