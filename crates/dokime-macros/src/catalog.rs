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
    if !attribute.is_empty() {
        return Error::new_spanned(attribute, "Dokime contribution catalogs take no arguments").to_compile_error();
    }
    match projection(item.clone()).and_then(|declared| identifiers(item).map(|identifiers| quote!(#declared #identifiers))) {
        Ok(declared) => declared,
        Err(error) => {
            let error = error.to_compile_error();
            quote!(#error)
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

fn projection(item: TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let mut declared: syn::ItemEnum = syn::parse2(item)?;
    let name = declared.ident.clone();
    let mut projected = declared.generics.clone();
    projected.params.push(syn::parse_quote!(__DokimeVisitor));
    projected.make_where_clause().predicates.push(syn::parse_quote!(__DokimeVisitor: #sdk::catalog::Visitor));
    let mut seen = std::collections::BTreeSet::new();
    let mut members = Vec::new();
    let mut entries = Vec::new();
    let mut direct_values = Vec::new();
    let mut has_nested = false;
    let mut contains = Vec::new();
    let (implementation, types, predicates) = declared.generics.split_for_impl();
    let (implementation, types, predicates) = (quote!(#implementation), quote!(#types), quote!(#predicates));
    for variant in &mut declared.variants {
        if variant.discriminant.is_some() {
            return Err(Error::new_spanned(variant, "catalog variants cannot have discriminants"));
        }
        let syn::Fields::Unnamed(fields) = &variant.fields else {
            return Err(Error::new_spanned(variant, "a catalog variant has exactly one unnamed typed field"));
        };
        if fields.unnamed.len() != 1 {
            return Err(Error::new_spanned(fields, "a catalog variant has exactly one unnamed typed field"));
        }
        let field = &fields.unnamed[0];
        if !field.attrs.is_empty() {
            return Err(Error::new_spanned(field, "catalog member fields cannot carry attributes"));
        }
        let member = &field.ty;
        if !seen.insert(member.to_token_stream().to_string()) {
            return Err(Error::new_spanned(member, "a catalog member type may appear only once"));
        }
        let nested = variant.attrs.iter().any(|attribute| attribute.path().is_ident("nested"));
        variant.attrs.retain(|attribute| !attribute.path().is_ident("nested"));
        if nested {
            has_nested = true;
            projected.make_where_clause().predicates.push(syn::parse_quote!(#member: #sdk::catalog::Catalog<__DokimeVisitor>));
            entries.push((
                quote!(<#member as #sdk::catalog::Catalog<__DokimeVisitor>>::Entries),
                quote!(<#member as #sdk::catalog::Catalog<__DokimeVisitor>>::ENTRIES),
            ));
            members.push((member.clone(), true));
            continue;
        }
        projected.make_where_clause().predicates.push(syn::parse_quote!(__DokimeVisitor: #sdk::catalog::Member<#member>));
        entries.push((
            quote!([<__DokimeVisitor as #sdk::catalog::Visitor>::Entry; 1]),
            quote!([<__DokimeVisitor as #sdk::catalog::Member<#member>>::ENTRY]),
        ));
        direct_values.push(quote!(<__DokimeVisitor as #sdk::catalog::Member<#member>>::ENTRY));
        members.push((member.clone(), false));
        contains.push(quote!(impl #implementation #sdk::catalog::Contains<#member> for #name #types #predicates {}));
    }
    let (entries_type, values) = if has_nested {
        entries.into_iter().rev().fold((quote!(()), quote!(())), |(tail_type, tail), (head_type, head)| {
            (quote!((#head_type, #tail_type)), quote!((#head, #tail)))
        })
    } else {
        let count = direct_values.len();
        (quote!([<__DokimeVisitor as #sdk::catalog::Visitor>::Entry; #count]), quote!([#(#direct_values),*]))
    };
    let (projection_impl, _, projection_where) = projected.split_for_impl();
    let inspection = [
        quote!(#sdk::__harmos::operations::WireDeclaration),
        quote!(&'static [#sdk::__harmos::operations::WireDeclaration]),
        quote!(#sdk::__harmos::catalog::OperationEntry),
        quote!(#sdk::__harmos::catalog::RequirementEntry),
        quote!(#sdk::__harmos::catalog::ObserverEntry),
        quote!(#sdk::__harmos::catalog::ListenerEntry),
        quote!(&'static [#sdk::__harmos::catalog::PublicationEntry]),
        quote!(#sdk::__harmos::Catalog),
        quote!(&'static [#sdk::__harmos::Catalog]),
    ]
    .into_iter()
    .map(|entry| {
        let mut generics = projected.clone();
        generics.make_where_clause().predicates.push(syn::parse_quote!(
            __DokimeVisitor: #sdk::catalog::Visitor<Entry = #entry>
        ));
        for (member, nested) in &members {
            if *nested {
                generics.make_where_clause().predicates.push(syn::parse_quote!(
                    #member: #sdk::catalog::InertCatalog<__DokimeVisitor, #entry>
                ));
            }
        }
        let nodes = members.iter().map(|(member, nested)| {
            if *nested {
                quote!(#sdk::catalog::Node::Nested(<#member as #sdk::catalog::InertCatalog<__DokimeVisitor, #entry>>::ENTRIES))
            } else {
                quote!(#sdk::catalog::Node::Leaf(<__DokimeVisitor as #sdk::catalog::Member<#member>>::ENTRY))
            }
        });
        let (implementation, _, predicates) = generics.split_for_impl();
        Ok::<_, Error>(quote! {
            impl #implementation #sdk::catalog::InertCatalog<__DokimeVisitor, #entry> for #name #types #predicates {
                const ENTRIES: &'static [#sdk::catalog::Node<#entry>] = &[#(#nodes),*];
            }
        })
    })
    .collect::<Result<Vec<_>, _>>()?;
    Ok(quote! {
        #[allow(dead_code, clippy::large_enum_variant, clippy::enum_variant_names)]
        #declared
        impl #projection_impl #sdk::catalog::Catalog<__DokimeVisitor> for #name #types #projection_where {
            type Entries = #entries_type;
            const ENTRIES: Self::Entries = #values;
        }
        #(#inspection)*
        #(#contains)*
    })
}

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
