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

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Error = Box<dyn std::error::Error>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Export Domain Defaults Alongside The Generated Operation Bindings.
pub(super) fn export_application_settings(directory: &std::path::Path) -> Result<(), Error> {
    let settings = dokime::domain::model::settings::application::ApplicationSettings::default();
    let bytes = serde_json::to_vec_pretty(&settings)?;
    std::fs::create_dir_all(directory)?;
    dokime::harmos::filesystem::publication::write(&directory.join("application-settings.json"), &bytes)
        .map_err(|error| invalid(error.to_string()))?;
    let models = preference_models()?;
    std::fs::write(directory.join("application-settings-models.json"), serde_json::to_vec_pretty(&models)?)?;
    std::fs::write(directory.join("application-settings.ts"), typescript_models(&models)?)?;
    Ok(())
}

/* === Private Functions === */

fn invalid(message: String) -> Error {
    message.into()
}

// Parse The Domain Owners So Native JSON Bindings Cannot Drift Into Handwritten Mirrors.
fn preference_models() -> Result<serde_json::Value, Error> {
    let mut items = std::collections::BTreeMap::new();
    for source in dokime::domain::model::settings::APPLICATION_SETTINGS_SOURCES {
        let file = syn::parse_file(source).map_err(|error| invalid(error.to_string()))?;
        for item in file.items {
            let name = match &item {
                syn::Item::Struct(item) => item.ident.to_string(),
                syn::Item::Enum(item) => item.ident.to_string(),
                _ => continue,
            };
            items.insert(name, item);
        }
    }
    let mut pending = vec![
        "ApplicationSettings".to_owned(),
        "SettingsSnapshot".to_owned(),
        "UpdateSettings".to_owned(),
        "SettingsScope".to_owned(),
        "Document".to_owned(),
        "PairingFile".to_owned(),
        "PairedClient".to_owned(),
        "PairedClientPage".to_owned(),
        "CreatedPairing".to_owned(),
    ];
    let mut models = serde_json::Map::new();
    while let Some(name) = pending.pop() {
        if models.contains_key(&name) {
            continue;
        }
        let item = items.get(&name).ok_or_else(|| invalid(format!("missing preference declaration {name}")))?;
        let model = match item {
            syn::Item::Struct(item) => {
                let mut fields = serde_json::Map::new();
                for field in &item.fields {
                    let name = field.ident.as_ref().ok_or_else(|| invalid("preference tuple fields are unsupported".into()))?;
                    fields.insert(name.to_string(), preference_type(&field.ty, &mut pending)?);
                }
                serde_json::json!({"fields": fields})
            }
            syn::Item::Enum(item) => {
                let mut snake = false;
                for attribute in &item.attrs {
                    if attribute.path().is_ident("serde") {
                        attribute
                            .parse_nested_meta(|meta| {
                                if meta.path.is_ident("rename_all") {
                                    snake = meta.value()?.parse::<syn::LitStr>()?.value() == "snake_case";
                                }
                                Ok(())
                            })
                            .map_err(|error| invalid(error.to_string()))?;
                    }
                }
                let mut variants = Vec::new();
                for variant in &item.variants {
                    if !matches!(variant.fields, syn::Fields::Unit) {
                        return Err(invalid("preference enums must be scalar".into()));
                    }
                    let name = variant.ident.to_string();
                    let name = if snake { snake_case(&name) } else { name };
                    variants.push(name);
                }
                serde_json::json!({"variants": variants})
            }
            _ => unreachable!(),
        };
        models.insert(name, model);
    }
    Ok(serde_json::Value::Object(models))
}

fn preference_type(value: &syn::Type, pending: &mut Vec<String>) -> Result<serde_json::Value, Error> {
    let syn::Type::Path(value) = value else {
        return Err(invalid("unsupported preference field type".into()));
    };
    let segment = value.path.segments.last().ok_or_else(|| invalid("empty preference type".into()))?;
    let name = segment.ident.to_string();
    if matches!(name.as_str(), "Option" | "Vec") {
        let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return Err(invalid("missing preference type argument".into()));
        };
        let Some(syn::GenericArgument::Type(inner)) = arguments.args.first() else {
            return Err(invalid("invalid preference type argument".into()));
        };
        let inner = preference_type(inner, pending)?;
        return Ok(if name == "Option" { serde_json::json!({"optional": inner}) } else { serde_json::json!({"list": inner}) });
    }
    let primitive = match name.as_str() {
        "String" => "string",
        "bool" => "boolean",
        "u8" | "u32" | "u64" => "number",
        _ => {
            pending.push(name.clone());
            return Ok(serde_json::Value::String(name));
        }
    };
    Ok(serde_json::Value::String(primitive.into()))
}

fn snake_case(value: &str) -> String {
    let mut output = String::new();
    for (index, character) in value.chars().enumerate() {
        if index > 0 && character.is_uppercase() {
            output.push('_');
        }
        output.extend(character.to_lowercase());
    }
    output
}

fn typescript_models(models: &serde_json::Value) -> Result<String, Error> {
    use std::fmt::Write;
    let mut output = String::from("// Generated From Dokime Domain Declarations. Do Not Edit.\n");
    for (name, model) in models.as_object().expect("model inventory is an object") {
        if let Some(variants) = model.get("variants") {
            let variants = variants
                .as_array()
                .expect("variant inventory is an array")
                .iter()
                .map(serde_json::Value::to_string)
                .collect::<Vec<_>>()
                .join(" | ");
            writeln!(output, "export type {name} = {variants};").expect("writing to a string");
        } else {
            writeln!(output, "export interface {name} {{").expect("writing to a string");
            for (field, value) in model["fields"].as_object().expect("field inventory is an object") {
                writeln!(output, "  {field}: {};", typescript_type(value)?).expect("writing to a string");
            }
            output.push_str("}\n");
        }
    }
    Ok(output)
}

fn typescript_type(value: &serde_json::Value) -> Result<String, Error> {
    if let Some(name) = value.as_str() {
        return Ok(name.into());
    }
    if let Some(inner) = value.get("optional") {
        return Ok(format!("{} | null", typescript_type(inner)?));
    }
    if let Some(inner) = value.get("list") {
        return Ok(format!("Array<{}>", typescript_type(inner)?));
    }
    Err(invalid("invalid preference model type".into()))
}

/* === Tests === */
