/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Secret Admission Fixture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::env;

// Third Party Imports

// Local Imports
use dokime::protocol::plugin::host::v1::{ReadSecret, evidence::RecordsAppendRequest};
use dokime::{prelude::*, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Parameters)]
#[harmos(package = "secret_fixture.v1")]
struct Parameters {
    identifier: String,
}

#[derive(dokime::Record)]
#[harmos(package = "secret_fixture.v1")]
struct ReadResult {
    value: String,
    refusal: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    let owner = env::var_os("DOKIME_SECRET_FIXTURE_OWNER").is_some();
    let (plugin, secret) = if owner { ("secret_owner", "secrets.other") } else { ("secret_reader", "secrets.owned") };
    let template = Template::new(format!("{plugin}.read"), "Read Secret")
        .parameters(Parameters { identifier: secret.into() })
        .step(Step::new("Read Secret", read));
    Plugin::new(plugin, "1.0.0")
        .suite(Suite::new(plugin, "Secret Reader").template(template))
        .record(Record::<ReadResult>::new(format!("{plugin}.read_result"), "Secret Read"))
        .setting(v1::Setting {
            metadata: Some(v1::Metadata {
                identifier: secret.into(),
                label: "Declared Secret".into(),
                ..Default::default()
            }),
            kind: Some(v1::setting::Kind::Secret(v1::SecretSetting { required: true })),
        })
        .setting(v1::Setting {
            metadata: Some(v1::Metadata {
                identifier: "secrets.text".into(),
                label: "Text Setting".into(),
                ..Default::default()
            }),
            kind: Some(v1::setting::Kind::Text(v1::TextSetting::default())),
        })
        .requires_host::<ReadSecret>()
        .requires_host::<RecordsAppendRequest>()
        .remote()
        .serve()
}

async fn read(ctx: &Context) -> Result<()> {
    let identifier = ctx.parameters::<Parameters>()?.identifier;
    let result = match ctx.secret(identifier).await {
        Ok(secret) => ReadResult {
            value: String::from_utf8(secret.value).map_err(|error| Error::Invalid(error.to_string()))?,
            refusal: String::new(),
        },
        Err(error) => ReadResult {
            value: String::new(),
            refusal: error.to_string(),
        },
    };
    let Some(v1::origin::Kind::PluginRun(run)) = ctx.origin()?.kind.as_ref() else {
        return Err(Error::Invalid("Read Secret: Use a Plugin Execution".into()));
    };
    ctx.record(&Record::new(format!("{}.read_result", run.plugin), "Secret Read"), result).await?;
    Ok(())
}

/* === Tests === */
