/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared Dokime Message Declarations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

pub(crate) enum Kind {
    Plain,
    Row,
    Record,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn message(kind: Kind, attribute: TokenStream, item: TokenStream) -> TokenStream {
    declare(kind, attribute, item).unwrap_or_else(Error::into_compile_error)
}

pub(crate) fn check(input: &DeriveInput) -> Result<(), Error> {
    for attribute in &input.attrs {
        let Some(name) = attribute.path().segments.last() else {
            continue;
        };
        if ["message", "record", "row", "stream", "choice"].iter().any(|kind| name.ident == *kind) {
            return Err(Error::new_spanned(
                attribute,
                "declare one message shape: use message, record, row, stream or choice without stacking them",
            ));
        }
    }
    Ok(())
}

/* === Private Functions === */

fn declare(kind: Kind, attribute: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let mut input: DeriveInput = syn::parse2(item)?;
    check(&input)?;
    let (shared, domain) = match kind {
        Kind::Plain => (crate::facade::harmos_attribute("message", attribute, quote!(#input)), quote!()),
        Kind::Row => {
            crate::row::prepare(&mut input)?;
            let domain = crate::row::implementation(&input)?;
            let stripped = crate::row::stripped(&input);
            let shared = crate::facade::harmos_attribute("message", attribute, quote!(#stripped));
            (shared, domain)
        }
        Kind::Record => {
            let domain = crate::record::implementation(attribute, &input)?;
            let stripped = crate::record::stripped(&input);
            let shared = crate::facade::harmos_attribute("message", quote!(), quote!(#stripped));
            (shared, domain)
        }
    };
    Ok(crate::facade::contracted(quote! {
        #shared
        #domain
    }))
}

/* === Tests === */
