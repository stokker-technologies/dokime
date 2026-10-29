/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Host-Side Behaviour Of Each Declared Operation. The Operation Owns Its
// `impl Operation`; The Host Inserts One `Host<O>` Resource Per Operation It
// Serves, Implemented Beside The Engines It Drives.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// Third Party Imports
use harmos::{Context, Operation, OperationDeclaration, Response};
use tonic::Status;

// Local Imports

/* === Constants === */

/* === Macros === */

// Implements `Operation` For Each Listed Declaration By Serving It Through Its `Host<O>` Resource.
#[macro_export]
macro_rules! hosted {
    ($($operation:ty),* $(,)?) => {
        $(
            impl ::harmos::Operation for $operation {
                type Error = ::tonic::Status;

                async fn run(
                    self, ctx: ::harmos::Context<'_, Self>,
                ) -> ::core::result::Result<::harmos::Response<<Self as ::harmos::OperationDeclaration>::Response>, ::tonic::Status> {
                    let host = ctx.resource::<$crate::hosting::Host<Self>>();
                    host.serve(self, ctx).await
                }
            }
        )*
    };
}

/* === Type Aliases === */

pub type Served<'a, T> = Pin<Box<dyn Future<Output = Result<Response<T>, Status>> + Send + 'a>>;

/* === Structs === */

pub struct Host<O: Operation>(Arc<dyn Serve<O>>);

/* === Enums === */

/* === Traits === */

pub trait Serve<O: Operation>: Send + Sync + 'static {
    fn serve<'a>(&'a self, request: O, ctx: Context<'a, O>) -> Served<'a, <O as OperationDeclaration>::Response>;
}

/* === Implementations === */

impl<O: Operation> Host<O> {
    pub fn new(serve: Arc<dyn Serve<O>>) -> Arc<Self> {
        Arc::new(Self(serve))
    }

    pub fn serve<'a>(&'a self, request: O, ctx: Context<'a, O>) -> Served<'a, <O as OperationDeclaration>::Response> {
        self.0.serve(request, ctx)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
