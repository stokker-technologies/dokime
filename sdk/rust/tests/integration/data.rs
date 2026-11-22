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
use dokime::Suites;
use dokime::ValueType;
use dokime::operations;
use dokime::prelude::*;
use dokime::templates::Context;
use dokime::{
    DataExpression, ExactNumber, FieldReference, FromValue, SettingDeclaration, SettingValue, Value, encoding, setting_value, value,
};

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
fn telemetry_at_records_preserves_owned_operands_and_optional_tolerance() -> Result<()> {
    let input = FieldReference::new("readings").field("value");
    let sampled = input.at("status");
    let bounded = sampled.clone().within_s(0.5);
    let bytes = encoding::Message::encode_to_vec(&bounded);
    let decoded = <DataExpression as encoding::Message>::decode(bytes.as_slice()).map_err(|error| Error::Invalid(error.to_string()))?;
    assert_eq!(decoded.at_record().unwrap().record_type, "status");
    assert_eq!(decoded.at_record().unwrap().within_s, Some(0.5));
    assert_eq!(sampled.at_record().unwrap().within_s, None);
    Ok(())
}

#[test]
fn compose_typed_data_and_refuse_invalid_declarations_at_startup() -> Result<()> {
    // Define Parameters from Data and Compose a Suite by Value.
    let template = Template::of::<()>().identifier("case").label("Data Case").parameters(Inputs {
        limit: 70.0,
        channels: vec![1, 3],
        serial: None,
    });
    let echo = Operation::new("data.echo", Visibility::Public, async |_: &operations::Context, request: Payload| Ok(request));
    let declarations = Plugin::of::<()>("1.0.0")
        .identifier("data")
        .operation(echo)
        .setting(SettingDeclaration {
            name: "enabled".into(),
            title: "Enabled".into(),
            description: String::new(),
            default: Some(SettingValue {
                value: Some(setting_value::Value::Flag(true)),
            }),
            minimum: None,
            maximum: None,
            ..Default::default()
        })
        .suite(Suite::of::<()>().identifier("data.data").label("Data").template(template))
        .declarations()?;
    let section = declarations.sections.iter().find(|section| section.r#type == "dokime.model.v1.Suites").expect("Suites Section");
    let suites = <Suites as encoding::Message>::decode(section.payload.as_slice()).map_err(|error| Error::Invalid(error.to_string()))?;
    let parameters = &suites.suites[0].templates[0].parameters;
    assert_eq!(parameters[1].element_type, ValueType::UnsignedInteger as i32);
    assert_eq!(parameters[1].r#type, ValueType::List as i32);

    // Preserve Signed Data and Bytes Through the Public Data Derive.
    let payload = Payload { signed: -9, bytes: vec![0, 255] };
    let bytes = encoding::Message::encode_to_vec(&payload);
    let decoded = <Payload as encoding::Message>::decode(bytes.as_slice()).map_err(|error| Error::Invalid(error.to_string()))?;
    assert_eq!(decoded.signed, -9);
    assert_eq!(decoded.bytes, [0, 255]);
    assert!(declarations.sections.iter().any(|section| section.r#type == "harmos.model.v1.SettingsDeclarations"));

    // Retain Integer Precision and Rational Values Without Floating Conversion.
    let exact = ExactNumber {
        negative: true,
        numerator: "1152921504606847975".into(),
        denominator: "3".into(),
    };
    let value = Value {
        kind: Some(value::Kind::Exact(exact.clone())),
    };
    assert_eq!(<ExactNumber as FromValue>::from_value(&value)?, exact);
    assert!(<f64 as FromValue>::from_value(&value).is_err());

    // Reject Duplicate Step Identifiers Before Serving Any Request.
    let template = Template::of::<()>()
        .identifier("duplicate")
        .label("Duplicate")
        .step(Step::new("Same", async |_: &Context| Ok(())).identifier("same"))
        .step(Step::new("Same", async |_: &Context| Ok(())).identifier("same"));
    assert!(
        Plugin::of::<()>("1.0.0")
            .identifier("data")
            .suite(Suite::of::<()>().identifier("data.data").label("Data").template(template))
            .declarations()
            .is_err()
    );

    // Reject Duplicate Templates Across Suites and Nonfinite Defaults.
    let plugin = Plugin::of::<()>("1.0.0")
        .identifier("data")
        .suite(Suite::of::<()>().identifier("data.first").label("First").template(Template::of::<()>().identifier("case").label("Case")))
        .suite(Suite::of::<()>().identifier("data.first").label("First").template(Template::of::<()>().identifier("case").label("Case")));
    assert!(plugin.declarations().is_err());
    let template = Template::of::<()>().identifier("invalid").label("Invalid").parameters(Inputs {
        limit: f64::NAN,
        channels: Vec::new(),
        serial: None,
    });
    assert!(
        Plugin::of::<()>("1.0.0")
            .identifier("data")
            .suite(Suite::of::<()>().identifier("data.data").label("Data").template(template))
            .declarations()
            .is_err()
    );
    Ok(())
}
