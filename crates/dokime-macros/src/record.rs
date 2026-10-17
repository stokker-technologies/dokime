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
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{DeriveInput, Error, Expr, Ident, LitStr, Type};
// Local Imports
use crate::arguments::Arguments;

/* === Constants === */
const FIELD_ARGUMENTS: &[&str] = &["label", "unit", "format", "message", "description"];

const SEVERITIES: &[(&str, &str)] = &[
    ("debug", "Debug"),
    ("info", "Info"),
    ("warning", "Warning"),
    ("error", "Error"),
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct RecordFieldDeclaration {
    name: Ident,
    ty: Type,
    label: TokenStream,
    described: TokenStream,
    unit: Option<Expr>,
    format: Option<Expr>,
    message: bool,
}

#[derive(Default)]
struct FieldArguments {
    label: Option<TokenStream>,
    unit: Option<Expr>,
    format: Option<Expr>,
    message: bool,
    description: Option<Expr>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn implementation(
    attribute: TokenStream,
    input: &DeriveInput,
) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let arguments = Arguments::read(
        attribute,
        "a record",
        &["id", "label", "severity", "description"],
    )?;
    let identifier = arguments
        .required(
            "id",
            "a record declares the id its occurrences are filed under",
        )?
        .clone();
    let label = arguments
        .required("label", "an operator reads the record by its label")?
        .clone();
    let severity = match arguments.value("severity") {
        Some(written) => declared_severity(written, "a record is filed")?,
        None => quote!(#sdk::records::RecordSeverity::Info),
    };
    let described = crate::arguments::description(arguments.value("description"), &input.attrs)
        .map(|text| quote!(.described(#text)));
    let fields = declarations(input)?;
    let name = &input.ident;
    let (generics, types, predicates) = input.generics.split_for_impl();
    let contract_fields = fields.iter().map(contract_field);
    let details = fields.iter().map(detail);
    let reads = fields.iter().map(read);
    let handles = handles(&fields, &identifier);
    let message = match fields.iter().find(|field| field.message) {
        Some(field) => {
            let name = &field.name;
            quote!(#sdk::records::message_of(&self.#name, #label))
        }
        None => quote!(::std::string::ToString::to_string(#label)),
    };

    Ok(quote! {
        #[automatically_derived]
        impl #generics #name #types #predicates {
            #handles
        }

        #[automatically_derived]
        impl #generics #sdk::records::DeclaredRecord for #name #types #predicates {
            const RECORD_TYPE: &'static str = #identifier;
            const SEVERITY: #sdk::records::RecordSeverity = #severity;

            fn contract() -> #sdk::records::RecordContract {
                #sdk::records::RecordContract::declared(#identifier, #label)
                    #described
                    .severity(#severity)
                    #(.field(#contract_fields))*
                    .strict()
            }

            fn message(&self) -> ::std::string::String {
                #message
            }

            fn into_details(self) -> ::std::collections::BTreeMap<
                ::std::string::String,
                #sdk::Value,
            > {
                let mut details = ::std::collections::BTreeMap::new();
                #(#details)*
                details
            }

            fn from_record(
                record: &#sdk::__domain::model::records::Record,
            ) -> #sdk::Result<Self> {
                if record.record_type.as_str() != <Self as #sdk::records::DeclaredRecord>::RECORD_TYPE {
                    return ::std::result::Result::Err(
                        #sdk::Error::record_mismatched(
                            <Self as #sdk::records::DeclaredRecord>::RECORD_TYPE,
                            &record.record_type,
                        ),
                    );
                }

                <Self as #sdk::records::DeclaredRecord>::from_details(&record.details).map_err(
                    |error| #sdk::Error::record_field(
                        <Self as #sdk::records::DeclaredRecord>::RECORD_TYPE,
                        error,
                    ),
                )
            }

            fn from_details(
                values: &::std::collections::BTreeMap<::std::string::String, #sdk::Value>,
            ) -> ::std::result::Result<Self, #sdk::FieldError> {
                ::std::result::Result::Ok(Self { #(#reads),* })
            }
        }

        #[automatically_derived]
        impl #generics #sdk::execution::Listenable for #name #types #predicates {
            type Item = Self;
            type Listening = #sdk::execution::ListeningRecords<Self>;

            fn subscription() -> #sdk::definition::Subscription {
                #sdk::definition::Subscription::Record(<Self as #sdk::records::DeclaredRecord>::contract().record_type)
            }

            fn listening(
                _streams: &#sdk::telemetry::Streams,
                deliveries: &#sdk::records::Deliveries,
            ) -> Self::Listening {
                #sdk::execution::ListeningRecords::new(deliveries)
            }
        }

        #[automatically_derived]
        impl #generics #sdk::Publishable for #name #types #predicates {
            type Payload = #sdk::__domain::model::records::RecordPublishRequest;

            fn publication(self) -> Self::Payload {
                <Self as #sdk::records::DeclaredRecord>::request(self)
            }
        }
    })
}

// A prompt's level reads on the same scale, so it is parsed here once.
pub(crate) fn declared_severity(value: &Expr, subject: &str) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
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
    let Some((_, variant)) = SEVERITIES.iter().find(|(name, _)| *name == written) else {
        return Err(Error::new(
            value.span(),
            format!(
                "{subject} at one of {}",
                SEVERITIES
                    .iter()
                    .map(|(name, _)| *name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    };
    let variant = Ident::new(variant, value.span());

    Ok(quote!(#sdk::records::RecordSeverity::#variant))
}

/* === Private Functions === */
fn handles(fields: &[RecordFieldDeclaration], identifier: &Expr) -> TokenStream {
    let sdk = crate::facade::root();
    let accessors = fields.iter().map(|field| {
        let name = &field.name;
        let detail = LitStr::new(&name.to_string(), name.span());

        quote! {

            pub fn #name() -> #sdk::document::RecordField {
                #sdk::document::RecordField::new(#identifier, #detail)
            }
        }
    });

    quote! {
                pub fn marker() -> #sdk::document::RecordMarker {
            #sdk::document::RecordMarker::new(#identifier)
        }

        #(#accessors)*
    }
}

fn declarations(input: &DeriveInput) -> Result<Vec<RecordFieldDeclaration>, Error> {
    let named = crate::arguments::named_struct_fields(
        input,
        "a record is a struct: its fields are the fields of its contract",
        "a record names its fields: a declared field has an identifier",
    )?;
    let mut declared = Vec::new();
    let mut message: Option<proc_macro2::Span> = None;
    for field in &named.named {
        let name = field
            .ident
            .clone()
            .expect("a named field carries an identifier");
        let written = arguments(field)?;
        if written.message {
            if let Some(first) = message {
                let mut refusal = Error::new(
                    name.span(),
                    "a record says one thing: only one field is its message",
                );
                refusal.combine(Error::new(first, "the message was already marked here"));
                return Err(refusal);
            }
            message = Some(name.span());
        }
        declared.push(RecordFieldDeclaration {
            label: written.label.unwrap_or_else(|| {
                let titled = crate::arguments::titled(&name.to_string());

                quote!(#titled)
            }),
            described: crate::arguments::described(written.description.as_ref(), &field.attrs),
            unit: written.unit,
            format: written.format,
            message: written.message,
            name,
            ty: field.ty.clone(),
        });
    }

    Ok(declared)
}

fn arguments(field: &syn::Field) -> Result<FieldArguments, Error> {
    let mut written = FieldArguments::default();
    let read = crate::arguments::field_arguments(
        field,
        "record",
        "a record field argument",
        FIELD_ARGUMENTS,
        &["message"],
    )?;
    written.message = read.flag("message");
    for (name, value) in read.values() {
        match name.to_string().as_str() {
            "label" => written.label = Some(value.to_token_stream()),
            "unit" => written.unit = Some(value),
            "format" => written.format = Some(value),
            "description" => written.description = Some(value),
            other => unreachable!("`{other}` is refused where the arguments are read"),
        }
    }

    Ok(written)
}

fn contract_field(field: &RecordFieldDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let identifier = LitStr::new(&field.name.to_string(), field.name.span());
    let label = &field.label;
    let ty = &field.ty;
    let described = &field.described;
    let declaration = quote! {
        #sdk::records::RecordField::new(
            #sdk::Metadata::new(
                #sdk::Identifier::declared(#identifier, "record field"),
                #label,
                #described,
            ),
            <#ty as #sdk::FieldValue>::VALUE_TYPE,
        )
    };
    let declaration = match &field.unit {
        Some(unit) => quote!(#declaration.with_unit(#sdk::definition::authored_unit(#unit))),
        None => declaration,
    };
    let declaration = match &field.format {
        // See `row::column` for why this is a glob inside a block rather than
        // a bare expression.
        Some(format) => quote! {
            #declaration.with_format(#sdk::definition::authored_format({
                use #sdk::document::format::*;
                #format
            }))
        },
        None => declaration,
    };

    // An optional field is a fact the occurrence may not have known: saying so
    // is what keeps the host from refusing an honest absence.
    if crate::row::optional(&field.ty) {
        quote!(#declaration.nullable())
    } else {
        declaration
    }
}

fn detail(field: &RecordFieldDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let name = &field.name;
    let identifier = LitStr::new(&name.to_string(), name.span());

    quote! {
        details.insert(
            ::std::string::ToString::to_string(#identifier),
            #sdk::FieldValue::into_value(self.#name),
        );
    }
}

fn read(field: &RecordFieldDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let name = &field.name;
    let identifier = LitStr::new(&name.to_string(), name.span());

    quote!(#name: #sdk::records::field(values, #identifier)?)
}

pub(crate) fn stripped(input: &DeriveInput) -> DeriveInput {
    crate::arguments::stripped(input, "record")
}

/* === Tests === */
