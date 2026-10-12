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

    if crate::row::derives_row(&declared) {
        return Err(Error::new(
            declared.ident.span(),
            "stream already describes this row: remove `TelemetryRow` from its derives",
        ));
    }

    let mut forwarded = Punctuated::<Meta, Token![,]>::new();
    let mut foreign = false;
    let mut named = false;
    for argument in Punctuated::<Meta, Token![,]>::parse_terminated.parse2(attribute)? {
        if argument.path().is_ident("foreign") {
            if !matches!(argument, Meta::Path(_)) || foreign {
                return Err(Error::new_spanned(
                    argument,
                    "write `foreign` once to declare read-only access",
                ));
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
        return Err(Error::new(
            name.span(),
            "a foreign stream declares the columns this plugin reads; add named fields",
        ));
    }
    if named && columns > 0 {
        return Err(Error::new(
            name.span(),
            "a named stream with `row = Shape` must be a unit struct; put fields on Shape",
        ));
    }
    if !named && columns == 0 {
        return Err(Error::new(
            name.span(),
            "a stream requires named row fields or `row = Shape` on a unit struct",
        ));
    }
    let (row, row_half) = if columns == 0 {
        (quote!(<#name #types as #sdk::StreamSpec>::Row), quote!())
    } else {
        (quote!(Self), crate::row::implementation(&declared)?)
    };
    let closing = if foreign {
        quote!(.foreign())
    } else {
        quote!(.strict())
    };
    let stripped = crate::row::stripped(&declared);
    let shared = crate::facade::harmos_attribute("stream", quote!(#forwarded), quote!(#stripped));
    let owned = (!foreign).then(|| {
        quote! {
            #[automatically_derived]
            impl #generics #sdk::telemetry::OwnedStream for #name #types #predicates {}
        }
    });
    let publishable = (!foreign && columns > 0).then(|| quote! {
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
                Self::Row {
                    stream: <#name #types as #sdk::telemetry::TelemetryStream>::stream_id(),
                    values: <#name #types as #sdk::telemetry::TelemetryRow>::into_row(row),
                }
            }
        }
    });

    Ok(quote! {
        #shared
        #row_half

        #[automatically_derived]
        impl #generics #sdk::telemetry::TelemetryStream for #name #types #predicates {
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
