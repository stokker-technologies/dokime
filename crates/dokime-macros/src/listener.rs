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
use syn::{Error, ImplItem, ItemImpl};

// Local Imports
use crate::arguments::Arguments;

/* === Constants === */

const ARGUMENTS: &[&str] = &["id", "record", "description"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn listener(attribute: TokenStream, item: TokenStream) -> TokenStream {
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
    let arguments = Arguments::read(attribute, "a listener", ARGUMENTS)?;
    let mut block: ItemImpl = syn::parse2(item.clone())?;
    if block.trait_.is_some() {
        return Err(Error::new_spanned(&block.self_ty, "`#[dokime_runtime::listener]` declares the listener's own inherent impl"));
    }
    let identifier = arguments.required("id", "the host calls a listener by its operation id")?;
    let identifier = &crate::arguments::wire_identity(Some(identifier.clone()), crate::arguments::type_name(&block.self_ty)?)?;
    let record = arguments.required("record", "a listener hears one declared record type")?;
    let description = arguments.value("description").map(|description| quote!(const DESCRIPTION: &'static str = #description;));
    let hears =
        block.items.iter().any(|item| matches!(item, ImplItem::Fn(method) if method.sig.ident == "hear" && method.sig.asyncness.is_some()));
    if !hears {
        return Err(Error::new_spanned(
            &block.self_ty,
            "a listener writes `async fn hear(&mut self, ctx: &Supervisor, record: Record) -> Result<(), Error>`",
        ));
    }
    let owner = block.self_ty.clone();
    block.items.push(syn::parse_quote! {
        async fn run(
            &mut self,
            context: &#sdk::Supervisor,
            delivery: &#sdk::plugin::Heard<#owner>,
        ) -> ::core::result::Result<(), #sdk::Refusal> {
            let record = #sdk::plugin::refusal(delivery.0.record::<#record>())?;
            #sdk::plugin::refusal(<#owner>::hear(self, context, record).await)
        }
    });

    Ok(quote! {
        #[#sdk::__harmos::operation(harmos_path = #sdk::__harmos)]
        #block

        #[automatically_derived]
        impl #sdk::plugin::Listener for #owner {
            const ID: &'static str = #identifier;
            #description
            type Record = #record;
        }
    })
}

/* === Tests === */
