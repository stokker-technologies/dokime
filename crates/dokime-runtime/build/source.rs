/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Normalized Storage Codec Source

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use sha2::{Digest, Sha256};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Preserve Literal Contents And Token Boundaries, Discard All Source Trivia.
pub fn digest(source: &str) -> String {
    let tokens: TokenStream = source.parse().expect("storage input must be valid Rust tokens");
    let mut hash = Sha256::new();
    hash_tokens(&mut hash, tokens);
    hex(&hash.finalize())
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Private Functions === */

fn hash_tokens(hash: &mut Sha256, tokens: TokenStream) {
    let mut tokens = tokens.into_iter().peekable();
    while let Some(token) = tokens.next() {
        // Rust Tokenization Converts Doc Comments To Attributes.
        if matches!(&token, TokenTree::Punct(value) if value.as_char() == '#') {
            let mut lookahead = tokens.clone();
            if matches!(lookahead.peek(), Some(TokenTree::Punct(value)) if value.as_char() == '!') {
                lookahead.next();
            }
            if let Some(TokenTree::Group(group)) = lookahead.next()
                && group.delimiter() == Delimiter::Bracket
                && matches!(group.stream().into_iter().next(), Some(TokenTree::Ident(name)) if name == "doc")
            {
                tokens = lookahead;
                continue;
            }
        }
        match token {
            TokenTree::Group(group) => {
                hash.update([match group.delimiter() {
                    Delimiter::Parenthesis => b'(',
                    Delimiter::Brace => b'{',
                    Delimiter::Bracket => b'[',
                    Delimiter::None => b' ',
                }]);
                hash_tokens(hash, group.stream());
                hash.update(b")");
            }
            token => {
                let text = token.to_string();
                hash.update((text.len() as u64).to_le_bytes());
                hash.update(text.as_bytes());
            }
        }
    }
}

/* === Tests === */
