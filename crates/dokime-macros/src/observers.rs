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
use quote::quote;
use syn::parse::Parser;
use syn::{Error, Meta, Token, punctuated::Punctuated};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn stream(attribute: TokenStream, item: TokenStream) -> TokenStream {
    match declare(attribute, &item) {
        Ok(declared) => crate::facade::contracted(declared),
        Err(refused) => {
            let refusal = refused.to_compile_error();

            quote! { #item #refusal }
        }
    }
}

pub(crate) fn observer(attribute: TokenStream, item: TokenStream) -> TokenStream {
    crate::facade::harmos_attribute("observer", attribute, item)
}

/* === Private Functions === */
fn declare(attribute: TokenStream, item: &TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let mut declared: syn::DeriveInput = syn::parse2(item.clone())?;

    crate::message::check(&declared)?;

    let mut forwarded = Punctuated::<Meta, Token![,]>::new();
    let mut foreign = false;
    let mut named = false;
    for argument in Punctuated::<Meta, Token![,]>::parse_terminated.parse2(attribute)? {
        if argument.path().is_ident("foreign") {
            if !matches!(argument, Meta::Path(_)) || foreign {
                return Err(Error::new_spanned(argument, "write `foreign` once to declare read-only access"));
            }
            foreign = true;
        } else {
            named |= argument.path().is_ident("row");
            forwarded.push(argument);
        }
    }
    crate::row::prepare(&mut declared)?;
    let name = &declared.ident;
    let (generics, types, predicates) = declared.generics.split_for_impl();
    let columns = crate::row::column_count(&declared)?;

    if foreign && columns == 0 {
        return Err(Error::new(name.span(), "a foreign stream declares the columns this plugin reads; add named fields"));
    }
    if named && columns > 0 {
        return Err(Error::new(name.span(), "a named stream with `row = Shape` must be a unit struct; put fields on Shape"));
    }
    if !named && columns == 0 {
        return Err(Error::new(name.span(), "a stream requires named row fields or `row = Shape` on a unit struct"));
    }
    let (row, row_half) = if columns == 0 {
        (quote!(<#name #types as #sdk::StreamSpec>::Row), quote!())
    } else {
        (quote!(Self), crate::row::implementation(&declared)?)
    };
    let closing = if foreign { quote!(.foreign()) } else { quote!(.strict()) };
    let published = !foreign;
    let mut stripped = crate::row::stripped(&declared);
    if columns > 0 {
        let derives = stripped
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident("derive"))
            .map(|attribute| attribute.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated))
            .collect::<Result<Vec<_>, _>>()?;
        let has = |name: &str| derives.iter().flatten().any(|path| path.segments.last().is_some_and(|part| part.ident == name));
        let mut added = Vec::new();
        if !has("Clone") {
            added.push(quote!(Clone));
        }
        if !has("Serialize") {
            added.push(quote!(#sdk::__harmos::__private::serde::Serialize));
        }
        if !has("Deserialize") {
            added.push(quote!(#sdk::__harmos::__private::serde::Deserialize));
        }
        if !added.is_empty() {
            stripped.attrs.push(syn::parse_quote!(#[derive(#(#added),*)]));
        }
        if !has("Serialize") && !has("Deserialize") {
            let serde_path =
                syn::LitStr::new(&format!("{}::__harmos::__private::serde", quote!(#sdk)).replace(' ', ""), proc_macro2::Span::call_site());
            stripped.attrs.push(syn::parse_quote!(#[serde(crate = #serde_path)]));
        }
    }
    // Named Dokime Streams Always Carry A Typed Storage Payload.
    let shape = if columns > 0 { crate::facade::harmos_attribute("message", quote!(), quote!(#stripped)) } else { quote!(#stripped) };
    let shared = crate::facade::harmos_attribute("stream", quote!(#forwarded), shape);
    let owned = (!foreign).then(|| {
        quote! {
            #[automatically_derived]
            impl #generics #sdk::telemetry::OwnedStream for #name #types #predicates {}
        }
    });
    let publishable = (!foreign && columns > 0).then(|| {
        quote! {
            #[automatically_derived]
            impl #generics #sdk::Publishable for #name #types #predicates {
                type Payload = Self;

                fn publication(self) -> Self {
                    self
                }
            }

            #[automatically_derived]
            impl #generics ::core::convert::From<#name #types> for #sdk::publication::Publication #predicates {
                fn from(row: #name #types) -> Self {
                    let payload = #sdk::Message::encode_to_vec(&row);
                    Self::Row {
                        payload,
                        stream: <#name #types as #sdk::telemetry::TelemetryStream>::stream_id(),
                        values: <#name #types as #sdk::telemetry::TelemetryRow>::into_row(row),
                    }
                }
            }
        }
    });

    Ok(quote! {
        #shared
        #row_half

        #[automatically_derived]
        impl #generics #sdk::telemetry::TelemetryStream for #name #types #predicates {
            const PUBLISHED: bool = #published;

            fn contract() -> #sdk::telemetry::TelemetryContract {
                let metadata = <Self as #sdk::StreamSpec>::metadata();
                #sdk::telemetry::TelemetryContract {
                    schema_version: <Self as #sdk::StreamSpec>::VERSION,
                    metadata: metadata.clone(),
                    ..#sdk::telemetry::TelemetryContract::declared(
                        <Self as #sdk::StreamSpec>::ID,
                        &metadata.label,
                    )
                        .columns(<#row as #sdk::telemetry::TelemetryRow>::columns())
                        .typed::<#row>()
                        #closing
                }
            }
        }
        #owned

        #[automatically_derived]
        impl #generics #sdk::execution::Listenable for #name #types #predicates {
            type Item = #row;
            type Listening = #sdk::execution::ListeningRows<Self>;

            fn subscription() -> #sdk::definition::Subscription {
                #sdk::definition::Subscription::Stream(<Self as #sdk::telemetry::TelemetryStream>::stream_id())
            }

            fn listening(
                streams: &#sdk::telemetry::Streams,
                _deliveries: &#sdk::records::Deliveries,
            ) -> Self::Listening {
                #sdk::execution::ListeningRows::new(streams)
            }
        }
        #publishable
    })
}

/* === Tests === */
