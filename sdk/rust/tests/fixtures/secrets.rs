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
use dokime::prelude::*;
use dokime::templates::Context;
use dokime::{Metadata, SecretDeclaration, SettingDeclaration, SettingValue, setting_value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Parameters)]
#[dokime(package = "secret_fixture")]
struct Parameters {
    identifier: String,
}

#[derive(dokime::Record)]
#[dokime(package = "secret_fixture")]
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
    let template = Template::of::<()>()
        .identifier("read")
        .label("Read Secret")
        .record(Record::<ReadResult>::new("read_result", "Secret Read"))
        .parameters(Parameters { identifier: secret.into() })
        .step(Step::new("Read Secret", read));
    Plugin::of::<()>("1.0.0")
        .identifier(plugin)
        .suite(Suite::of::<()>().identifier(plugin).label("Secret Reader").template(template))
        .secret(SecretDeclaration {
            metadata: Some(Metadata {
                identifier: secret.into(),
                label: "Declared Secret".into(),
                ..Default::default()
            }),
            required: true,
        })
        .setting(SettingDeclaration {
            name: "secrets.text".into(),
            title: "Text Setting".into(),
            description: String::new(),
            default: Some(SettingValue {
                value: Some(setting_value::Value::Text(String::new())),
            }),
        })
        .run()
}

async fn read(ctx: &Context) -> Result<()> {
    let identifier = ctx.parameters::<Parameters>()?.identifier;
    let result = match ctx.host().secrets.get(identifier).await {
        Ok(secret) => ReadResult {
            value: String::from_utf8(secret.secret).map_err(|error| Error::Invalid(error.to_string()))?,
            refusal: String::new(),
        },
        Err(error) => ReadResult {
            value: String::new(),
            refusal: error.to_string(),
        },
    };
    ctx.record(&Record::new("read_result", "Secret Read"), result).await?;
    Ok(())
}

/* === Tests === */
