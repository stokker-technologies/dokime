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
use proc_macro2::TokenStream;

use quote::{format_ident, quote};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{
    Data, DeriveInput, Error, Expr, Fields, Ident, LitStr, Path, Token, Type,
    punctuated::Punctuated,
};
// Local Imports

/* === Constants === */
const ROLES: &[&str] = &[
    "value",
    "counter",
    "state",
    "distance",
    "latency",
    "throughput",
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct RowFieldDeclaration {
    name: Ident,
    ty: Type,
    format: Option<Expr>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn row(item: TokenStream) -> TokenStream {
    match expand(item) {
        Ok(derived) => crate::facade::contracted(derived),
        Err(refused) => refused.to_compile_error(),
    }
}

pub(crate) fn implementation(input: &DeriveInput) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let row = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let fields = declarations(input)?;
    let columns = fields.iter().map(column);
    let values = fields.iter().map(value);
    let reads = fields.iter().map(read);
    let handles = handles(&fields);

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #row #type_generics #where_clause {
            #(#handles)*
        }

        #[automatically_derived]
        impl #impl_generics #sdk::telemetry::TelemetryRow for #row #type_generics
            #where_clause
        {
            fn columns() -> ::std::vec::Vec<#sdk::telemetry::TelemetryColumn> {
                let mut fields = <Self as #sdk::__harmos::RowMetadata>::row_metadata().into_iter();
                ::std::vec![#(#columns),*]
            }

            fn into_row(self) -> ::std::collections::BTreeMap<
                ::std::string::String,
                #sdk::core::Value,
            > {
                let mut row = ::std::collections::BTreeMap::new();
                #(#values)*
                row
            }

            fn from_row(
                values: &::std::collections::BTreeMap<::std::string::String, #sdk::core::Value>,
            ) -> ::std::result::Result<Self, #sdk::FieldError> {
                ::std::result::Result::Ok(Self { #(#reads),* })
            }
        }
    })
}

pub(crate) fn stripped(input: &DeriveInput) -> DeriveInput {
    crate::arguments::stripped(input, "dokime_row_format")
}

pub(crate) fn derives_row(input: &DeriveInput) -> bool {
    input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("derive"))
        .filter_map(|attribute| {
            attribute
                .parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
                .ok()
        })
        .flatten()
        .any(|path| {
            path.segments
                .last()
                .is_some_and(|segment| segment.ident == "TelemetryRow")
        })
}

pub(crate) fn column_count(input: &DeriveInput) -> Result<usize, Error> {
    let Data::Struct(structure) = &input.data else {
        return Err(Error::new(
            input.ident.span(),
            "a stream is declared on a struct: its fields are the stream's columns",
        ));
    };

    match &structure.fields {
        Fields::Named(named) => Ok(named.named.len()),
        Fields::Unit => Ok(0),
        Fields::Unnamed(unnamed) => Err(Error::new(
            unnamed.span(),
            "a telemetry row names its fields: a column has an identifier",
        )),
    }
}

pub(crate) fn prepare(input: &mut DeriveInput) -> Result<(), Error> {
    let Data::Struct(structure) = &mut input.data else {
        return Ok(());
    };

    for field in &mut structure.fields {
        let mut attributes = Vec::new();
        let mut format_seen = false;
        for attribute in &field.attrs {
            if !attribute.path().is_ident("row") {
                attributes.push(attribute.clone());
                continue;
            }
            let entries =
                attribute.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)?;
            let mut forwarded = Punctuated::<syn::Meta, Token![,]>::new();
            for entry in entries {
                if entry.path().is_ident("format") {
                    let syn::Meta::NameValue(value) = entry else {
                        return Err(Error::new_spanned(
                            entry,
                            "write `format = fixed(1)` or another document format",
                        ));
                    };
                    if format_seen {
                        return Err(Error::new_spanned(
                            value.path,
                            "`format` is declared twice on this field",
                        ));
                    }
                    format_seen = true;
                    let value = value.value;
                    attributes.push(syn::parse_quote!(#[dokime_row_format(#value)]));
                } else {
                    if let syn::Meta::NameValue(value) = &entry
                        && value.path.is_ident("role")
                    {
                        validate_role(&value.value)?;
                    }
                    forwarded.push(entry);
                }
            }
            if !forwarded.is_empty() {
                attributes.push(syn::parse_quote!(#[row(#forwarded)]));
            }
        }
        field.attrs = attributes;
    }
    Ok(())
}

pub(crate) fn optional(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };

    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Option")
}

/* === Private Functions === */
fn expand(item: TokenStream) -> Result<TokenStream, Error> {
    let mut input: DeriveInput = syn::parse2(item)?;
    prepare(&mut input)?;

    implementation(&input)
}

fn handles(fields: &[RowFieldDeclaration]) -> Vec<TokenStream> {
    let sdk = crate::facade::root();
    let mut handles = fields
        .iter()
        .map(|field| {
            let name = &field.name;
            let method = format_ident!("column_{}", name.unraw());
            let identifier = LitStr::new(&name.unraw().to_string(), name.span());

            quote! {

                pub fn #method() -> #sdk::document::Column<Self> {
                    #sdk::document::Column::new(#identifier)
                }
            }
        })
        .collect::<Vec<_>>();

    handles.push(quote! {
                pub fn column_timestamp() -> #sdk::document::Column<Self> {
            #sdk::document::Column::new("timestamp")
        }
    });

    handles
}

fn declarations(input: &DeriveInput) -> Result<Vec<RowFieldDeclaration>, Error> {
    let named = crate::arguments::named_struct_fields(
        input,
        "a telemetry row is a struct: its fields are the stream's columns",
        "a telemetry row names its fields: a column has an identifier",
    )?;

    named
        .named
        .iter()
        .map(|field| {
            let name = field
                .ident
                .clone()
                .expect("a named field carries an identifier");
            if name.unraw() == "timestamp" {
                return Err(Error::new(
                    name.span(),
                    "`timestamp` is the automatic column every row carries; rename this field",
                ));
            }
            if name.unraw().to_string().starts_with("column_") {
                return Err(Error::new(
                    name.span(),
                    "the `column_` prefix is reserved for generated plot handles",
                ));
            }
            let format = field
                .attrs
                .iter()
                .find(|attribute| attribute.path().is_ident("dokime_row_format"))
                .map(syn::Attribute::parse_args::<Expr>)
                .transpose()?;

            Ok(RowFieldDeclaration {
                format,
                name,
                ty: field.ty.clone(),
            })
        })
        .collect()
}

fn validate_role(value: &Expr) -> Result<(), Error> {
    let written = match value {
        Expr::Path(path) => path
            .path
            .get_ident()
            .map(ToString::to_string)
            .unwrap_or_default(),
        Expr::Lit(literal) => match &literal.lit {
            syn::Lit::Str(text) => text.value(),
            _ => String::new(),
        },
        _ => String::new(),
    };
    if written == "time" {
        return Err(Error::new(
            value.span(),
            "`role = time` was removed: every row carries one automatic `timestamp` column. \
             Name it with `<Row>::column_timestamp()`.",
        ));
    }
    if !ROLES.contains(&written.as_str()) {
        return Err(Error::new(
            value.span(),
            format!("a column is read under one of {}", ROLES.join(", ")),
        ));
    }

    Ok(())
}

fn column(field: &RowFieldDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let identifier = LitStr::new(&field.name.unraw().to_string(), field.name.span());
    let ty = &field.ty;
    let declaration = quote! {
        #sdk::telemetry::row::declared_column(
            fields.find(|field| field.metadata.identifier.as_str() == #identifier)
                .expect("Harmos generated metadata for every row field"),
            <#ty as #sdk::FieldValue>::VALUE_TYPE,
        )
    };
    let declaration = match &field.format {
        // The glob is what lets `format = engineering()` name a constructor
        // with no path of its own; the surrounding block is what keeps
        // `unused_braces` quiet about it, since it is a block with a
        // statement rather than a bare group.
        Some(format) => quote! {
            #declaration.with_format(#sdk::definition::authored_format({
                use #sdk::document::format::*;
                #format
            }))
        },
        None => declaration,
    };

    // An optional field is a column that may have no measurement this tick:
    // saying so is what keeps the host from refusing an honest absence.
    if optional(&field.ty) {
        quote!(#declaration.nullable())
    } else {
        declaration
    }
}

fn value(field: &RowFieldDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let name = &field.name;
    let identifier = LitStr::new(&name.unraw().to_string(), name.span());

    quote! {
        row.insert(
            ::std::string::ToString::to_string(#identifier),
            #sdk::FieldValue::into_value(self.#name),
        );
    }
}

fn read(field: &RowFieldDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let name = &field.name;
    let identifier = LitStr::new(&name.unraw().to_string(), name.span());

    quote!(#name: #sdk::telemetry::field(values, #identifier)?)
}

/* === Tests === */
