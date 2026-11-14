/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Data

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::{prelude::*, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Parameters)]
struct Inputs {
    limit: f64,
    channels: Vec<u32>,
    serial: Option<String>,
}

#[derive(dokime::Settings)]
struct Saved {
    enabled: bool,
}

#[derive(dokime::Message)]
struct Payload {
    signed: i64,
    bytes: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn compose_typed_data_and_refuse_invalid_catalogs_at_startup() -> Result<()> {
    // Define Parameters from Data and Compose a Suite by Value.
    let template = Template::new("data.case", "Data Case").parameters(Inputs {
        limit: 70.0,
        channels: vec![1, 3],
        serial: None,
    });
    let echo = Operation::new("data.echo", Visibility::Public, async |_: &Context, request: Payload| Ok(request));
    let catalog = Plugin::new("data", "1.0.0")
        .operation(echo)
        .settings(Saved { enabled: true })
        .suite(Suite::new("data", "Data").template(template))
        .catalog()?;
    let section = catalog.sections.iter().find(|section| section.r#type == "dokime.v1.Suites").expect("Suites Section");
    let suites =
        <v1::Suites as dokime::encoding::Message>::decode(section.payload.as_slice()).map_err(|error| Error::Invalid(error.to_string()))?;
    let parameters = &suites.suites[0].templates[0].parameters;
    assert_eq!(parameters[1].element_type, v1::ValueType::UnsignedInteger as i32);
    assert_eq!(parameters[1].r#type, v1::ValueType::List as i32);

    // Preserve Signed Data and Bytes Through the Public Data Derive.
    let payload = Payload { signed: -9, bytes: vec![0, 255] };
    let bytes = dokime::encoding::Message::encode_to_vec(&payload);
    let decoded = <Payload as dokime::encoding::Message>::decode(bytes.as_slice()).map_err(|error| Error::Invalid(error.to_string()))?;
    assert_eq!(decoded.signed, -9);
    assert_eq!(decoded.bytes, [0, 255]);
    assert!(catalog.sections.iter().any(|section| section.r#type == "dokime.v1.SettingsDeclarations"));

    // Reject Duplicate Step Identifiers Before Serving Any Request.
    let template = Template::new("data.duplicate", "Duplicate")
        .step(Step::new("Same", async |_: &Context| Ok(())))
        .step(Step::new("Same", async |_: &Context| Ok(())));
    assert!(Plugin::new("data", "1.0.0").suite(Suite::new("data", "Data").template(template)).catalog().is_err());

    // Reject Duplicate Templates Across Suites and Nonfinite Defaults.
    let plugin = Plugin::new("data", "1.0.0")
        .suite(Suite::new("first", "First").template(Template::new("data.case", "Case")))
        .suite(Suite::new("second", "Second").template(Template::new("data.case", "Case")));
    assert!(plugin.catalog().is_err());
    let template = Template::new("data.invalid", "Invalid").parameters(Inputs {
        limit: f64::NAN,
        channels: Vec::new(),
        serial: None,
    });
    assert!(Plugin::new("data", "1.0.0").suite(Suite::new("data", "Data").template(template)).catalog().is_err());
    Ok(())
}
