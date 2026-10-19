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
use std::collections::BTreeSet;

// Third Party Imports
use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{DeriveInput, Error, Expr, Ident, LitStr, Meta, Token, Type};

// Local Imports
use crate::arguments::Shape;

/* === Constants === */

const SECTION_ARGUMENTS: &[&str] = &["label", "description", "status"];
const FIELD_ARGUMENTS: &[&str] = &["label", "description", "default", "min", "max", "step", "from_environment", "options", "discovered"];
const FIELD_FLAGS: &[&str] = &["secret"];
const NUMBERS: &[&str] = &["u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "isize", "f32", "f64"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct SettingDeclaration {
    described: TokenStream,
    name: Ident,
    identifier: LitStr,
    leaf: Type,
    shape: Shape,
    kind: Kind,
    label: TokenStream,
    written: FieldArguments,
}

#[derive(Default)]
struct FieldArguments {
    label: Option<TokenStream>,
    description: Option<Expr>,
    default: Option<Expr>,
    min: Option<Expr>,
    max: Option<Expr>,
    step: Option<Expr>,
    secret: bool,
    from_environment: Option<Expr>,
    options: Option<Type>,
    discovered: Option<Type>,
    visible_when: Option<(Ident, Expr)>,
}

/* === Enums === */

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Boolean,
    Text,
    Path,
    Number,
    Choice,
    Secret,
}

/* === Traits === */

/* === Implementations === */

impl Kind {
    fn named(self) -> &'static str {
        match self {
            Self::Boolean => "a boolean",
            Self::Text => "a text",
            Self::Path => "a path",
            Self::Number => "a number",
            Self::Choice => "a choice",
            Self::Secret => "a secret",
        }
    }
}

/* === Public Functions === */

pub(crate) fn section(attribute: TokenStream, item: TokenStream) -> TokenStream {
    match declare(attribute, &item) {
        Ok(declared) => crate::facade::contracted(declared),
        Err(refused) => {
            let refusal = refused.to_compile_error();

            quote! { #item #refusal }
        }
    }
}

pub(crate) fn settings(attribute: TokenStream, item: TokenStream) -> TokenStream {
    match aggregate(attribute, &item) {
        Ok(declared) => crate::facade::contracted(declared),
        Err(refused) => {
            let refusal = refused.to_compile_error();

            quote! { #item #refusal }
        }
    }
}

/* === Private Functions === */

fn aggregate(attribute: TokenStream, item: &TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    crate::arguments::Arguments::read(attribute, "a settings aggregate", &[])?;
    let input: DeriveInput = syn::parse2(item.clone())?;
    let name = &input.ident;
    if !input.generics.params.is_empty() {
        return Err(Error::new(input.generics.span(), "a settings aggregate is concrete: one plugin is opened from one card"));
    }
    let named = crate::arguments::named_struct_fields(
        &input,
        "a plugin's settings are a struct whose fields are its sections; a plugin with none \
         writes `dokime_runtime::plugin::NoSettings` in `initialize` instead",
        "a settings aggregate names its fields: the field order is the card order, and a plugin \
         with nothing to configure writes `dokime_runtime::plugin::NoSettings` in `initialize` instead",
    )?;
    let fields = named.named.iter().map(|field| field.ident.clone().expect("a named field carries an identifier")).collect::<Vec<_>>();
    let sections = named.named.iter().map(|field| field.ty.clone()).collect::<Vec<_>>();
    let section = quote!(#sdk::plugin::settings::SettingsSection);
    Ok(quote! {
        #input

        #[automatically_derived]
        impl #sdk::plugin::settings::PluginSettings for #name {
            fn catalog() -> #sdk::settings::Settings {
                let mut contributions = ::std::vec::Vec::new();
                #(contributions.extend(<#sections as #section>::entries());)*

                #sdk::settings::Settings { contributions }
            }

            fn status_operation() -> ::core::option::Option<&'static str> {
                ::core::option::Option::None
                    #(.or(<#sections as #section>::status_operation()))*
            }

            fn option_providers()
                -> ::std::vec::Vec<#sdk::domains::OptionProviderContribution> {
                let mut offered = ::std::vec::Vec::new();
                #(offered.extend(<#sections as #section>::option_providers());)*

                offered
            }

            fn decode(
                values: &#sdk::settings::SettingValues,
                context: &#sdk::plugin::Configuration,
            ) -> #sdk::Result<Self> {
                let _ = (values, context);

                ::core::result::Result::Ok(Self {
                    #(#fields: <#sections as #section>::decode(values, context)?,)*
                })
            }

            fn discoveries() -> ::std::vec::Vec<#sdk::plugin::settings::Discovery> {
                let mut found = ::std::vec::Vec::new();
                #(found.extend(<#sections as #section>::discoveries());)*

                found
            }
        }

    })
}

fn declare(attribute: TokenStream, item: &TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let arguments = crate::arguments::Arguments::read(attribute, "a settings section", SECTION_ARGUMENTS)?;
    let input: DeriveInput = syn::parse2(item.clone())?;
    if !input.generics.params.is_empty() {
        return Err(Error::new(input.generics.span(), "a settings section is concrete: its fields are the settings an operator fills in"));
    }
    let fields = declarations(&input)?;
    let name = &input.ident;
    let described = crate::arguments::description(arguments.value("description"), &input.attrs).unwrap_or_else(|| {
        let empty = String::new();
        quote!(#empty)
    });
    let label = match arguments.value("label") {
        Some(label) => label.to_token_stream(),
        None => {
            let titled = crate::arguments::titled(&name.unraw().to_string());

            quote!(#titled)
        }
    };
    let status = match arguments.value("status") {
        Some(written) => Some(request(written)?),
        None => None,
    };
    let status_operation = match &status {
        Some(request) => quote_spanned!(request.span()=>
            ::core::option::Option::Some(#sdk::plugin::settings::operation_of::<#request>())
        ),
        None => quote!(::core::option::Option::None),
    };

    let declared = stripped(&input);
    let entries = fields.iter().map(entry).collect::<Result<Vec<_>, _>>()?;
    let reads = fields.iter().map(read).collect::<Vec<_>>();
    let providers = fields
        .iter()
        .filter_map(|field| field.written.options.as_ref())
        .map(|request| {
            quote_spanned!(request.span()=>
                #sdk::plugin::settings::option_provider(
                    #sdk::plugin::settings::operation_of::<#request>(),
                )
            )
        })
        .collect::<Vec<_>>();
    let discoveries = fields
        .iter()
        .filter_map(|field| {
            let request = field.written.discovered.as_ref()?;
            let identifier = &field.identifier;

            Some(quote_spanned!(request.span()=>
                #sdk::plugin::settings::discovery(
                    #identifier,
                    #sdk::plugin::settings::operation_of::<#request>(),
                )
            ))
        })
        .collect::<Vec<_>>();

    // What each kind of link is is what the operation answers, asserted where
    // the link is written: a probe answers the SDK's own status, a dropdown
    // source its options, and a discovery the value the field decodes. The bound
    // is read as a value, so the refusal is spanned on the request the author
    // wrote and names it, the section it is written in and the response it was
    // expected to answer.
    let asserted = linked(&fields, status.as_ref()).map(|(_, request, answered)| {
        quote_spanned!(request.span()=>
            const _: () = {

                #[allow(dead_code)]
                const LINK: fn() = #sdk::plugin::settings::answers::<#request, #answered>;
            };
        )
    });
    Ok(quote! {
        #declared

        #(#asserted)*

        #[automatically_derived]
        impl #sdk::plugin::settings::SettingsSection for #name {
            const LABEL: &'static str = #label;
            const DESCRIPTION: &'static str = #described;

            fn entries() -> ::std::vec::Vec<#sdk::settings::SettingsEntry> {
                ::std::vec![#(#entries),*]
            }

            fn status_operation() -> ::core::option::Option<&'static str> {
                #status_operation
            }

            fn option_providers()
                -> ::std::vec::Vec<#sdk::domains::OptionProviderContribution> {
                ::std::vec![#(#providers),*]
            }

            fn decode(
                values: &#sdk::settings::SettingValues,
                context: &#sdk::plugin::Configuration,
            ) -> #sdk::Result<Self> {
                let _ = (values, context);

                ::core::result::Result::Ok(Self { #(#reads),* })
            }

            fn discoveries() -> ::std::vec::Vec<#sdk::plugin::settings::Discovery> {
                ::std::vec![#(#discoveries),*]
            }
        }

    })
}

fn linked<'a>(fields: &'a [SettingDeclaration], status: Option<&'a Type>) -> impl Iterator<Item = (String, &'a Type, TokenStream)> {
    let sdk = crate::facade::root();
    let probe = status.map(|request| ("its readiness probe".to_owned(), request, quote!(#sdk::domains::StatusResponse)));
    let offered = fields.iter().filter_map(move |field| {
        let sdk = crate::facade::root();

        Some((format!("the choices of `{}`", field.name.unraw()), field.written.options.as_ref()?, quote!(#sdk::domains::OptionsResponse)))
    });
    let found = fields.iter().filter_map(move |field| {
        let sdk = crate::facade::root();

        Some((
            format!("the discovered default of `{}`", field.name.unraw()),
            field.written.discovered.as_ref()?,
            quote!(#sdk::plugin::settings::Discovered),
        ))
    });

    probe.into_iter().chain(offered).chain(found)
}

fn request(written: &Expr) -> Result<Type, Error> {
    let Expr::Path(path) = written else {
        return Err(Error::new(written.span(), "a link names the operation type, as `status = GetStatus`"));
    };
    let path = &path.path;

    Ok(syn::parse_quote_spanned!(path.span()=> #path))
}

fn declarations(input: &DeriveInput) -> Result<Vec<SettingDeclaration>, Error> {
    let named = crate::arguments::named_struct_fields(
        input,
        "a settings section is a struct: its fields are the settings it declares",
        "a settings section names its fields: a field's name is its setting's id",
    )?;
    let siblings =
        named.named.iter().filter_map(|field| field.ident.as_ref().map(|name| name.unraw().to_string())).collect::<BTreeSet<_>>();
    named
        .named
        .iter()
        .map(|field| {
            let name = field.ident.clone().expect("a named field carries an identifier");
            let written = arguments(field)?;
            let (shape, leaf) = crate::arguments::shape(&field.ty);
            let kind = kind(&leaf, &written);
            validate(&name, kind, shape, &written, &siblings)?;
            let identifier = LitStr::new(&name.unraw().to_string(), name.span());
            let label = written.label.clone().unwrap_or_else(|| {
                let titled = crate::arguments::titled(&name.unraw().to_string());

                quote!(#titled)
            });

            Ok(SettingDeclaration {
                described: crate::arguments::described(written.description.as_ref(), &field.attrs),
                identifier,
                name,
                leaf,
                shape,
                kind,
                label,
                written,
            })
        })
        .collect()
}

fn kind(leaf: &Type, written: &FieldArguments) -> Kind {
    let named = match leaf {
        Type::Path(path) => path.path.segments.last().map(|segment| segment.ident.to_string()),
        _ => None,
    };
    match named.as_deref() {
        Some("bool") => Kind::Boolean,
        Some("String" | "str") => Kind::Text,
        Some("PathBuf" | "Path") => Kind::Path,
        Some("Secret") => Kind::Secret,
        Some(number) if NUMBERS.contains(&number) => Kind::Number,
        // A type the SDK does not own declares its own closed set, which is
        // what `#[dokime_runtime::choice]` writes; a `secret` flag on anything else is
        // refused beside this, where the field's own span is still available.
        _ if written.secret => Kind::Secret,
        _ => Kind::Choice,
    }
}

fn validate(name: &Ident, kind: Kind, shape: Shape, written: &FieldArguments, siblings: &BTreeSet<String>) -> Result<(), Error> {
    if written.secret && !matches!(kind, Kind::Secret) {
        return Err(Error::new(name.span(), "a secret is declared by its type: write the field as `dokime_runtime::plugin::Secret`"));
    }
    if written.from_environment.is_some() && !matches!(kind, Kind::Secret) {
        return Err(Error::new(name.span(), "only a secret reads its default from the environment"));
    }
    if (written.min.is_some() || written.max.is_some() || written.step.is_some()) && !matches!(kind, Kind::Number) {
        return Err(Error::new(name.span(), format!("min, max and step bound a number, and this field declares {}", kind.named())));
    }
    if written.options.is_some() && !matches!(kind, Kind::Choice) {
        return Err(Error::new(name.span(), format!("`options = …` fills a choice, and this field declares {}", kind.named())));
    }
    if matches!(kind, Kind::Secret) && !matches!(shape, Shape::Required | Shape::Optional) {
        return Err(Error::new(name.span(), "a secret is one binding: declare it as `Secret` or `Option<Secret>`"));
    }
    // A plugin has to be able to initialize before discovery has ever run, so
    // what discovery fills in is a field the plugin can hold empty.
    if written.discovered.is_some() && !matches!(shape, Shape::Optional | Shape::OptionalList) {
        return Err(Error::new(name.span(), "a discovered setting is filled in after the plugin opens: declare it as `Option<T>`"));
    }
    if let Some((sibling, _)) = &written.visible_when
        && !siblings.contains(&sibling.unraw().to_string())
    {
        return Err(Error::new(sibling.span(), format!("`{sibling}` is not a field of this section: visible_when names a sibling")));
    }

    Ok(())
}

fn arguments(field: &syn::Field) -> Result<FieldArguments, Error> {
    let mut written = FieldArguments::default();
    // `visible_when(sibling = value)` is the one argument written as a set, so
    // it is read here rather than through the shared name/value reader.
    let mut remaining = field.attrs.clone();
    for attribute in &field.attrs {
        if !attribute.path().is_ident("setting") {
            continue;
        }
        for entry in attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
            let Meta::List(list) = &entry else {
                continue;
            };
            if !list.path.is_ident("visible_when") {
                continue;
            }
            let pairs = crate::arguments::pairs(list)?;
            let [(sibling, value)] = pairs.as_slice() else {
                return Err(Error::new(list.span(), "visible_when names one sibling and the value that shows this field"));
            };
            if written.visible_when.replace((sibling.clone(), value.clone())).is_some() {
                return Err(Error::new(list.span(), "a field states one visibility"));
            }
        }
    }
    // The shared reader refuses a set argument, so the one this declaration owns
    // is taken out of the attribute before the rest of it is read.
    let mut without = field.clone();
    without.attrs = std::mem::take(&mut remaining)
        .into_iter()
        .map(|attribute| match attribute.path().is_ident("setting") {
            true => stripped_visibility(attribute),
            false => Ok(attribute),
        })
        .collect::<Result<Vec<_>, _>>()?;

    written.secret =
        crate::arguments::field_arguments(&without, "setting", "a setting argument", FIELD_ARGUMENTS, FIELD_FLAGS)?.flag("secret");
    for (name, value) in
        crate::arguments::field_arguments(&without, "setting", "a setting argument", FIELD_ARGUMENTS, FIELD_FLAGS)?.values()
    {
        match name.to_string().as_str() {
            "label" => written.label = Some(value.to_token_stream()),
            "description" => written.description = Some(value),
            "default" => written.default = Some(value),
            "min" => written.min = Some(value),
            "max" => written.max = Some(value),
            "step" => written.step = Some(value),
            "from_environment" => written.from_environment = Some(value),
            "options" | "discovered" => {
                let linked = request(&value)?;
                match name == "options" {
                    true => written.options = Some(linked),
                    false => written.discovered = Some(linked),
                }
            }
            other => unreachable!("`{other}` is refused where the arguments are read"),
        }
    }

    Ok(written)
}

fn stripped_visibility(attribute: syn::Attribute) -> Result<syn::Attribute, Error> {
    let retained = attribute
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
        .into_iter()
        .filter(|entry| !matches!(entry, Meta::List(list) if list.path.is_ident("visible_when")))
        .collect::<Punctuated<Meta, Token![,]>>();

    Ok(syn::parse_quote!(#[setting(#retained)]))
}

fn entry(field: &SettingDeclaration) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let identifier = &field.identifier;
    let label = &field.label;
    let leaf = &field.leaf;
    let described = &field.described;
    // Only a value an operator must supply is required, which is the same thing
    // the field's own shape already says: `Option<T>` is what a plugin can open
    // without.
    let required = matches!(field.shape, Shape::Required | Shape::List);
    let bound = |bound: &Option<Expr>| match bound {
        Some(value) => quote!(::core::option::Option::Some((#value) as f64)),
        None => quote!(::core::option::Option::None),
    };

    let declared = match field.kind {
        Kind::Boolean => {
            let default = match &field.written.default {
                Some(default) => quote!(#default),
                None => quote!(false),
            };

            quote! {
                #sdk::settings::SettingsEntry::Boolean(#sdk::settings::Boolean {
                    id: ::std::string::ToString::to_string(#identifier),
                    title: ::std::string::ToString::to_string(#label),
                    description: #described,
                    default: #default,
                })
            }
        }
        Kind::Text => {
            let default = match &field.written.default {
                Some(default) => quote!(::std::string::ToString::to_string(&#default)),
                None => quote!(::std::string::String::new()),
            };

            quote! {
                #sdk::settings::SettingsEntry::TextArea(#sdk::settings::TextArea {
                    id: ::std::string::ToString::to_string(#identifier),
                    title: ::std::string::ToString::to_string(#label),
                    description: #described,
                    default: #default,
                    required: #required,
                })
            }
        }
        Kind::Path => {
            let default = match &field.written.default {
                Some(default) => {
                    quote!(#sdk::plugin::settings::written_path(#default))
                }
                None => quote!(::std::string::String::new()),
            };

            quote! {
                #sdk::settings::SettingsEntry::Path(#sdk::settings::Path {
                    id: ::std::string::ToString::to_string(#identifier),
                    title: ::std::string::ToString::to_string(#label),
                    description: #described,
                    default: #default,
                    required: #required,
                })
            }
        }
        Kind::Number => {
            let default = match &field.written.default {
                Some(default) => quote!((#default) as f64),
                None => quote!(0.0),
            };
            let min = bound(&field.written.min);
            let max = bound(&field.written.max);
            let step = bound(&field.written.step);

            quote! {
                #sdk::settings::SettingsEntry::Number(#sdk::settings::Number {
                    id: ::std::string::ToString::to_string(#identifier),
                    title: ::std::string::ToString::to_string(#label),
                    description: #described,
                    default: #default,
                    min: #min,
                    max: #max,
                    step: #step,
                })
            }
        }
        Kind::Choice => {
            let default = match &field.written.default {
                Some(default) => quote_spanned!(leaf.span()=>
                    #sdk::plugin::settings::selected::<#leaf>(#default)
                ),
                None => quote_spanned!(leaf.span()=>
                    #sdk::plugin::settings::first_choice::<#leaf>()
                ),
            };
            let provider = match &field.written.options {
                Some(request) => quote_spanned!(request.span()=>
                    ::core::option::Option::Some(::std::string::ToString::to_string(
                        #sdk::plugin::settings::operation_of::<#request>(),
                    ))
                ),
                None => quote!(::core::option::Option::None),
            };

            quote! {
                #sdk::settings::SettingsEntry::MultipleChoice(#sdk::settings::MultipleChoice {
                    id: ::std::string::ToString::to_string(#identifier),
                    title: ::std::string::ToString::to_string(#label),
                    description: #described,
                    default: #default,
                    choices: #sdk::plugin::settings::choices::<#leaf>(),
                    provider: #provider,
                })
            }
        }
        Kind::Secret => {
            let default = match &field.written.from_environment {
                Some(variable) => quote! {
                    ::core::option::Option::Some(#sdk::settings::SecretReference::Environment {
                        variable: ::std::string::ToString::to_string(#variable),
                    })
                },
                None => quote!(::core::option::Option::None),
            };

            quote! {
                #sdk::settings::SettingsEntry::SecretReference(#sdk::settings::SecretSetting {
                    id: ::std::string::ToString::to_string(#identifier),
                    title: ::std::string::ToString::to_string(#label),
                    description: #described,
                    default: #default,
                    required: #required,
                })
            }
        }
    };

    Ok(declared)
}

fn read(field: &SettingDeclaration) -> TokenStream {
    let sdk = crate::facade::root();
    let name = &field.name;
    let identifier = &field.identifier;
    let leaf = &field.leaf;
    // A secret never travels in the values: it is bound by the host and read
    // out of the context, which is why the decode is handed both.
    if matches!(field.kind, Kind::Secret) {
        let read = match field.shape {
            Shape::Optional => quote!(#sdk::plugin::settings::bound_secret(context, #identifier)),
            _ => quote!(#sdk::plugin::settings::secret(context, #identifier)?),
        };

        return quote!(#name: #read);
    }
    // An optional text setting is absent when an operator left it blank: a
    // text field always carries a value, and the empty string is what an
    // unfilled one holds.
    if matches!(field.kind, Kind::Text) && matches!(field.shape, Shape::Optional) {
        return quote!(#name: #sdk::plugin::settings::optional_text(values, #identifier)?);
    }
    // A path is text on the wire, and the field's own type is what turns it
    // back into one.
    if matches!(field.kind, Kind::Path) {
        let read = match field.shape {
            Shape::Optional => quote! {
                #sdk::plugin::settings::optional_path(values, #identifier)?
            },
            _ => quote!(#sdk::plugin::settings::path(values, #identifier)?),
        };

        return quote!(#name: #read);
    }
    let reader = match field.shape {
        Shape::Required => quote!(scalar),
        Shape::Optional => quote!(optional),
        Shape::List => quote!(list),
        Shape::OptionalList => quote!(optional_list),
    };
    // A number is carried as one `f64`, so an integer field reads what the
    // catalog could not spell back out of it first.
    let source = match field.kind {
        Kind::Number => {
            quote_spanned!(leaf.span()=> &#sdk::plugin::settings::numbers::<#leaf>(values, #identifier))
        }
        _ => quote!(values),
    };
    let read = quote_spanned!(leaf.span()=>
        #sdk::parameters::#reader::<#leaf>(
            #source,
            #identifier,
            #sdk::parameters::Bounds::default(),
        )?
    );

    quote!(#name: #read)
}

fn stripped(input: &DeriveInput) -> TokenStream {
    crate::arguments::stripped(input, "setting").to_token_stream()
}

/* === Tests === */
