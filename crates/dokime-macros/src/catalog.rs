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
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::Error;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn catalog(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let declared = crate::facade::harmos_attribute("catalog", attribute, item.clone());
    match identifiers(item) {
        Ok(identifiers) => quote!(#declared #identifiers),
        Err(error) => {
            let error = error.to_compile_error();
            quote!(#declared #error)
        }
    }
}

pub(crate) fn argument(arguments: &crate::arguments::Arguments, name: &str) -> Result<TokenStream, Error> {
    match arguments.catalog(name)? {
        Some(value) => {
            let path: syn::Path = syn::parse2(value.to_token_stream())?;
            Ok(quote!(#path))
        }
        None => Ok(quote!(())),
    }
}

pub(crate) fn assert_unique(catalog: &TokenStream, kind: &str) -> TokenStream {
    let sdk = crate::facade::root();
    let kind = Ident::new(kind, Span::call_site());
    quote! {
        const _: () = {
            const IDENTIFIERS: &[#sdk::catalog::Node<&str>] =
                <#catalog as #sdk::catalog::IdentifierCatalog<#sdk::catalog::#kind>>::IDENTIFIERS;
            const COUNT: usize = #sdk::__harmos::catalog::count(IDENTIFIERS);
            const FLAT: [&str; COUNT] = #sdk::catalog::flatten(IDENTIFIERS, "");
            #sdk::catalog::assert_unique(&FLAT);
        };
    }
}

/* === Private Functions === */

fn identifiers(item: TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let declared: syn::ItemEnum = syn::parse2(item)?;
    let name = &declared.ident;
    let mut generics = declared.generics.clone();
    generics.params.push(syn::parse_quote!(__DokimeIdentifiers));
    generics.make_where_clause().predicates.push(syn::parse_quote!(
        __DokimeIdentifiers: #sdk::catalog::Visitor<Entry = #sdk::catalog::Node<&'static str>>
    ));
    let mut nodes = Vec::new();
    for variant in &declared.variants {
        let Some(field) = variant.fields.iter().next() else { continue };
        let member = &field.ty;
        let nested = variant.attrs.iter().any(|attribute| attribute.path().is_ident("nested"));
        if nested {
            generics.make_where_clause().predicates.push(syn::parse_quote!(
                #member: #sdk::catalog::IdentifierCatalog<__DokimeIdentifiers>
            ));
            nodes.push(quote!(#sdk::catalog::Node::Nested(
                <#member as #sdk::catalog::IdentifierCatalog<__DokimeIdentifiers>>::IDENTIFIERS
            )));
            continue;
        }
        generics.make_where_clause().predicates.push(syn::parse_quote!(
            __DokimeIdentifiers: #sdk::catalog::Member<#member>
        ));
        nodes.push(quote!(<__DokimeIdentifiers as #sdk::catalog::Member<#member>>::ENTRY));
    }
    let (implementation, _, predicates) = generics.split_for_impl();
    let (_, types, _) = declared.generics.split_for_impl();
    Ok(quote! {
        impl #implementation #sdk::catalog::IdentifierCatalog<__DokimeIdentifiers> for #name #types #predicates {
            const IDENTIFIERS: &'static [#sdk::catalog::Node<&'static str>] = &[#(#nodes),*];
        }
    })
}

/* === Tests === */
