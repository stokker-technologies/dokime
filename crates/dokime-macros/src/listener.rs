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

const ARGUMENTS: &[&str] = &["id", "record", "description", "resources"];

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
    let block: ItemImpl = syn::parse2(item.clone())?;
    if block.trait_.is_some() {
        return Err(Error::new_spanned(&block.self_ty, "`#[dokime::listener]` declares the listener's own inherent impl"));
    }
    let identifier = arguments.required("id", "the host calls a listener by its operation id")?;
    let identifier = &crate::arguments::wire_identity(Some(identifier.clone()), crate::arguments::type_name(&block.self_ty)?)?;
    let record = arguments.required("record", "a listener hears one declared record type")?;
    let description = arguments.value("description").map(|description| quote!(const DESCRIPTION: &'static str = #description;));
    let resources = match arguments.value("resources") {
        Some(value) => crate::arguments::elements(value)?,
        None => Vec::new(),
    };
    let hears =
        block.items.iter().any(|item| matches!(item, ImplItem::Fn(method) if method.sig.ident == "hear" && method.sig.asyncness.is_some()));
    if !hears {
        return Err(Error::new_spanned(
            &block.self_ty,
            "a listener writes `async fn hear(ctx: &ListenerContext<'_, Self>, record: Record) -> Result<(), Error>`",
        ));
    }
    let owner = block.self_ty.clone();
    let harmos = quote!(#sdk::__harmos);

    // The listener type is its callback operation's identity; the shared
    // `Delivery` is the request and `Hearing` the carrier that decodes the
    // record and hands it to `hear` with the declared resources and the
    // attempt's own host.
    Ok(quote! {
        #block

        #[automatically_derived]
        impl #harmos::operations::OperationDeclaration for #owner {
            type Request = #sdk::plugin::Delivery;
            type Response = ();
            type Input = #harmos::operations::NoStream;
            type Output = #harmos::operations::NoStream;
            const ID: &'static str = #identifier;
            const POLICY: #harmos::operations::OperationPolicy = #sdk::plugin::LISTENER_POLICY;
            const VISIBILITY: #harmos::operations::Visibility = #harmos::operations::Visibility::Host;
            #description
        }

        #[automatically_derived]
        impl #harmos::Executable for #owner {
            type Carrier = #sdk::plugin::Hearing<#owner>;
            const RESOURCES: &'static [#harmos::Resource] =
                &[#(#harmos::Resource::of::<#resources>(),)* #harmos::Resource::of::<#sdk::Supervisor>()];
        }

        #(
            #[automatically_derived]
            impl #harmos::operations::Requires<#resources> for #owner {}
        )*

        #[automatically_derived]
        impl #harmos::operations::Requires<#sdk::Supervisor> for #owner {}

        #[automatically_derived]
        impl #sdk::plugin::Listener for #owner {
            type Record = #record;

            fn hear<'__dokime>(
                ctx: &'__dokime #sdk::plugin::ListenerContext<'__dokime, Self>,
                record: #record,
            ) -> #sdk::plugin::HearFuture<'__dokime> {
                ::std::boxed::Box::pin(async move { #sdk::plugin::refusal(<#owner>::hear(ctx, record).await) })
            }
        }
    })
}

/* === Tests === */
