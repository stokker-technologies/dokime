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
use syn::spanned::Spanned;
use syn::{Error, ImplItem, ImplItemFn, ItemImpl};
// Local Imports
use crate::arguments::{self, Arguments};

/* === Constants === */
const ARGUMENTS: &[&str] = &[
    "id",
    "label",
    "description",
    "product",
    "tags",
    "templates",
    "records",
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn suite(attribute: TokenStream, item: TokenStream) -> TokenStream {
    match expand(attribute, &item) {
        Ok(declared) => crate::facade::contracted(declared),
        Err(refused) => {
            let refusal = refused.to_compile_error();

            quote! { #item #refusal }
        }
    }
}

/* === Private Functions === */
fn expand(attribute: TokenStream, item: &TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let arguments = Arguments::read(attribute, "a suite", ARGUMENTS)?;
    let block: ItemImpl = syn::parse2(item.clone())?;
    if let Some((_, path, _)) = &block.trait_ {
        return Err(Error::new(
            path.span(),
            "`#[dokime::suite]` declares the suite's own inherent impl, not a trait implementation",
        ));
    }
    let suite = block.self_ty.clone();

    let identifier = match arguments.value("id") {
        Some(identifier) => identifier.clone(),
        None => arguments.identifier(arguments::type_name(&suite)?)?,
    };
    let label = arguments.required("label", "an operator reads the suite by its label")?;
    let metadata = arguments::metadata(
        &identifier,
        label,
        arguments.value("description"),
        &block.attrs,
    );
    let tags = arguments
        .value("tags")
        .map(arguments::elements)
        .transpose()?
        .unwrap_or_default();
    let tags = quote!(::std::vec![#(::std::string::ToString::to_string(#tags)),*]);

    let templates = crate::catalog::argument(&arguments, "templates")?;
    let declarations = quote!(<#templates as #sdk::catalog::Catalog<#sdk::catalog::Templates>>::ENTRIES
        .iter().map(|entry| (entry.declaration)()).collect());
    let product = arguments.value("product").map(|product| {
        quote!(definition.product = ::core::option::Option::Some(
            ::std::string::ToString::to_string(#product)
        );)
    });
    let workflow = quote! {
        definition.workflow = ::core::option::Option::Some(
            ::std::string::ToString::to_string(#identifier)
        );
    };
    // A record the suite names is one every template it owns publishes. One a
    // single template publishes is named on that template instead, beside the
    // step that files it, and reaches the registration through the same closed
    // template set the stream contracts do.
    let records = crate::catalog::argument(&arguments, "records")?;
    let records = quote!(definition.record_contracts = <#records as #sdk::catalog::Catalog<#sdk::catalog::Records>>::ENTRIES
        .iter().map(|contract| contract()).collect(););
    let template_records = quote! {
        for entry in <#templates as #sdk::catalog::Catalog<#sdk::catalog::Templates>>::ENTRIES {
            definition.record_contracts.extend((entry.records)());
        }
    };
    let telemetry = declared(&block, "telemetry")
        .map(|_| quote!(definition.telemetry_contracts = <Self>::telemetry();));
    let stream_contracts = quote! {
        for entry in <#templates as #sdk::catalog::Catalog<#sdk::catalog::Templates>>::ENTRIES {
            definition.telemetry_contracts.extend((entry.streams)());
        }
    };
    let parameter_groups = match declared(&block, "parameter_groups") {
        Some(_) => quote!(<Self>::parameter_groups()),
        None => quote!(::std::vec::Vec::new()),
    };

    Ok(quote! {
        #block

        #[automatically_derived]
        impl #suite {
                        pub fn suite() -> #sdk::definition::TestSuite {
                #sdk::definition::TestSuite {
                    metadata: match #sdk::definition::AuthoredMetadata::build(#metadata) {
                        ::core::result::Result::Ok(metadata) => metadata,
                        ::core::result::Result::Err(refusal) => ::core::panic!(
                            "suite `{}` is not declarable: {}", #identifier, refusal
                        ),
                    },
                    tags: #tags,
                    parameter_groups: #parameter_groups,
                    test_case_templates: #declarations,
                }
            }

                        pub fn definition() -> #sdk::domains::TestSuiteDefinition {
                let mut definition = match #sdk::domains::TestSuiteDefinition::new(
                    #identifier,
                    <Self>::suite(),
                ) {
                    ::core::result::Result::Ok(definition) => definition,
                    ::core::result::Result::Err(refusal) => ::core::panic!(
                        "suite `{}` is not declarable: {}", #identifier, refusal
                    ),
                };
                #product
                definition.tags = #tags;
                #workflow
                #records
                #template_records
                #telemetry
                #stream_contracts
                definition
            }
        }

        #[automatically_derived]
        impl #sdk::definition::DeclaredSuite for #suite {
            const OPERATIONS: &'static [#sdk::catalog::Node<#sdk::__harmos::operations::WireDeclaration>] =
                &<#templates as #sdk::catalog::Catalog<#sdk::catalog::TemplateOperations>>::ENTRIES;

            fn definition() -> #sdk::domains::TestSuiteDefinition {
                <Self>::definition()
            }

            fn bind<__DokimePlugin: #sdk::__harmos::artifact::plugin::Declared>(
                table: &mut #sdk::__harmos::sidecar::operations::WireTable<#sdk::Supervisor>,
                _: &str,
                _: &[#sdk::records::RecordContract],
            ) -> ::core::result::Result<(), #sdk::__harmos::sidecar::operations::ServingError> {
                let contracts = <Self>::definition().record_contracts;
                for bind in <#templates as #sdk::catalog::Catalog<#sdk::catalog::TemplateBindings<__DokimePlugin>>>::ENTRIES {
                    bind(table, #identifier, &contracts)?;
                }
                ::core::result::Result::Ok(())
            }

            fn runners() -> ::std::vec::Vec<::std::sync::Arc<dyn #sdk::execution::Runnable>> {
                let contracts = <Self>::definition().record_contracts;
                <#templates as #sdk::catalog::Catalog<#sdk::catalog::TemplateRunners>>::ENTRIES
                    .iter()
                    .map(|runner| runner(#identifier, &contracts))
                    .collect()
            }
        }

    })
}

fn declared<'a>(block: &'a ItemImpl, name: &str) -> Option<&'a ImplItemFn> {
    block.items.iter().find_map(|item| match item {
        ImplItem::Fn(function) if function.sig.ident == name => Some(function),
        _ => None,
    })
}

/* === Tests === */
