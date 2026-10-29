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
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Error, GenericArgument, ItemImpl, Meta, Path, PathArguments, Token, Type};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
struct Contributed {
    suites: Option<Type>,
    evidence: Option<Type>,
    listeners: Option<Type>,
    records: Option<Type>,
}

struct RequiredEntry {
    path: Path,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl syn::parse::Parse for RequiredEntry {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let path: Path = input.parse()?;
        if input.peek(Token![:]) {
            let _: Token![:] = input.parse()?;
            // The grants are forwarded untouched; only the operation is read.
            while !input.is_empty() && !input.peek(Token![,]) {
                let _: proc_macro2::TokenTree = input.parse()?;
            }
        }
        Ok(Self { path })
    }
}

/* === Public Functions === */

pub(crate) fn plugin(attribute: TokenStream, item: TokenStream) -> TokenStream {
    match expand(attribute, &item) {
        Ok(expanded) => crate::facade::contracted(expanded),
        Err(error) => {
            let error = error.to_compile_error();
            quote!(#item #error)
        }
    }
}

/* === Private Functions === */

fn expand(attribute: TokenStream, item: &TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let mut block: ItemImpl = syn::parse2(item.clone())?;
    let Type::Path(owner) = block.self_ty.as_ref() else {
        return Err(Error::new(block.self_ty.span(), "`#[dokime::plugin]` belongs on `impl dokime::Plugin for ConcreteType`"));
    };
    let owner = owner
        .path
        .segments
        .last()
        .map(|segment| segment.ident.clone())
        .ok_or_else(|| Error::new(block.self_ty.span(), "a plugin is a named type"))?;
    let arguments = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(attribute)?;
    owner_segment(&arguments)?;
    let contributed = contributed(&arguments)?;
    let mut generated = Vec::new();
    if let Some(suites) = &contributed.suites {
        generated.push(crate::catalog::assert_unique(&quote!(#suites), "SuiteIdentifiers"));
        generated.push(crate::catalog::assert_unique(&quote!(#suites), "SuiteTemplateIdentifiers"));
    }
    if let Some(streams) = &contributed.evidence {
        generated.push(crate::catalog::assert_unique(&quote!(#streams), "StreamIdentifiers"));
    }
    if let Some(records) = &contributed.records {
        generated.push(crate::catalog::assert_unique(&quote!(#records), "RecordIdentifiers"));
    }
    let mut catalogs = Vec::new();
    if let Some(suites) = &contributed.suites {
        catalogs.push(quote!(#sdk::catalog::Node::Nested(
            <#suites as #sdk::catalog::InertCatalog<#sdk::catalog::SuiteCatalogs, &'static [#sdk::__harmos::Catalog]>>::ENTRIES
        )));
    }
    if let Some(listeners) = &contributed.listeners {
        catalogs.push(quote!(#sdk::catalog::Node::Leaf(&[#listeners::CATALOG])));
    }
    let inventory = block
        .items
        .iter_mut()
        .find_map(|item| match item {
            syn::ImplItem::Const(item) if item.ident == "CATALOGS" => Some(item),
            _ => None,
        })
        .ok_or_else(|| Error::new(block.self_ty.span(), "a plugin declares its operation catalogs in const CATALOGS"))?;
    let authored = inventory.expr.clone();
    inventory.expr = syn::parse_quote!({
        const AUTHORED: &[#sdk::__harmos::Catalog] = #authored;
        const CONTRIBUTED: &[#sdk::catalog::Node<&'static [#sdk::__harmos::Catalog]>] = &[#(#catalogs),*];
        const COUNT: usize = AUTHORED.len() + #sdk::catalog::contribution_catalog_count(CONTRIBUTED);
        const CATALOGS: [#sdk::__harmos::Catalog; COUNT] = #sdk::catalog::plugin_catalogs(AUTHORED, CONTRIBUTED);
        &CATALOGS
    });
    let mut forwarded = vec![streams(&contributed, &block.self_ty)];
    for argument in arguments {
        match &argument {
            Meta::List(list) if list.path.is_ident("streams") => {
                return Err(Error::new(
                    list.span(),
                    "a plugin registers the streams its templates, observers and evidence declare; remove `streams(...)` and declare a stream no template or observer publishes in `dokime::Evidence<Streams>`",
                ));
            }
            Meta::NameValue(named) if named.path.is_ident("stream_entries") => {
                return Err(Error::new(
                    named.span(),
                    "`#[dokime::plugin]` derives its stream grant from its declarations; remove `stream_entries`",
                ));
            }
            Meta::NameValue(named) if named.path.is_ident("requires") => {
                let authored = &named.value;
                let hosted = hosted(&contributed);
                if hosted.is_empty() {
                    forwarded.push(quote!(#argument));
                    continue;
                }
                let name = format_ident!("__DokimeRequires{owner}");
                let variants = hosted
                    .iter()
                    .enumerate()
                    .map(|(index, path)| {
                        let variant = format_ident!("Host{index}");
                        quote!(#variant(#path))
                    })
                    .collect::<Vec<_>>();
                generated.push(quote! {

                    #[#sdk::catalog]
                    pub enum #name {
                        #[nested]
                        Authored(#authored),
                        #(#variants),*
                    }
                });
                forwarded.push(quote!(requires = #name));
            }
            Meta::List(list) if list.path.is_ident("requires") => {
                let written = list.tokens.clone();
                let named = Punctuated::<RequiredEntry, Token![,]>::parse_terminated.parse2(written.clone())?;
                let mut appended = Vec::new();
                for path in hosted(&contributed) {
                    let last = last_ident(&path);
                    if !named.iter().any(|entry| last_ident(&entry.path) == last) {
                        appended.push(path);
                    }
                }
                let trailing = matches!(
                    written.clone().into_iter().last(),
                    Some(proc_macro2::TokenTree::Punct(punct)) if punct.as_char() == ','
                );
                let separator = (!written.is_empty() && !trailing && !appended.is_empty()).then(|| quote!(,));
                forwarded.push(quote!(requires(#written #separator #(#appended),*)));
            }
            _ => forwarded.push(quote!(#argument)),
        }
    }

    Ok(quote! {
        #(#generated)*

        #[#sdk::__harmos::sidecar(harmos_path = #sdk::__harmos, #(#forwarded),*)]
        #block
    })
}

// Harmos qualifies every operation the plugin serves as `<plugin id>.<operation>`,
// so the id is one owner segment; refusing it here names the line it was written on.
fn owner_segment(arguments: &Punctuated<Meta, Token![,]>) -> Result<(), Error> {
    for argument in arguments {
        let Meta::NameValue(named) = argument else { continue };
        if !named.path.is_ident("id") {
            continue;
        }
        let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(value), .. }) = &named.value else { continue };
        let text = value.value();
        let mut bytes = text.bytes();
        let valid = bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
            && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
        if !valid {
            return Err(Error::new_spanned(
                value,
                "a plugin id is one owner segment: a lowercase ASCII letter, then lowercase letters, digits or `_` (no `-` or `.`)",
            ));
        }
    }
    Ok(())
}

fn contributed(arguments: &Punctuated<Meta, Token![,]>) -> Result<Contributed, Error> {
    let mut contributed = Contributed::default();
    for argument in arguments {
        let Meta::List(list) = argument else { continue };
        if !list.path.is_ident("contributions") {
            continue;
        }
        let listed = Punctuated::<Type, Token![,]>::parse_terminated.parse2(list.tokens.clone())?;
        for contribution in listed {
            let Type::Path(path) = &contribution else {
                continue;
            };
            let Some(last) = path.path.segments.last() else {
                continue;
            };
            let generics = match &last.arguments {
                PathArguments::AngleBracketed(arguments) => arguments
                    .args
                    .iter()
                    .filter_map(|argument| match argument {
                        GenericArgument::Type(ty) => Some(ty.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            };
            match last.ident.to_string().as_str() {
                "Suites" => contributed.suites = generics.first().cloned(),
                "Evidence" => contributed.evidence = generics.first().cloned(),
                "Records" => {
                    contributed.records = generics.first().cloned();
                    contributed.listeners =
                        generics.get(1).filter(|ty| !matches!(ty, Type::Tuple(tuple) if tuple.elems.is_empty())).cloned();
                }
                _ => {}
            }
        }
    }
    Ok(contributed)
}

// The plugin publishes what its declarations name: the streams of its suites'
// templates and their observers, of its evidence, and of its own observers.
// Several declarations may name one stream; it is granted once, and two that
// disagree about it stop the build.
fn streams(contributed: &Contributed, owner: &Type) -> TokenStream {
    let sdk = crate::facade::root();
    let entry = quote!(#sdk::catalog::Publication);
    let suites = contributed.suites.iter().map(|suites| {
        quote!(#sdk::catalog::Node::Nested(
            &<#suites as #sdk::catalog::Catalog<#sdk::catalog::SuiteStreams>>::ENTRIES
        ))
    });
    let evidence = contributed.evidence.iter().map(|streams| {
        quote!(#sdk::catalog::Node::Nested(
            &<#streams as #sdk::catalog::Catalog<#sdk::catalog::Streams>>::ENTRIES
        ))
    });

    quote!(stream_entries = {
        const DECLARED: &[#sdk::catalog::Node<#entry>] = &[#(#suites,)* #(#evidence,)*];
        const OBSERVED: &[#sdk::catalog::Node<#sdk::__harmos::sidecar::inventory::observers::Entry>] =
            <#owner as #sdk::__harmos::plugin::Declared>::OBSERVERS;
        const COUNT: usize = #sdk::catalog::publication_count(DECLARED, OBSERVED);
        const GATHERED: [#entry; COUNT] = #sdk::catalog::publications::<COUNT>(DECLARED, OBSERVED);
        const DISTINCT: usize = #sdk::catalog::distinct_publications(&GATHERED);
        const GRANTED: [#entry; DISTINCT] = #sdk::catalog::deduplicated::<DISTINCT>(&GATHERED);
        &GRANTED
    })
}

fn hosted(contributed: &Contributed) -> Vec<Path> {
    let sdk = crate::facade::root();
    let mut hosted: Vec<Path> = Vec::new();
    if contributed.suites.is_some() {
        hosted.push(syn::parse_quote!(#sdk::transport::EvidenceBatch));
        hosted.push(syn::parse_quote!(#sdk::transport::OperatorQuestion));
        hosted.push(syn::parse_quote!(#sdk::transport::LeaseAsk));
        hosted.push(syn::parse_quote!(#sdk::__harmos::builtins::streams::Publish));
        // The host's step built-in every template reaches through
        // `ctx.host()` without listing it.
        hosted.push(syn::parse_quote!(#sdk::operations::files::WatchFile));
        hosted.push(syn::parse_quote!(#sdk::operations::files::AwaitFileWatchArmed));
    }
    if contributed.records.is_some() {
        hosted.push(syn::parse_quote!(#sdk::records::RecordPublication));
    }
    hosted
}

fn last_ident(path: &Path) -> String {
    path.segments.last().map(|segment| segment.ident.to_string()).unwrap_or_default()
}

/* === Tests === */
