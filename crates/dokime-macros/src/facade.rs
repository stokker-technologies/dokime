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

// Local Imports

/* === Constants === */

pub(crate) const MACRO_CONTRACT: u32 = 26;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct SdkRoot;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ToTokens for SdkRoot {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = name();
        let name = Ident::new(&name, Span::call_site());
        tokens.extend(quote!(::#name));
    }
}

/* === Public Functions === */

pub(crate) fn root() -> SdkRoot {
    SdkRoot
}

pub(crate) fn name() -> String {
    match proc_macro_crate::crate_name("dokime-runtime") {
        Ok(proc_macro_crate::FoundCrate::Name(name)) => name,
        _ => "dokime_runtime".to_owned(),
    }
}

pub(crate) fn harmos_attribute(name: &str, attribute: TokenStream, item: TokenStream) -> TokenStream {
    let sdk = root();
    let name = Ident::new(name, Span::call_site());
    quote! {
        #[#sdk::__harmos::#name(harmos_path = #sdk::__harmos, #attribute)]
        #item
    }
}

pub(crate) fn contracted(declared: TokenStream) -> TokenStream {
    let sdk = root();

    quote! {
        #declared

        const _: () = assert!(
            #sdk::MACRO_CONTRACT == #MACRO_CONTRACT,
            "dokime-runtime and dokime-macros resolve to different revisions; depend on `dokime-runtime` only, pinned to one rev, and run `cargo update -p dokime-runtime`"
        );
    }
}

/* === Private Functions === */

/* === Tests === */
