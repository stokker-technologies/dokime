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
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Error, Expr, Field, Fields, FieldsNamed, Ident, Meta, MetaList, Token, Type};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, Default)]
pub(crate) struct FieldEntries {
    flags: Vec<Ident>,
    values: Vec<(Ident, Expr)>,
}

pub(crate) struct Arguments {
    span: Span,
    entries: Vec<Meta>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Shape {
    Required,
    Optional,
    List,
    OptionalList,
}

/* === Traits === */

/* === Implementations === */

impl Arguments {
    pub(crate) fn read(attribute: TokenStream, kind: &str, known: &[&str]) -> Result<Self, Error> {
        let span = attribute.span();
        let entries = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(attribute)?.into_iter().collect::<Vec<_>>();
        for entry in &entries {
            let Some(name) = entry.path().get_ident() else {
                return Err(Error::new(entry.span(), "an argument is one bare name"));
            };
            if !known.contains(&name.to_string().as_str()) {
                let accepted = if known.is_empty() { "no arguments".to_owned() } else { known.join(", ") };

                return Err(Error::new(name.span(), format!("`{name}` is not {kind} argument; {kind} takes {accepted}")));
            }
        }

        Ok(Self { span, entries })
    }

    pub(crate) fn value(&self, name: &str) -> Option<&Expr> {
        self.entries.iter().find_map(|entry| match entry {
            Meta::NameValue(named) if named.path.is_ident(name) => Some(&named.value),
            _ => None,
        })
    }

    pub(crate) fn flag(&self, name: &str) -> Result<bool, Error> {
        let mut matching = self.entries.iter().filter(|entry| entry.path().is_ident(name));
        let first = matching.next();
        if let Some(duplicate) = matching.next() {
            return Err(Error::new_spanned(duplicate, format!("declare `{name}` only once")));
        }
        match first {
            Some(Meta::Path(_)) => Ok(true),
            Some(other) => Err(Error::new_spanned(other, format!("declare the bare flag `{name}`"))),
            None => Ok(false),
        }
    }

    pub(crate) fn catalog(&self, name: &str) -> Result<Option<&Expr>, Error> {
        let mut matching = self.entries.iter().filter(|entry| entry.path().is_ident(name));
        let first = matching.next();
        if let Some(duplicate) = matching.next() {
            return Err(Error::new_spanned(duplicate, format!("declare `{name}` only once")));
        }

        match first {
            Some(Meta::NameValue(named)) => Ok(Some(&named.value)),
            Some(other) => Err(Error::new_spanned(other, format!("declare a named catalog with `{name} = Catalog`"))),
            None => Ok(None),
        }
    }

    pub(crate) fn identifier(&self, name: &Ident) -> Result<Expr, Error> {
        match self.entries.iter().find(|entry| entry.path().is_ident("id")) {
            Some(Meta::NameValue(value)) => Ok(value.value.clone()),
            Some(entry) => Err(Error::new(entry.span(), "use `id = …` to specify an identity")),
            None => {
                let value = name.unraw().to_string();
                if !value.starts_with(|c: char| c.is_ascii_alphanumeric()) || !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    return Err(Error::new(name.span(), "the item name is not a valid Dokime identifier; specify `id = …`"));
                }
                Ok(syn::parse_quote!(#value))
            }
        }
    }

    pub(crate) fn required(&self, name: &str, purpose: &str) -> Result<&Expr, Error> {
        self.value(name).ok_or_else(|| Error::new(self.span, format!("`{name} = …` is required: {purpose}")))
    }

    pub(crate) fn set(&self, name: &str) -> Option<&MetaList> {
        self.entries.iter().find_map(|entry| match entry {
            Meta::List(list) if list.path.is_ident(name) => Some(list),
            _ => None,
        })
    }
}

impl FieldEntries {
    pub(crate) fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|flag| flag == name)
    }

    pub(crate) fn values(self) -> Vec<(Ident, Expr)> {
        self.values
    }
}

/* === Public Functions === */

pub(crate) fn documentation(attributes: &[Attribute]) -> String {
    let mut paragraphs: Vec<String> = Vec::new();
    let mut paragraph = String::new();

    for line in attributes.iter().filter_map(documented) {
        let line = line.trim();

        if line.is_empty() {
            if !paragraph.is_empty() {
                paragraphs.push(std::mem::take(&mut paragraph));
            }

            continue;
        }
        if !paragraph.is_empty() {
            paragraph.push(' ');
        }
        paragraph.push_str(line);
    }

    if !paragraph.is_empty() {
        paragraphs.push(paragraph);
    }

    paragraphs.join("\n")
}

pub(crate) fn description(written: Option<&Expr>, attributes: &[Attribute]) -> Option<TokenStream> {
    match written {
        Some(expr) => Some(quote!(#expr)),
        None => {
            let text = documentation(attributes);

            (!text.is_empty()).then(|| quote!(#text))
        }
    }
}

pub(crate) fn described(written: Option<&Expr>, attributes: &[Attribute]) -> TokenStream {
    match description(written, attributes) {
        Some(text) => quote!(::core::option::Option::Some(
            ::std::string::ToString::to_string(#text)
        )),
        None => quote!(::core::option::Option::<::std::string::String>::None),
    }
}

pub(crate) fn type_name(ty: &Type) -> Result<&Ident, Error> {
    if let Type::Path(path) = ty
        && path.qself.is_none()
        && let Some(segment) = path.path.segments.last()
    {
        return Ok(&segment.ident);
    }
    Err(Error::new(ty.span(), "a declaration names a concrete type"))
}

pub(crate) fn wire_identity(written: Option<Expr>, name: &Ident) -> Result<Expr, Error> {
    let (value, span) = match &written {
        Some(Expr::Lit(literal)) => match &literal.lit {
            syn::Lit::Str(text) => (text.value(), literal.span()),
            other => return Err(Error::new(other.span(), "an id is a string")),
        },
        Some(_) => return Ok(written.expect("the expression was matched")),
        None => (name.unraw().to_string(), name.span()),
    };
    if !value.starts_with(|first: char| first.is_ascii_lowercase())
        || !value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'.' | b'_'))
    {
        return Err(Error::new(
            span,
            format!(
                "`{value}` is not an id: a wire identity is lowercase ASCII letters, digits, dots, hyphens, and underscores, starting with a letter; specify `id = …`"
            ),
        ));
    }

    Ok(written.unwrap_or_else(|| syn::parse_quote!(#value)))
}

pub(crate) fn pairs(set: &MetaList) -> Result<Vec<(Ident, Expr)>, Error> {
    let entries = set.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    entries
        .into_iter()
        .map(|entry| match entry {
            Meta::NameValue(named) => {
                let key = named.path.get_ident().cloned().ok_or_else(|| Error::new(named.path.span(), "an entry key is one bare name"))?;
                Ok((key, named.value))
            }
            other => Err(Error::new(other.span(), "an entry is `key = expression`")),
        })
        .collect()
}

pub(crate) fn elements(value: &Expr) -> Result<Vec<Expr>, Error> {
    match value {
        Expr::Array(array) => Ok(array.elems.iter().cloned().collect()),
        other => Err(Error::new(other.span(), "this argument is a list: write it as `[\"one\", \"two\"]`")),
    }
}

pub(crate) fn metadata(identifier: &Expr, label: &Expr, written: Option<&Expr>, attributes: &[Attribute]) -> TokenStream {
    let sdk = crate::facade::root();
    let about = described(written, attributes);

    quote!(#sdk::definition::authored_metadata(#identifier, #label, #about))
}

pub(crate) fn field_arguments(
    field: &Field, attribute: &str, noun: &str, accepted: &[&str], flags: &[&str],
) -> Result<FieldEntries, Error> {
    let mut read = FieldEntries::default();
    for written in field.attrs.iter().filter(|written| written.path().is_ident(attribute)) {
        for entry in written.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
            let Some(name) = entry.path().get_ident().cloned() else {
                return Err(Error::new(entry.span(), "an argument is one bare name"));
            };
            let spelled = name.to_string();
            let bare = flags.contains(&spelled.as_str());
            let known = match &entry {
                Meta::Path(_) => bare,
                Meta::NameValue(_) => accepted.contains(&spelled.as_str()) && !bare,
                Meta::List(_) => false,
            };
            if !known {
                return Err(Error::new(name.span(), format!("`{spelled}` is not {noun}; a field takes {}", accepted.join(", "))));
            }
            match entry {
                Meta::Path(_) => read.flags.push(name),
                Meta::NameValue(named) => read.values.push((name, named.value)),
                Meta::List(_) => unreachable!("a set argument is refused above"),
            }
        }
    }

    Ok(read)
}

pub(crate) fn shape(declared: &Type) -> (Shape, Type) {
    match (wrapper(declared, "Option"), wrapper(declared, "Vec")) {
        (Some(inner), _) => match wrapper(&inner, "Vec") {
            Some(element) => (Shape::OptionalList, element),
            None => (Shape::Optional, inner),
        },
        (None, Some(element)) => (Shape::List, element),
        (None, None) => (Shape::Required, declared.clone()),
    }
}

pub(crate) fn wrapper(declared: &Type, name: &str) -> Option<Type> {
    let Type::Path(path) = declared else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != name {
        return None;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    arguments.args.iter().find_map(|argument| match argument {
        syn::GenericArgument::Type(inner) => Some(inner.clone()),
        _ => None,
    })
}

pub(crate) fn named_fields<'a>(fields: &'a Fields, unnamed: &str) -> Result<&'a FieldsNamed, Error> {
    match fields {
        Fields::Named(named) => Ok(named),
        other => Err(Error::new(other.span(), unnamed)),
    }
}

pub(crate) fn named_struct_fields<'a>(input: &'a DeriveInput, not_a_struct: &str, unnamed: &str) -> Result<&'a FieldsNamed, Error> {
    let Data::Struct(structure) = &input.data else {
        return Err(Error::new(input.ident.span(), not_a_struct));
    };

    named_fields(&structure.fields, unnamed)
}

pub(crate) fn stripped(input: &DeriveInput, attribute: &str) -> DeriveInput {
    let mut emitted = input.clone();
    let read = |attrs: &mut Vec<Attribute>| {
        attrs.retain(|written| !written.path().is_ident(attribute));
    };
    read(&mut emitted.attrs);
    match &mut emitted.data {
        Data::Struct(structure) => {
            for field in &mut structure.fields {
                read(&mut field.attrs);
            }
        }
        Data::Enum(enumeration) => {
            for variant in &mut enumeration.variants {
                read(&mut variant.attrs);
                for field in &mut variant.fields {
                    read(&mut field.attrs);
                }
            }
        }
        Data::Union(contents) => {
            for field in &mut contents.fields.named {
                read(&mut field.attrs);
            }
        }
    }

    emitted
}

pub(crate) fn titled(name: &str) -> String {
    name.split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut characters = word.chars();
            match characters.next() {
                Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/* === Private Functions === */

fn documented(attribute: &Attribute) -> Option<String> {
    let Meta::NameValue(named) = &attribute.meta else {
        return None;
    };
    if !named.path.is_ident("doc") {
        return None;
    }
    let Expr::Lit(literal) = &named.value else {
        return None;
    };
    let syn::Lit::Str(text) = &literal.lit else {
        return None;
    };

    Some(text.value())
}

/* === Tests === */
