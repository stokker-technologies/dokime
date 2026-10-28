/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Browse Registered Suite Catalogs

mod list;
mod show;

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    #[command(about = "List Registered Suites In Catalog Order.")]
    List(list::Args),

    #[command(about = "Show A Registered Suite And Its Template Contracts.")]
    Show(show::Args),
}

pub(crate) async fn run(command: Command, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    match command {
        Command::List(args) => list::run(args, settings).await,
        Command::Show(args) => show::run(args, settings).await,
    }
}

// The CLI names a template by its readable ID, `<suite id>.<segment>`, in the
// `template_id` it prints and accepts; the registered UUID stays on the wire.
fn readable(suite: &dokime::domain::model::catalog::wire::SuiteDefinition) -> crate::Result<serde_json::Value> {
    let mut value = serde_json::to_value(suite)?;
    let templates = value.get_mut("templates").and_then(serde_json::Value::as_array_mut).into_iter().flatten().zip(&suite.templates);
    for (written, template) in templates {
        let identifier = template
            .metadata
            .as_ref()
            .map(|metadata| metadata.identifier.clone())
            .ok_or_else(|| crate::Error::Setup("a template has no identifier".into()))?;
        if let Some(written) = written.as_object_mut() {
            written.insert("template_id".into(), identifier.into());
        }
    }
    Ok(value)
}
