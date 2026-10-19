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
pub(crate) fn argument(arguments: &crate::arguments::Arguments, name: &str) -> Result<TokenStream, Error> {
    match arguments.catalog(name)? {
        Some(value) => {
            let path: syn::Path = syn::parse2(value.to_token_stream())?;
            Ok(quote!(#path))
        }
        None => Ok(quote!(())),
    }
}

/* === Private Functions === */

/* === Tests === */
