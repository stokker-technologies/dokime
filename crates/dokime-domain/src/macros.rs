/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

macro_rules! product_error {
    ($error:ty, $formatter:ident { $($pattern:pat => $display:expr),+ $(,)? }) => {
        impl std::fmt::Display for $error {
            fn fmt(&self, $formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $($pattern => $display),+
                }
            }
        }

        impl std::error::Error for $error {}
    };
}
