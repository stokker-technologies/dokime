/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Expr, Fields, Ident, LitStr, Meta, Token, Type, Variant};

use crate::arguments::Shape;

const FIELD_ARGUMENTS: &[&str] = &["label", "description", "unit", "default", "expansion", "min", "max", "min_items", "max_items"];
const VARIANT_ARGUMENTS: &[&str] = &["id", "label"];

struct ParameterDeclaration {
    name: Ident,
    identifier: LitStr,
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
    unit: Option<Expr>,
    default: Option<Expr>,
    expansion: Option<TokenStream>,
    expanded: bool,
    min: Option<Expr>,
    max: Option<Expr>,
    min_items: Option<Expr>,
    max_items: Option<Expr>,
}

struct ChoiceVariant {
    name: Ident,
    identifier: LitStr,
    label: TokenStream,
}

pub(crate) fn parameters(attribute: TokenStream, item: TokenStream) -> TokenStream {
    match declare(attribute, &item) {
        Ok(declared) => crate::facade::contracted(declared),
        Err(refused) => {
            let refusal = refused.to_compile_error();

            quote! { #item #refusal }
        }
    }
}

pub(crate) fn choice(attribute: TokenStream, item: TokenStream) -> TokenStream {
    match declare_choice(attribute, &item) {
        Ok(declared) => crate::facade::contracted(declared),
        Err(refused) => {
            let refusal = refused.to_compile_error();

            quote! { #item #refusal }
        }
    }
}

pub(crate) fn underscored(name: &str) -> String {
    let mut written = String::with_capacity(name.len() + 4);
    for (index, character) in name.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 {
                written.push('_');
            }
            written.extend(character.to_lowercase());
        } else {
            written.push(character);
        }
    }

    written
}

fn declare(attribute: TokenStream, item: &TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    crate::arguments::Arguments::read(attribute, "a parameters struct", &[])?;
    let input: DeriveInput = syn::parse2(item.clone())?;
    let fields = declarations(&input)?;
    let declared = stripped(&input);
    let name = &input.ident;
    let (generics, types, predicates) = input.generics.split_for_impl();
    let declarations = fields.iter().map(declaration);
    let reads = fields.iter().map(read);
    let handles = fields.iter().map(handle);

    Ok(quote! {
        #declared

        #[automatically_derived]
        impl #generics #name #types #predicates {
            #(#handles)*
        }

        #[automatically_derived]
        impl #generics #sdk::parameters::FromParameters for #name #types #predicates {
            fn declaration() -> ::std::vec::Vec<#sdk::definition::Parameter> {
                ::std::vec![#(#declarations),*]
            }

            fn from_parameters(
                values: &::std::collections::BTreeMap<::std::string::String, #sdk::Value>,
            ) -> #sdk::Result<Self> {
                ::core::result::Result::Ok(Self { #(#reads),* })
            }
        }
    })
}

fn declarations(input: &DeriveInput) -> Result<Vec<ParameterDeclaration>, Error> {
    let named = crate::arguments::named_struct_fields(
        input,
        "a parameters declaration is a struct: its fields are the template's inputs",
        "a parameters struct names its fields: a field's name is its parameter's id",
    )?;
    named
        .named
        .iter()
        .map(|field| {
            let name = field.ident.clone().expect("a named field carries an identifier");
            let written = arguments(field)?;
            let (shape, leaf) = crate::arguments::shape(&field.ty);
            if matches!(shape, Shape::Required | Shape::Optional) && (written.min_items.is_some() || written.max_items.is_some()) {
                return Err(Error::new(name.span(), "min_items and max_items bound a list: declare the field as `Vec<T>`"));
            }
            let identifier = LitStr::new(&name.unraw().to_string(), name.span());
            let label = written.label.clone().unwrap_or_else(|| {
                let titled = crate::arguments::titled(&name.unraw().to_string());

                quote!(#titled)
            });

            Ok(ParameterDeclaration {
                identifier,
                name,
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

    for (name, value) in crate::arguments::field_arguments(field, "parameter", "a parameter argument", FIELD_ARGUMENTS, &[])?.values() {
        match name.to_string().as_str() {
            "label" => written.label = Some(value.to_token_stream()),
            "description" => written.description = Some(value),
            "unit" => written.unit = Some(value),
            "default" => written.default = Some(value),
            "expansion" => {
                let sdk = crate::facade::root();
                written.expansion = Some(match &value {
                    Expr::Lit(lit) if matches!(&lit.lit, syn::Lit::Str(text) if text.value() == "single") => {
                        quote!(#sdk::definition::ParameterExpansion::Single)
                    }
                    Expr::Lit(lit) if matches!(&lit.lit, syn::Lit::Str(text) if text.value() == "cartesian") => {
                        written.expanded = true;
                        quote!(#sdk::definition::ParameterExpansion::Cartesian)
                    }
                    Expr::Call(call) if matches!(&*call.func, Expr::Path(path) if path.path.is_ident("zip")) && call.args.len() == 1 => {
                        let group = &call.args[0];
                        let Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(group), .. }) = group else {
                            return Err(Error::new_spanned(group, "zip requires a literal group name"));
                        };
                        if group.value().trim().is_empty() {
                            return Err(Error::new_spanned(group, "zip group cannot be empty"));
                        }
                        written.expanded = true;
                        quote!(#sdk::definition::ParameterExpansion::Zip {
                            group: #sdk::Identifier::new(#group).expect("valid zip group identifier")
                        })
                    }
                    _ => {
                        return Err(Error::new_spanned(value, "expansion must be \"single\", \"cartesian\", or zip(\"group\")"));
                    }
                });
            }
            "min" => written.min = Some(value),
            "max" => written.max = Some(value),
            "min_items" => written.min_items = Some(value),
            "max_items" => written.max_items = Some(value),
            other => unreachable!("`{other}` is refused where the arguments are read"),
        }
    }

    Ok(written)
}

fn declaration(field: &ParameterDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let identifier = &field.identifier;
    let label = &field.label;
    let leaf = &field.leaf;
    let required = matches!(field.shape, Shape::Required | Shape::List);
    let declare = match field.shape {
        Shape::Required | Shape::Optional => quote_spanned!(leaf.span()=>
            #sdk::parameters::declare::<#leaf>(#identifier, #label, #required)
        ),
        // A list is one value of one case, declared over its element's type
        // with the length bounds the field stated.
        Shape::List | Shape::OptionalList => {
            let bounds = bounds(&field.written);

            quote_spanned!(leaf.span()=>
                #sdk::parameters::declare_list::<#leaf>(#identifier, #label, #required, #bounds)
            )
        }
    };
    let min = field.written.min.as_ref().map(|min| quote!(let declared = declared.min((#min) as f64);));
    let max = field.written.max.as_ref().map(|max| quote!(let declared = declared.max((#max) as f64);));
    let described = field.described.as_ref().map(|description| quote!(let declared = declared.description(#description);));
    let unit = field.written.unit.as_ref().map(|unit| quote!(let declared = declared.unit(#sdk::definition::authored_unit(#unit));));
    // A bare `None` names no type, and `Into<ParameterDefault>` is generic
    // over the option's payload, so the compiler cannot infer it from the
    // literal alone; the field's own leaf type is what it means.
    let expansion = field.written.expansion.as_ref().map(|strategy| quote!(let declared = declared.expansion(#strategy);));
    let default = field.written.default.as_ref().map(|default| {
        let candidates = matches!(default, Expr::Array(_) | Expr::Range(_))
            || matches!(default, Expr::Macro(expression) if expression.mac.path.is_ident("vec"));
        if field.written.expanded && candidates {
            return quote!(let declared = declared.default(#sdk::definition::ParameterDefault::space(#default)););
        }
        let default = match default {
            Expr::Path(path) if path.path.is_ident("None") => {
                quote_spanned!(default.span()=> ::core::option::Option::<#leaf>::None)
            }
            other => other.to_token_stream(),
        };
        quote!(let declared = declared.default(#default);)
    });

    quote! {{
        let declared = #declare;
        #min
        #max
        #described
        #unit
        #default
        #expansion
        declared
    }}
}

fn read(field: &ParameterDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let name = &field.name;
    let identifier = &field.identifier;
    let leaf = &field.leaf;
    let bounds = bounds(&field.written);
    let reader = match field.shape {
        Shape::Required => quote!(scalar),
        Shape::Optional => quote!(optional),
        Shape::List => quote!(list),
        Shape::OptionalList => quote!(optional_list),
    };
    let read = quote_spanned!(leaf.span()=>
        #sdk::parameters::#reader::<#leaf>(values, #identifier, #bounds)?
    );

    quote!(#name: #read)
}

fn bounds(written: &FieldArguments) -> TokenStream {
    let sdk = crate::facade::root();
    let number = |bound: &Option<Expr>| match bound {
        Some(value) => quote!(::core::option::Option::Some((#value) as f64)),
        None => quote!(::core::option::Option::None),
    };
    let count = |bound: &Option<Expr>| match bound {
        Some(value) => quote!(::core::option::Option::Some((#value) as usize)),
        None => quote!(::core::option::Option::None),
    };
    let min = number(&written.min);
    let max = number(&written.max);
    let min_items = count(&written.min_items);
    let max_items = count(&written.max_items);

    quote!(#sdk::parameters::Bounds {
        min: #min,
        max: #max,
        min_items: #min_items,
        max_items: #max_items,
    })
}

fn handle(field: &ParameterDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let name = &field.name;
    let identifier = &field.identifier;

    quote! {

        pub fn #name() -> #sdk::parameters::ParameterHandle {
            #sdk::parameters::ParameterHandle::new(#identifier)
        }
    }
}

fn stripped(input: &DeriveInput) -> TokenStream {
    crate::arguments::stripped(input, "parameter").to_token_stream()
}

fn declare_choice(attribute: TokenStream, item: &TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    crate::arguments::Arguments::read(attribute, "a choice", &[])?;
    let input: DeriveInput = syn::parse2(item.clone())?;
    let Data::Enum(enumeration) = &input.data else {
        return Err(Error::new(input.ident.span(), "a choice is an enum: its variants are the values it admits"));
    };
    let variants = enumeration.variants.iter().map(variant).collect::<Result<Vec<_>, _>>()?;
    if variants.is_empty() {
        return Err(Error::new(input.ident.span(), "a choice admits at least one value: declare a variant"));
    }
    // Choices Used In Records And Rows Share The Same Message Codec.
    let stripped = stripped_choice(&input);
    let declared = crate::message::message(crate::message::Kind::Plain, quote!(), stripped);
    let name = &input.ident;
    let (generics, types, predicates) = input.generics.split_for_impl();
    let choices = variants.iter().map(|variant| {
        let identifier = &variant.identifier;
        let label = &variant.label;

        quote!(#sdk::parameters::ParameterChoice::new(#identifier, #label))
    });
    let refinements = variants.iter().map(|variant| {
        let identifier = &variant.identifier;
        let name = &variant.name;

        quote!(#identifier => ::core::result::Result::Ok(Self::#name),)
    });
    let encoded = variants.iter().map(|variant| {
        let variant_name = &variant.name;
        let identifier = &variant.identifier;
        quote!(Self::#variant_name => #sdk::Value::Text(::std::string::String::from(#identifier)),)
    });
    let admitted = variants.iter().map(|variant| variant.identifier.value()).collect::<Vec<_>>().join(", ");
    let offered = variants.iter().map(|variant| &variant.identifier);

    Ok(quote! {
        #declared

        #[automatically_derived]
        impl #generics #sdk::prompts::PromptFieldValue for #name #types #predicates {
            const KIND: #sdk::prompts::PromptFieldKind = #sdk::prompts::PromptFieldKind::Choice;

            fn choices() -> ::std::vec::Vec<::std::string::String> {
                ::std::vec![#(::std::string::String::from(#offered)),*]
            }
        }

        #[automatically_derived]
        impl #generics #sdk::parameters::ParameterValue for #name #types #predicates {
            type Base = ::std::string::String;

            fn choices() -> ::core::option::Option<
                ::std::vec::Vec<#sdk::parameters::ParameterChoice>,
            > {
                ::core::option::Option::Some(::std::vec![#(#choices),*])
            }

            fn refine(
                base: ::std::string::String,
            ) -> ::core::result::Result<Self, ::std::string::String> {
                match base.as_str() {
                    #(#refinements)*
                    other => ::core::result::Result::Err(::std::format!(
                        "{other:?} is not one of: {}", #admitted
                    )),
                }
            }
        }

        #[automatically_derived]
        impl #generics #sdk::values::FieldValue for #name #types #predicates {
            const VALUE_TYPE: #sdk::ValueType = #sdk::ValueType::Text;

            fn into_value(self) -> #sdk::Value {
                match self { #(#encoded)* }
            }

            fn from_value(value: ::core::option::Option<&#sdk::Value>) -> ::core::result::Result<Self, #sdk::values::FieldError> {
                let base = <::std::string::String as #sdk::values::FieldValue>::from_value(value)?;
                <Self as #sdk::parameters::ParameterValue>::refine(base).map_err(|reason| #sdk::values::FieldError {
                    field: ::std::string::String::new(),
                    declared: #sdk::ValueType::Text,
                    found: ::core::option::Option::Some(reason),
                })
            }
        }

    })
}

fn variant(variant: &Variant) -> Result<ChoiceVariant, Error> {
    if !matches!(variant.fields, Fields::Unit) {
        return Err(Error::new(variant.fields.span(), "a choice's variants carry no data: one variant is one value"));
    }
    let mut identifier = None;
    let mut label = None;
    for attribute in variant.attrs.iter().filter(|attribute| attribute.path().is_ident("choice")) {
        for entry in attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?.into_iter() {
            let Meta::NameValue(named) = &entry else {
                return Err(Error::new(entry.span(), "an argument is `name = value`"));
            };
            let Some(name) = named.path.get_ident() else {
                return Err(Error::new(named.path.span(), "an argument is one bare name"));
            };
            match name.to_string().as_str() {
                "id" => identifier = Some(literal(&named.value)?),
                "label" => label = Some(named.value.to_token_stream()),
                other => {
                    return Err(Error::new(
                        name.span(),
                        format!("`{other}` is not a choice argument; a variant takes {}", VARIANT_ARGUMENTS.join(", ")),
                    ));
                }
            }
        }
    }
    let written = variant.ident.unraw().to_string();

    Ok(ChoiceVariant {
        identifier: identifier.unwrap_or_else(|| LitStr::new(&underscored(&written), variant.ident.span())),
        label: label.unwrap_or_else(|| {
            let titled = spaced(&written);

            quote!(#titled)
        }),
        name: variant.ident.clone(),
    })
}

fn literal(value: &Expr) -> Result<LitStr, Error> {
    match value {
        Expr::Lit(literal) => match &literal.lit {
            syn::Lit::Str(text) => Ok(text.clone()),
            other => Err(Error::new(other.span(), "a choice's id is a string literal")),
        },
        other => Err(Error::new(other.span(), "a choice's id is a string literal: it is what the resolved case carries")),
    }
}

fn spaced(name: &str) -> String {
    let mut written = String::with_capacity(name.len() + 4);
    for (index, character) in name.chars().enumerate() {
        if character.is_ascii_uppercase() && index > 0 {
            written.push(' ');
        }
        written.push(character);
    }

    written
}

fn stripped_choice(input: &DeriveInput) -> TokenStream {
    crate::arguments::stripped(input, "choice").to_token_stream()
}
