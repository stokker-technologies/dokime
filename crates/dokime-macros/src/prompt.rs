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
use quote::{ToTokens, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Expr, Fields, Ident, LitStr, Type};

// Local Imports
use crate::arguments::Arguments;

/* === Constants === */

const ARGUMENTS: &[&str] = &["id", "label", "description", "submit", "cancel", "level"];
const FIELD_ARGUMENTS: &[&str] =
    &["label", "description", "default", "min", "max", "min_length", "max_length", "pattern", "min_items", "max_items"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct FieldDeclaration {
    name: Ident,
    identifier: LitStr,
    ty: Type,
    leaf: Type,
    shape: Shape,
    label: TokenStream,
    described: Option<TokenStream>,
    written: FieldArguments,
}

#[derive(Default)]
struct FieldArguments {
    label: Option<TokenStream>,
    description: Option<Expr>,
    default: Option<Expr>,
    min: Option<Expr>,
    max: Option<Expr>,
    min_length: Option<Expr>,
    max_length: Option<Expr>,
    pattern: Option<Expr>,
    min_items: Option<Expr>,
    max_items: Option<Expr>,
}

/* === Enums === */

#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Required,
    Optional,
    Multiple,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn prompt(attribute: TokenStream, item: TokenStream) -> TokenStream {
    match declare(attribute, &item) {
        Ok(declared) => crate::facade::contracted(declared),
        Err(refused) => {
            let refusal = refused.to_compile_error();

            quote! { #item #refusal }
        }
    }
}

/* === Private Functions === */

fn declare(attribute: TokenStream, item: &TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let input: DeriveInput = syn::parse2(item.clone())?;
    let arguments = Arguments::read(attribute, "a prompt", ARGUMENTS)?;
    let identifier = arguments.required("id", "a prompt declares the id the host matches an asking against")?.clone();
    let label = arguments.required("label", "an operator reads the question by its label")?.clone();
    let description = crate::arguments::description(arguments.value("description"), &input.attrs).map(|text| {
        quote!(
            declared.message =
                ::core::option::Option::Some(::std::string::String::from(#text));
        )
    });
    let level = arguments
        .value("level")
        .map(|written| crate::record::declared_severity(written, "a prompt is asked"))
        .transpose()?
        .map(|level| quote!(declared.level = #level;));
    let Data::Struct(structure) = &input.data else {
        return Err(Error::new(
            input.ident.span(),
            "a prompt is a struct: its fields are the question's fields, and a unit struct is a confirmation",
        ));
    };
    let name = &input.ident;
    let (generics, types, predicates) = input.generics.split_for_impl();
    let declared = stripped(&input);
    let inherent = quote! {
                pub const PROMPT_TYPE: &'static str = #identifier;
    };

    // === A Confirmation === //

    if matches!(structure.fields, Fields::Unit) {
        for relabel in ["submit", "cancel"] {
            if let Some(written) = arguments.value(relabel) {
                return Err(Error::new(written.span(), "a confirmation answers yes or no; `submit` and `cancel` label a form's actions"));
            }
        }

        let instance = instance(&input, &[]);
        let defaults = defaults(&input, &[]);

        return Ok(quote! {
            #instance
            #defaults
            #declared

            #[automatically_derived]
            impl #generics #name #types #predicates {
                #inherent
            }

            #[automatically_derived]
            impl #generics #sdk::prompts::DeclaredPrompt for #name #types #predicates {
                type Answer = bool;

                const PROMPT_TYPE: &'static str = #identifier;

                fn definition() -> #sdk::prompts::PromptDefinition {
                    let mut declared = #sdk::prompts::declared_confirmation(#identifier, #label);
                    #description
                    #level
                    declared
                }

                fn decode(
                    definition: &#sdk::prompts::PromptDefinition,
                    response: #sdk::prompts::PromptResponse,
                ) -> ::core::result::Result<bool, #sdk::Error> {
                    #sdk::prompts::confirmation_answer(definition, &response)
                }
            }
        });
    }

    // === A Form === //

    let fields = declarations(structure)?;
    let submit = arguments.value("submit").map(|label| quote!(declared.actions[0].label = ::std::string::String::from(#label);));
    let cancel = arguments.value("cancel").map(|label| quote!(declared.actions[1].label = ::std::string::String::from(#label);));
    let declared_fields = fields.iter().map(declaration);
    let reads = fields.iter().map(read);
    let instance = instance(&input, &fields);
    let defaults = defaults(&input, &fields);

    Ok(quote! {
        #instance
        #defaults
        #declared

        #[automatically_derived]
        impl #generics #name #types #predicates {
            #inherent
        }

        #[automatically_derived]
        impl #generics #sdk::prompts::DeclaredPrompt for #name #types #predicates {
            type Answer = Self;

            const PROMPT_TYPE: &'static str = #identifier;

            fn definition() -> #sdk::prompts::PromptDefinition {
                let mut declared = #sdk::prompts::declared_form(#identifier, #label);
                #description
                #level
                #submit
                #cancel
                declared.fields = ::std::vec![#(#declared_fields),*];
                declared
            }

            fn decode(
                definition: &#sdk::prompts::PromptDefinition,
                response: #sdk::prompts::PromptResponse,
            ) -> ::core::result::Result<Self, #sdk::Error> {
                #sdk::prompts::form_answer(definition, &response)?;
                ::core::result::Result::Ok(Self { #(#reads),* })
            }
        }
    })
}

fn instance(input: &DeriveInput, fields: &[FieldDeclaration]) -> TokenStream {
    let sdk = crate::facade::root();
    let name = &input.ident;
    let (generics, types, predicates) = input.generics.split_for_impl();
    let values = fields.iter().enumerate().map(|(index, field)| {
        let name = &field.name;
        quote! {
            definition.fields[#index].default = ::core::option::Option::Some(
                #sdk::values::FieldValue::into_value(self.#name.clone())
            );
        }
    });

    quote! {
        #[automatically_derived]
        impl #generics #sdk::prompts::Prompt for #name #types #predicates {
            type Response = <Self as #sdk::prompts::DeclaredPrompt>::Answer;

            fn definition(&self) -> #sdk::prompts::PromptDefinition {
                let mut definition = <Self as #sdk::prompts::DeclaredPrompt>::definition();
                #(#values)*
                definition
            }

            fn decode(&self, response: #sdk::prompts::PromptResponse) -> #sdk::Result<Self::Response> {
                <Self as #sdk::prompts::DeclaredPrompt>::decode(
                    &<Self as #sdk::prompts::Prompt>::definition(self), response,
                )
            }
        }
    }
}

fn defaults(input: &DeriveInput, fields: &[FieldDeclaration]) -> TokenStream {
    if fields.iter().any(|field| field.written.default.is_none()) {
        return TokenStream::new();
    }
    let name = &input.ident;
    let (generics, types, predicates) = input.generics.split_for_impl();
    let values = fields.iter().map(|field| {
        let name = &field.name;
        let value = field.written.default.as_ref().expect("all defaults were checked");
        quote!(#name: ::core::convert::Into::into(#value))
    });
    let value = if matches!(&input.data, Data::Struct(data) if matches!(data.fields, Fields::Unit)) {
        quote!(Self)
    } else {
        quote!(Self { #(#values),* })
    };

    quote! {
        #[automatically_derived]
        impl #generics ::core::default::Default for #name #types #predicates {
            fn default() -> Self { #value }
        }
    }
}

fn declarations(structure: &syn::DataStruct) -> Result<Vec<FieldDeclaration>, Error> {
    let named =
        crate::arguments::named_fields(&structure.fields, "a form names its fields: a field's name is its id in the operator's answer")?;
    named
        .named
        .iter()
        .map(|field| {
            let name = field.ident.clone().expect("a named field carries an identifier");
            let written = arguments(field)?;
            let (shape, leaf) = shape(&field.ty);
            if shape != Shape::Multiple && (written.min_items.is_some() || written.max_items.is_some()) {
                return Err(Error::new(name.span(), "min_items and max_items bound a multiple choice: declare the field as `Vec<T>`"));
            }
            let identifier = LitStr::new(&name.unraw().to_string(), name.span());
            let label = written.label.clone().unwrap_or_else(|| {
                let titled = crate::arguments::titled(&name.unraw().to_string());

                quote!(#titled)
            });

            Ok(FieldDeclaration {
                identifier,
                name,
                ty: field.ty.clone(),
                leaf,
                shape,
                label,
                described: crate::arguments::description(written.description.as_ref(), &field.attrs),
                written,
            })
        })
        .collect()
}

fn arguments(field: &syn::Field) -> Result<FieldArguments, Error> {
    let mut written = FieldArguments::default();

    for (name, value) in crate::arguments::field_arguments(field, "prompt", "a prompt field argument", FIELD_ARGUMENTS, &[])?.values() {
        match name.to_string().as_str() {
            "label" => written.label = Some(value.to_token_stream()),
            "description" => written.description = Some(value),
            "default" => written.default = Some(value),
            "min" => written.min = Some(value),
            "max" => written.max = Some(value),
            "min_length" => written.min_length = Some(value),
            "max_length" => written.max_length = Some(value),
            "pattern" => written.pattern = Some(value),
            "min_items" => written.min_items = Some(value),
            "max_items" => written.max_items = Some(value),
            other => unreachable!("`{other}` is refused where the arguments are read"),
        }
    }

    Ok(written)
}

fn shape(declared: &Type) -> (Shape, Type) {
    let (shape, leaf) = crate::arguments::shape(declared);
    let shape = match shape {
        crate::arguments::Shape::Required => Shape::Required,
        crate::arguments::Shape::Optional => Shape::Optional,
        crate::arguments::Shape::List | crate::arguments::Shape::OptionalList => Shape::Multiple,
    };

    (shape, leaf)
}

fn declaration(field: &FieldDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let identifier = &field.identifier;
    let label = &field.label;
    let leaf = &field.leaf;
    let required = field.shape != Shape::Optional;
    let multiple = field.shape == Shape::Multiple;
    let declare = quote_spanned!(leaf.span()=>
        #sdk::prompts::declared_field::<#leaf>(#identifier, #label, #required, #multiple)
    );
    let described = field.described.as_ref().map(|description| {
        quote!(field.metadata = #sdk::prompts::declared_metadata(
            #identifier,
            #label,
            ::core::option::Option::Some(::std::string::String::from(#description)),
        );)
    });
    let ty = &field.ty;
    let default = field.written.default.as_ref().map(|default| {
        quote_spanned!(default.span()=>
            field.default = ::core::option::Option::Some(
                #sdk::prompts::declared_default::<#ty>(#default)
            ).filter(|value| !matches!(value, #sdk::Value::Null));
        )
    });
    // A bound is a value of the field's own type, so an integer field's bound
    // travels as an integer and a float field's as a float.
    let min = field.written.min.as_ref().map(|min| {
        quote_spanned!(min.span()=>
            field.validation.minimum = ::core::option::Option::Some(
                #sdk::prompts::declared_default::<#leaf>(#min)
            );
        )
    });
    let max = field.written.max.as_ref().map(|max| {
        quote_spanned!(max.span()=>
            field.validation.maximum = ::core::option::Option::Some(
                #sdk::prompts::declared_default::<#leaf>(#max)
            );
        )
    });
    let count = |name: &str, bound: &Option<Expr>| {
        let name = Ident::new(name, proc_macro2::Span::call_site());
        bound.as_ref().map(|value| quote!(field.validation.#name = ::core::option::Option::Some((#value) as usize);))
    };
    let min_length = count("min_length", &field.written.min_length);
    let max_length = count("max_length", &field.written.max_length);
    let min_items = count("min_items", &field.written.min_items);
    let max_items = count("max_items", &field.written.max_items);
    let pattern = field.written.pattern.as_ref().map(|pattern| {
        quote!(field.validation.pattern = ::core::option::Option::Some(
            ::std::string::String::from(#pattern)
        );)
    });

    quote! {{
        let mut field = #declare;
        #described
        #default
        #min
        #max
        #min_length
        #max_length
        #min_items
        #max_items
        #pattern
        field
    }}
}

fn read(field: &FieldDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let name = &field.name;
    let identifier = &field.identifier;
    let ty = &field.ty;
    let read = quote_spanned!(ty.span()=>
        #sdk::prompts::answered::<#ty>(&response.values, #identifier)?
    );

    quote!(#name: #read)
}

fn stripped(input: &DeriveInput) -> TokenStream {
    crate::arguments::stripped(input, "prompt").to_token_stream()
}

/* === Tests === */
