/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Instrument Native Test Lifetimes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use quote::quote;
use syn::{ItemFn, parse_quote};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn expand(attribute: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let logging = match attribute.to_string().as_str() {
        "" => false,
        "logging" => true,
        _ => return syn::Error::new_spanned(attribute, "expected `logging` or no argument").to_compile_error(),
    };
    let mut function: ItemFn = match syn::parse2(item) {
        Ok(function) => function,
        Err(error) => return error.to_compile_error(),
    };
    let name = function.sig.ident.to_string();
    function.block.stmts.insert(
        0,
        parse_quote! {
            let _dokime_test_trace = dokime_tracing::test(concat!(module_path!(), "::", #name), #logging);
        },
    );
    quote!(#function)
}

/* === Private Functions === */

/* === Tests === */
