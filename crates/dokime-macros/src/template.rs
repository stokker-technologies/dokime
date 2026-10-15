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
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    Error, Expr, FnArg, GenericArgument, ImplItem, ImplItemFn, ItemImpl, Meta, Path, PathArguments,
    ReturnType, Type,
};
// Local Imports
use crate::arguments::{self, Arguments};

/* === Constants === */
const ARGUMENTS: &[&str] = &[
    "id",
    "label",
    "description",
    "template_id",
    "tags",
    "attributes",
    "prompts",
    "subscribes",
    "graceful_stop",
    "requires",
    "parameters",
    "streams",
    "records",
];

const DOCUMENT: &str = "document";
const CRITERIA: &str = "criteria";

const LIFECYCLE: &[&str] = &["initialize", "traceability", "cleanup"];

const RUNNABLE_CONTEXT: &str = "StepContext";

const CONTEXTS: &[&str] = &[
    "InitializationContext",
    "TraceabilityContext",
    "StepContext",
    "ObserverContext",
    "CleanupContext",
];

const CRITERION_CONTEXT: &str = "CriterionContext";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Stages<'a> {
    steps: &'a [DeclaredStep],
    criteria: &'a [DeclaredCriterion],
    observers: &'a [DeclaredObserver],
}

struct Grant {
    template: Type,
    own: Type,
    parameters: Type,
}

struct DeclaredStep {
    method: syn::Ident,
    identifier: syn::Expr,
    awaited: bool,
    declaration: TokenStream,
}

struct DeclaredCriterion {
    awaited: bool,
    method: syn::Ident,
    identifier: syn::Expr,
    declaration: TokenStream,
}

struct DeclaredObserver {
    awaited: bool,
    method: syn::Ident,
    streams: TokenStream,
    declaration: TokenStream,
    contracts: TokenStream,
    records: TokenStream,
}

/* === Enums === */
#[derive(Clone, Copy)]
pub(crate) enum Part {
    Ok,
    Err,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn template(attribute: TokenStream, item: TokenStream) -> TokenStream {
    match expand(attribute, &item) {
        Ok(declared) => crate::facade::contracted(declared),
        Err(refused) => {
            let refusal = refused.to_compile_error();

            quote! { #item #refusal }
        }
    }
}

pub(crate) fn marker(name: &str, item: TokenStream) -> TokenStream {
    let refusal = Error::new(
        item.span(),
        format!(
            "`#[dokime::{name}]` is read by its owning block: declare it inside `#[dokime::template]`"
        ),
    )
    .to_compile_error();

    quote! { #item #refusal }
}

pub(crate) fn marked<'a>(
    block: &'a ItemImpl,
    name: &'a str,
) -> impl Iterator<Item = (&'a ImplItemFn, &'a Meta)> {
    block.items.iter().filter_map(move |item| {
        let ImplItem::Fn(function) = item else {
            return None;
        };
        let marker = function
            .attrs
            .iter()
            .find(|attribute| marks(attribute.meta.path(), name))?;

        Some((function, &marker.meta))
    })
}

pub(crate) fn list(marker: &Meta, kind: &str, known: &[&str]) -> Result<Arguments, Error> {
    let Meta::List(list) = marker else {
        return Err(Error::new(
            marker.span(),
            format!("{kind} declares {}", known.join(" and ")),
        ));
    };

    Arguments::read(list.tokens.clone(), kind, known)
}

fn observed(signature: &syn::Signature) -> Result<(), Error> {
    const SHAPE: &str = "a template observer must be \
        `async fn name(ctx: &mut ObserverContext<'_>) -> Result<(), Error>`: \
        it takes one exclusive context and no receiver, and publishes what it saw \
        with `ctx.publish(value).await?`";
    if signature.asyncness.is_none() {
        return Err(Error::new(
            signature.span(),
            "a template observer is async: a tick awaits whatever it is watching",
        ));
    }
    if signature.receiver().is_some()
        || signature.inputs.len() != 1
        || !signature.generics.params.is_empty()
        || matches!(signature.safety, syn::Safety::Unsafe(_))
    {
        return Err(Error::new(signature.inputs.span(), SHAPE));
    }
    let FnArg::Typed(context) = &signature.inputs[0] else {
        return Err(Error::new(signature.inputs.span(), SHAPE));
    };
    let Type::Reference(reference) = context.ty.as_ref() else {
        return Err(Error::new(context.ty.span(), SHAPE));
    };
    if reference.mutability.is_none() {
        return Err(Error::new(
            context.ty.span(),
            "a template observer borrows its context exclusively: write \
             `ctx: &mut ObserverContext<'_>`",
        ));
    }
    let Type::Path(path) = reference.elem.as_ref() else {
        return Err(Error::new(context.ty.span(), SHAPE));
    };
    if path
        .path
        .segments
        .last()
        .is_none_or(|segment| segment.ident != "ObserverContext")
    {
        return Err(Error::new(context.ty.span(), SHAPE));
    }
    if matches!(signature.output, ReturnType::Default) {
        return Err(Error::new(
            signature.span(),
            "a template observer answers `Result<(), Error>`",
        ));
    }

    Ok(())
}

pub(crate) fn answer(reply: &Type, part: Part) -> Type {
    let sdk = crate::facade::root();
    match part {
        Part::Ok => syn::parse_quote!(<#reply as #sdk::__answer::Answer>::Ok),
        Part::Err => syn::parse_quote!(<#reply as #sdk::__answer::Answer>::Err),
    }
}

pub(crate) fn declared<'a>(block: &'a ItemImpl, name: &str) -> Option<&'a ImplItemFn> {
    block.items.iter().find_map(|item| match item {
        ImplItem::Fn(function) if function.sig.ident == name => Some(function),
        _ => None,
    })
}

pub(crate) fn marks(path: &Path, name: &str) -> bool {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();

    match segments
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [marker] => *marker == name,
        [root, marker] => (*root == "dokime" || *root == crate::facade::name()) && *marker == name,
        _ => false,
    }
}

/* === Private Functions === */
fn expand(attribute: TokenStream, item: &TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let arguments = Arguments::read(attribute, "a template", ARGUMENTS)?;
    let mut block: ItemImpl = syn::parse2(item.clone())?;
    if let Some((path, _)) = &block.trait_ {
        return Err(Error::new(
            path.span(),
            "`#[dokime::template]` declares the template's own inherent impl, not a trait implementation",
        ));
    }
    let template = block.self_ty.clone();

    let identifier = arguments
        .required(
            "id",
            "a template is referenced as `<suite id>.<segment>`, which its item name cannot spell",
        )?
        .clone();
    refuse_malformed_segment(&identifier)?;
    let label = arguments.required("label", "an operator reads the template by its label")?;
    let metadata = arguments::metadata(
        &identifier,
        label,
        arguments.value("description"),
        &block.attrs,
    );
    let template_id = match arguments.value("template_id") {
        Some(declared) => quote!(#declared),
        None => quote!(#sdk::core::TemplateId::from_seed(#identifier)),
    };

    let parameters = parameters(&block, arguments.value("parameters"))?;
    refuse_unrecognized_members(&block)?;
    let steps = steps(&block)?;
    if let Some(step) = steps
        .iter()
        .find(|step| LIFECYCLE.iter().any(|hook| step.method == hook))
    {
        return Err(Error::new_spanned(
            &step.method,
            "initialize, traceability and cleanup are execution lifecycle hooks, not declared steps",
        ));
    }
    let criteria = criteria(&block)?;
    let runnable = runnable(&block, &steps)?;
    let observers = observers(&block, runnable)?;
    if runnable {
        validate_lifecycle(&block)?;
    }
    let bindings = match runnable {
        // A runnable template's context carries a lifetime, which has no
        // spelling in the return type of a binding table; the engine reaches
        // its steps through `SidecarTemplate` instead.
        true => None,
        false => table(&block, &steps)?,
    };
    let phases = steps.iter().map(|step| &step.declaration);
    let streams = crate::catalog::argument(&arguments, "streams")?;
    let contracts = stream_contracts(&observers, &streams);
    let records = record_contracts(
        &crate::catalog::argument(&arguments, "records")?,
        &observers,
    );
    let graceful_stop = arguments.value("graceful_stop");
    let requested = crate::catalog::argument(&arguments, "subscribes")?;
    let duplicate_holds = refuse_double_holds(&block, &requested)?;
    let required = crate::catalog::argument(&arguments, "requires")?;
    let calls = requires(&template, &required);
    let engine = match runnable {
        true => {
            let grant = Grant {
                template: (*template).clone(),
                own: own_resources(&block)?,
                parameters: parameters.clone(),
            };
            let operations = operation_ids(&identifier)?;
            let generated = engine(
                &block,
                &grant,
                Stages {
                    steps: &steps,
                    criteria: &criteria,
                    observers: &observers,
                },
                &streams,
                graceful_stop,
                &operations,
                &required,
            )?;
            // Written after the engine is generated and before the block is
            // emitted: an author writes `StepContext<'_>` and the three type
            // arguments are filled in here, so the bare alias in authored code
            // and the engine's own signatures name one type.
            resolve_contexts(&mut block, &grant);
            Some(generated)
        }
        false => None,
    };
    strip(&mut block);

    let tags = arguments
        .value("tags")
        .map(arguments::elements)
        .transpose()?
        .map(|tags| quote!(ctx.set_tags(::std::vec![#(#tags),*]);));
    // Prompts are declared here so the host can check them against the
    // installed suite. A run cannot widen its own grant once it is going.
    let prompts = crate::catalog::argument(&arguments, "prompts")?;
    let prompts = quote!(ctx.set_prompts(
        <#prompts as #sdk::catalog::Catalog<#sdk::catalog::Prompts>>::ENTRIES.iter().map(|entry| entry())
    ););
    let subscriptions = subscriptions(&requested);
    let attributes = arguments
        .set("attributes")
        .map(arguments::pairs)
        .transpose()?
        .unwrap_or_default()
        .into_iter()
        .map(|(key, value)| {
            let key = key.to_string();

            quote!(ctx.set_attribute(#key, #sdk::core::Value::from(#value));)
        })
        .collect::<Vec<_>>();

    let declared_parameters = arguments.value("parameters").map(|_| {
        quote!(ctx.set_parameters(
            <#parameters as #sdk::parameters::FromParameters>::declaration()
        );)
    });
    let document =
        declared(&block, DOCUMENT).map(|_| quote!(ctx.set_document(<Self>::document());));
    let declared_criteria = match criteria.as_slice() {
        // Written as methods, the criteria *are* the declaration: an author who
        // said what each one measures has said what they are.
        [_, ..] => {
            let declarations = criteria.iter().map(|criterion| &criterion.declaration);

            Some(quote!(ctx.set_criteria(::std::vec![#(#declarations),*]);))
        }
        [] => declared(&block, CRITERIA).map(|_| quote!(ctx.set_criteria(<Self>::criteria());)),
    };

    Ok(quote! {
        #block

        #[automatically_derived]
        impl #template {
                                    pub fn template_id() -> #sdk::core::TemplateId {
                #template_id
            }

            #bindings

            fn __dokime_declare(ctx: &mut #sdk::definition::TemplateContext) {
                ctx.set_template_id(<Self>::template_id());
                ctx.set_metadata(#metadata);
                #tags
                #declared_parameters
                #document
                ctx.set_steps(::std::vec![#(#phases),*]);
                #declared_criteria
                #prompts
                #subscriptions
                #duplicate_holds
                #(#attributes)*
            }
        }

        #[automatically_derived]
        impl #sdk::definition::RustTemplate for #template {
            fn define(&self, ctx: &mut #sdk::definition::TemplateContext) {
                <Self>::__dokime_declare(ctx);
            }
        }

        #[automatically_derived]
        impl #sdk::definition::DeclaredTemplate for #template {
            const IDENTIFIER: &'static str = #identifier;

            fn template_id() -> #sdk::core::TemplateId {
                <Self>::template_id()
            }

            fn declaration() -> #sdk::definition::TestCaseTemplate {
                let mut ctx = #sdk::definition::TemplateContext::new();
                <Self>::__dokime_declare(&mut ctx);
                match ctx.into_test_case_template() {
                    ::core::result::Result::Ok(declaration) => declaration,
                    ::core::result::Result::Err(refusal) => ::core::panic!(
                        "template `{}` is not declarable: {}", #identifier, refusal
                    ),
                }
            }

            #contracts
            #records
        }

        #engine

        #calls
    })
}

fn requires(template: &Type, required: &TokenStream) -> TokenStream {
    let sdk = crate::facade::root();

    quote! {
        #[automatically_derived]
        impl #sdk::execution::Required for #template {
            type Operations = #required;
        }
    }
}

// A literal id is read on the line it was written: it names its suite and one
// lowercase segment. The suite checks the prefix and uniqueness, and checks a
// constant id the same way once it is evaluated.
fn refuse_malformed_segment(identifier: &Expr) -> Result<(), Error> {
    let Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(text),
        ..
    }) = identifier
    else {
        return Ok(());
    };
    let value = text.value();
    let segment = value.rsplit_once('.').map(|(_, segment)| segment);
    let valid = segment.is_some_and(|segment| {
        segment.starts_with(|first: char| first.is_ascii_lowercase())
            && segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    });
    if valid {
        return Ok(());
    }
    Err(Error::new(
        text.span(),
        format!(
            "`{value}` is not a template id: it is `<suite id>.<segment>`, the segment lowercase ASCII letters, digits, and underscores, starting with a letter"
        ),
    ))
}

fn operation_ids(identifier: &Expr) -> Result<(TokenStream, TokenStream), Error> {
    let sdk = crate::facade::root();
    let derived = |suffix: &str| {
        quote! {{
            const ID: &str = #identifier;
            const LENGTH: usize = ID.len() + #suffix.len();
            const BYTES: [u8; LENGTH] = #sdk::catalog::joined::<LENGTH>(ID, #suffix);
            match ::core::str::from_utf8(&BYTES) {
                ::core::result::Result::Ok(id) => id,
                ::core::result::Result::Err(_) => ::core::panic!("a template id is UTF-8"),
            }
        }}
    };
    Ok((derived(".run"), derived(".signal")))
}

fn subscriptions(streams: &TokenStream) -> TokenStream {
    let sdk = crate::facade::root();

    quote!(ctx.set_subscriptions(
        <#streams as #sdk::catalog::Catalog<#sdk::catalog::Subscriptions>>::ENTRIES.iter().map(|entry| entry()).collect()
    );)
}

fn refuse_double_holds(block: &ItemImpl, requested: &TokenStream) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let leased = leased_in_block(block);
    let checks = leased.iter().map(|(row, span)| {
        quote_spanned!(*span=>
            assert!(
                !<#requested as #sdk::catalog::Catalog<#sdk::catalog::Subscriptions>>::ENTRIES.iter()
                    .any(|entry| entry() == <#row as #sdk::execution::Listenable>::subscription()),
                "duplicate subscription: a stream in `subscribes` cannot also be leased inside the template"
            );
        )
    });

    Ok(quote!(#(#checks)*))
}

fn leased_in_block(block: &ItemImpl) -> Vec<(Path, Span)> {
    fn walk(tokens: TokenStream, found: &mut Vec<(Path, Span)>) {
        let tokens = tokens.into_iter().collect::<Vec<_>>();
        for (index, token) in tokens.iter().enumerate() {
            if let TokenTree::Group(group) = token {
                walk(group.stream(), found);
                continue;
            }
            let TokenTree::Ident(ident) = token else {
                continue;
            };
            if ident != "lease" {
                continue;
            }
            // `lease :: < Type >`, with the turbofish's two colons and the
            // angle brackets as separate punctuation.
            let mut rest = tokens[index + 1..].iter();
            if !matches!(rest.next(), Some(TokenTree::Punct(punct)) if punct.as_char() == ':') {
                continue;
            }
            if !matches!(rest.next(), Some(TokenTree::Punct(punct)) if punct.as_char() == ':') {
                continue;
            }
            if !matches!(rest.next(), Some(TokenTree::Punct(punct)) if punct.as_char() == '<') {
                continue;
            }
            let mut path_tokens = TokenStream::new();
            let mut depth = 0usize;
            for token in rest {
                if let TokenTree::Punct(punct) = token {
                    if punct.as_char() == '>' {
                        if depth == 0 {
                            break;
                        }
                        depth -= 1;
                    } else if punct.as_char() == '<' {
                        depth += 1;
                    }
                }
                path_tokens.extend(::core::iter::once(token.clone()));
            }
            if let Ok(path) = syn::parse2::<Path>(path_tokens) {
                let span = path.span();
                found.push((path, span));
            }
        }
    }

    let mut found = Vec::new();
    for item in &block.items {
        if let ImplItem::Fn(function) = item {
            let mut tokens = TokenStream::new();
            let group = proc_macro2::Group::new(Delimiter::Brace, {
                let block = &function.block;
                quote!(#block)
            });
            tokens.extend([TokenTree::Group(group)]);
            walk(tokens, &mut found);
        }
    }

    found
}

fn steps(block: &ItemImpl) -> Result<Vec<DeclaredStep>, Error> {
    let sdk = crate::facade::root();
    marked(block, "step")
        .map(|(function, meta)| {
            let arguments = list(
                meta,
                "a step",
                &["id", "label", "description", "repeatable"],
            )?;
            let identifier = arguments.identifier(&function.sig.ident)?;
            let label = arguments.required("label", "an operator reads the step by its label")?;
            let step = if arguments.flag("repeatable")? {
                quote!(#sdk::definition::Step::new(#identifier, #label).repeatable())
            } else {
                quote!(#sdk::definition::Step::new(#identifier, #label))
            };
            let declaration = described(step, arguments.value("description"), &function.attrs);

            Ok(DeclaredStep {
                method: function.sig.ident.clone(),
                identifier: identifier.clone(),
                awaited: function.sig.asyncness.is_some(),
                declaration,
            })
        })
        .collect()
}

fn criteria(block: &ItemImpl) -> Result<Vec<DeclaredCriterion>, Error> {
    let sdk = crate::facade::root();
    marked(block, "criterion")
        .map(|(function, meta)| {
            if function.sig.receiver().is_some() || function.sig.inputs.len() != 1 {
                return Err(Error::new_spanned(
                    &function.sig,
                    "a criterion must be `fn(ctx: &CriterionContext<'_>) -> Result<bool, Error>`: \
                     a criterion is an associated function taking only &CriterionContext",
                ));
            }
            let arguments = list(meta, "a criterion", &["id", "label", "description"])?;
            let identifier = arguments.identifier(&function.sig.ident)?;
            let label =
                arguments.required("label", "an operator reads the criterion by its label")?;

            Ok(DeclaredCriterion {
                awaited: function.sig.asyncness.is_some(),
                method: function.sig.ident.clone(),
                identifier: identifier.clone(),
                // A criterion written as a function answers `true` or `false`,
                // so what it is expected to answer is not a second declaration.
                declaration: described(
                    quote!(#sdk::definition::authored_criterion(
                        #identifier,
                        #label,
                        ::core::option::Option::Some(#sdk::core::Value::from(true)),
                    )),
                    arguments.value("description"),
                    &function.attrs,
                ),
            })
        })
        .collect()
}

fn observers(block: &ItemImpl, runnable: bool) -> Result<Vec<DeclaredObserver>, Error> {
    let sdk = crate::facade::root();
    marked(block, "observer")
        .map(|(function, meta)| {
            if !runnable {
                return Err(Error::new_spanned(
                    &function.sig,
                    "a template observer belongs to a runnable template: its steps take &mut StepContext",
                ));
            }
            let arguments = list(
                meta,
                "an observer",
                &["interval_ms", "streams", "records", "label"],
            )?;
            let interval = arguments.required(
                "interval_ms",
                "an observer declares how often the engine calls it",
            )?;
            observed(&function.sig)?;
            let streams = crate::catalog::argument(&arguments, "streams")?;
            let records = crate::catalog::argument(&arguments, "records")?;
            let method = function.sig.ident.clone();
            let identifier = method.to_string();
            let label = match arguments.value("label") {
                Some(label) => quote!(#label),
                None => quote!(#identifier),
            };
            let contracts = quote!(<#streams as #sdk::catalog::Catalog<#sdk::catalog::StreamContracts>>::ENTRIES
                .iter().map(|contract| contract().for_template(<Self>::template_id()).every(#interval)));
            let filed = quote!(<#records as #sdk::catalog::Catalog<#sdk::catalog::Records>>::ENTRIES
                .iter().map(|contract| contract().for_template(<Self>::template_id())));

            Ok(DeclaredObserver {
                awaited: function.sig.asyncness.is_some(),
                method,
                streams,
                declaration: quote! {
                    #sdk::execution::DeclaredObserver::new(#identifier, #label, #interval)
                        .publishing(
                            (#contracts).collect(),
                            (#filed).collect(),
                        )
                },
                contracts,
                records: filed,
            })
        })
        .collect()
}

fn stream_contracts(observers: &[DeclaredObserver], streams: &TokenStream) -> TokenStream {
    let sdk = crate::facade::root();
    let watched = observers.iter().map(|observer| &observer.contracts);

    quote! {
        fn stream_contracts() -> ::std::vec::Vec<#sdk::telemetry::TelemetryContract> {
            let mut contracts = ::std::vec::Vec::new();
            #(contracts.extend(#watched);)*

            contracts.extend(<#streams as #sdk::catalog::Catalog<#sdk::catalog::StreamContracts>>::ENTRIES
                .iter().map(|contract| contract().for_template(<Self>::template_id())));
            contracts
        }
    }
}

fn record_contracts(records: &TokenStream, observers: &[DeclaredObserver]) -> TokenStream {
    let sdk = crate::facade::root();
    let filed = observers.iter().map(|observer| &observer.records);

    quote! {
        fn record_contracts() -> ::std::vec::Vec<#sdk::records::RecordContract> {
            let mut contracts: ::std::vec::Vec<_> = <#records as #sdk::catalog::Catalog<#sdk::catalog::Records>>::ENTRIES
                .iter().map(|contract| contract().for_template(<Self>::template_id())).collect();
            #(contracts.extend(#filed);)*
            contracts
        }
    }
}

fn runnable(block: &ItemImpl, steps: &[DeclaredStep]) -> Result<bool, Error> {
    let Some(first) = steps.first() else {
        return Ok(false);
    };
    let function = declared(block, &first.method.to_string())
        .expect("a declared step came from a method on this block");
    let (context, _) = signature(function)?;
    let Type::Path(path) = &context else {
        return Ok(false);
    };

    Ok(path
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == RUNNABLE_CONTEXT))
}

fn validate_lifecycle(block: &ItemImpl) -> Result<(), Error> {
    let initialize = declared(block, "initialize").ok_or_else(|| Error::new_spanned(
        &block.self_ty,
        "a runnable template requires async fn initialize(parameters: &Parameters, ctx: &mut InitializationContext<'_>) -> Result<Self, Error>",
    ))?;
    if initialize.sig.asyncness.is_none()
        || initialize.sig.receiver().is_some()
        || !(1..=2).contains(&initialize.sig.inputs.len())
        || matches!(initialize.sig.output, ReturnType::Default)
    {
        return Err(Error::new_spanned(
            &initialize.sig,
            "`initialize` must be `async fn initialize(parameters: &Parameters, ctx: &mut InitializationContext<'_>) -> Result<Self, Error>` \
             (or without `parameters` when the template declares none): initialize must be async and take the declared parameters and &mut InitializationContext",
        ));
    }
    if let Some(traceability) = declared(block, "traceability")
        && (traceability.sig.asyncness.is_none()
            || traceability.sig.receiver().is_none()
            || traceability.sig.inputs.len() != 2
            || matches!(traceability.sig.output, ReturnType::Default))
    {
        return Err(Error::new_spanned(
            &traceability.sig,
            "`traceability` must be `async fn traceability(&self, ctx: &TraceabilityContext<'_>) -> Result<Vec<TraceabilitySection>, Error>`: \
             traceability must be async and take &self and &TraceabilityContext",
        ));
    }
    if let Some(cleanup) = declared(block, "cleanup")
        && (cleanup.sig.asyncness.is_none()
            || cleanup.sig.receiver().is_none()
            || cleanup.sig.inputs.len() != 2
            || matches!(cleanup.sig.output, ReturnType::Default))
    {
        return Err(Error::new_spanned(
            &cleanup.sig,
            "`cleanup` must be `async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error>`: \
             cleanup must be async and take &mut self and &mut CleanupContext",
        ));
    }
    Ok(())
}

fn parameters(block: &ItemImpl, declared_type: Option<&Expr>) -> Result<Type, Error> {
    if let Some(method) = declared(block, "parameters") {
        return Err(Error::new_spanned(
            &method.sig,
            "parameters are a type: declare a `#[dokime::parameters]` struct and name it with `#[dokime::template(parameters = …)]`",
        ));
    }
    let Some(declared_type) = declared_type else {
        return Ok(syn::parse_quote!(()));
    };

    syn::parse2(quote!(#declared_type)).map_err(|_| {
        Error::new(
            declared_type.span(),
            "parameters names the template's parameters struct: `parameters = SoakParameters`",
        )
    })
}

fn refuse_unrecognized_members(block: &ItemImpl) -> Result<(), Error> {
    for item in &block.items {
        match item {
            ImplItem::Fn(function) => refuse_unrecognized_method(function)?,
            ImplItem::Const(constant) => return Err(unclaimed(&constant.ident, "const")),
            ImplItem::Type(alias) => return Err(unclaimed(&alias.ident, "type")),
            other => {
                return Err(Error::new(
                    other.span(),
                    "this item is not read by `#[dokime::template]`. A template impl holds \
                     initialize, traceability, document and cleanup, plus methods \
                     marked #[dokime::step], #[dokime::criterion] or #[dokime::observer]; \
                     put helpers in a separate impl block.",
                ));
            }
        }
    }

    Ok(())
}

fn refuse_unrecognized_method(function: &ImplItemFn) -> Result<(), Error> {
    if ["step", "criterion", "observer"].iter().any(|name| {
        function
            .attrs
            .iter()
            .any(|attribute| marks(attribute.meta.path(), name))
    }) {
        return Ok(());
    }
    let name = function.sig.ident.to_string();
    if name == "resources" {
        return Err(Error::new_spanned(
            &function.sig,
            "construct shared observer handles in initialize; a separate resources hook is not supported",
        ));
    }
    if LIFECYCLE.contains(&name.as_str())
        || name == DOCUMENT
        || name == CRITERIA
        || name == "parameters"
    {
        return Ok(());
    }

    Err(unclaimed(&function.sig.ident, "method"))
}

fn unclaimed(name: &syn::Ident, kind: &str) -> Error {
    Error::new_spanned(
        name,
        format!(
            "`{name}` is not a template {kind}. A template impl holds initialize, \
             traceability, document and cleanup, plus methods marked #[dokime::step], \
             #[dokime::criterion] or #[dokime::observer]; put helpers in a separate impl block."
        ),
    )
}

fn own_resources(block: &ItemImpl) -> Result<Type, Error> {
    let Some(initialize) = declared(block, "initialize") else {
        return Ok(syn::parse_quote!(()));
    };
    let ReturnType::Type(_, reply) = &initialize.sig.output else {
        unreachable!("initialize requires an explicit result");
    };
    let sdk = crate::facade::root();
    let output = answer(reply, Part::Ok);
    let template = &block.self_ty;
    Ok(syn::parse_quote!(<#output as #sdk::__answer::Initialized<#template>>::Own))
}

fn resolve_contexts(block: &mut ItemImpl, grant: &Grant) {
    for item in &mut block.items {
        let ImplItem::Fn(function) = item else {
            continue;
        };
        for input in &mut function.sig.inputs {
            let FnArg::Typed(argument) = input else {
                continue;
            };
            resolve_context(argument.ty.as_mut(), grant);
        }
    }
}

fn resolve_context(declared: &mut Type, grant: &Grant) {
    let Type::Reference(reference) = declared else {
        return;
    };
    let Type::Path(path) = reference.elem.as_mut() else {
        return;
    };
    let Some(segment) = path.path.segments.last_mut() else {
        return;
    };
    let name = segment.ident.to_string();
    let parameters = &grant.parameters;
    // A criterion reaches the inputs and no handle at all, so its grant carries
    // the one type argument rather than three.
    if name == CRITERION_CONTEXT {
        let lifetime = lifetime(&segment.arguments);
        let template = &grant.template;
        segment.arguments =
            PathArguments::AngleBracketed(syn::parse_quote!(<#lifetime, #template, #parameters>));
        return;
    }
    if !CONTEXTS.contains(&name.as_str()) {
        return;
    }
    let Grant { template, own, .. } = grant;
    let lifetime = lifetime(&segment.arguments);
    segment.arguments =
        PathArguments::AngleBracketed(syn::parse_quote!(<#lifetime, #template, #own, #parameters>));
}

fn lifetime(arguments: &PathArguments) -> syn::Lifetime {
    match arguments {
        PathArguments::AngleBracketed(arguments) => {
            arguments.args.iter().find_map(|argument| match argument {
                GenericArgument::Lifetime(lifetime) => Some(lifetime.clone()),
                _ => None,
            })
        }
        _ => None,
    }
    .unwrap_or_else(|| syn::parse_quote!('_))
}

fn engine(
    block: &ItemImpl,
    grant: &Grant,
    stages: Stages<'_>,
    streams: &TokenStream,
    graceful_stop: Option<&Expr>,
    operations: &(TokenStream, TokenStream),
    required: &TokenStream,
) -> Result<TokenStream, Error> {
    let sdk = crate::facade::root();
    let Stages {
        steps,
        criteria,
        observers,
    } = stages;
    let (run, signal) = operations;
    // Every arm below captures at the call rather than after it, spanned on the
    // author's own method. The `#[track_caller]` location a diagnostic records
    // is then the step, criterion or observer that failed — not this file, and
    // not the engine that polled it.
    let phases = steps.iter().map(|step| {
        let identifier = &step.identifier;
        let method = &step.method;
        let call = match step.awaited {
            true => quote!(<Self>::#method(self, ctx).await),
            false => quote!(<Self>::#method(self, ctx)),
        };
        let captured = quote_spanned!(method.span()=> #sdk::plugin::captured(#call));

        quote!(#identifier => #captured,)
    });
    let decisions = criteria.iter().map(|criterion| {
        let identifier = &criterion.identifier;
        let method = &criterion.method;
        let awaited = criterion.awaited.then(|| quote!(.await));
        let captured =
            quote_spanned!(method.span()=> #sdk::plugin::captured(<Self>::#method(ctx)#awaited));

        quote!(#identifier => #captured.map(::core::option::Option::Some),)
    });
    let watches = observers.iter().map(|declared| {
        let method = &declared.method;
        let identifier = method.to_string();
        let awaited = declared.awaited.then(|| quote!(.await));
        let captured = quote_spanned!(method.span()=>
            #sdk::plugin::captured(<Self>::#method(ctx)#awaited)
        );

        quote!(#identifier => #captured,)
    });
    let publications = observers.iter().map(|observer| &observer.streams)
        .chain(::core::iter::once(streams))
        .map(|catalog| quote! {
            for entry in <#catalog as #sdk::catalog::Catalog<#sdk::catalog::Publications<__DokimePlugin>>>::ENTRIES {
                if stream.as_str() == entry.id {
                    return (entry.publish)(host, token, rows).await;
                }
            }
        });
    let published = ::core::iter::once(streams)
        .chain(observers.iter().map(|observer| &observer.streams))
        .collect::<Vec<_>>();
    let declarations = observers.iter().map(|observer| &observer.declaration);
    let cancellation = graceful_stop.map(|parameter| {
        quote! {
            fn cancellation_is_completion(
                parameters: &::std::collections::BTreeMap<
                    ::std::string::String,
                    #sdk::core::Value,
                >,
            ) -> bool {
                matches!(
                    parameters.get(#parameter),
                    ::core::option::Option::Some(#sdk::core::Value::Bool(true))
                )
            }
        }
    });

    let Grant {
        own, parameters, ..
    } = grant;
    let template = &block.self_ty;
    let cleanup = declared(block, "cleanup").map(|method| {
        let captured = quote_spanned!(method.sig.ident.span()=>
            #sdk::plugin::captured(<Self>::cleanup(self, ctx).await)
        );

        quote! {
            fn cleanup<'__dokime>(
                &'__dokime mut self,
                ctx: &'__dokime mut #sdk::execution::CleanupContext<'_, Self, Self::Own, Self::Parameters>,
            ) -> #sdk::execution::StepFuture<'__dokime> {
                ::std::boxed::Box::pin(async move { #captured })
            }
        }
    });
    let stated = declared(block, "traceability").map(|method| {
        let captured = quote_spanned!(method.sig.ident.span()=>
            #sdk::plugin::captured(<Self>::traceability(self, ctx).await)
        );

        quote! {
            fn traceability<'__dokime>(
                &'__dokime self,
                ctx: &'__dokime #sdk::execution::TraceabilityContext<'_, Self, Self::Own, Self::Parameters>,
            ) -> #sdk::execution::TraceabilityFuture<'__dokime> {
                ::std::boxed::Box::pin(async move { #captured })
            }
        }
    });
    // A template that declared its inputs is handed them; one that named none
    // has nothing to be handed, so its `initialize` takes only the grant.
    let initialize = declared(block, "initialize");
    let arguments = match initialize.is_some_and(|method| method.sig.inputs.len() == 2) {
        true => quote!(ctx.parameters(), ctx),
        false => quote!(ctx),
    };
    let initialized =
        initialize.map_or_else(|| block.self_ty.span(), |method| method.sig.ident.span());
    let initialized = quote_spanned!(initialized=>
        #sdk::plugin::captured(<Self>::initialize(#arguments).await)
            .map(#sdk::__answer::Initialized::<Self>::into_parts)
    );

    Ok(quote! {
        #[automatically_derived]
        impl #sdk::execution::SidecarTemplate for #template {
            type Own = #own;
            type Parameters = #parameters;

            const RUN: &'static str = #run;
            const SIGNAL: &'static str = #signal;
            const REQUIRED: &'static [#sdk::catalog::Node<#sdk::__harmos::operations::WireDeclaration>] =
                <#required as #sdk::catalog::InertCatalog<
                    #sdk::__harmos::operations::Contracts,
                    #sdk::__harmos::operations::WireDeclaration,
                >>::ENTRIES;
            const STREAMS: &'static [#sdk::catalog::Node<#sdk::catalog::Publication>] = &[
                #(#sdk::catalog::Node::Nested(
                    &<#published as #sdk::catalog::Catalog<#sdk::catalog::Streams>>::ENTRIES
                )),*
            ];

            fn definition() -> #sdk::execution::TemplateDefinition {
                #sdk::execution::TemplateDefinition {
                    template: <Self as #sdk::definition::DeclaredTemplate>::declaration(),
                    records: <Self as #sdk::definition::DeclaredTemplate>::record_contracts(),
                    streams: <Self as #sdk::definition::DeclaredTemplate>::stream_contracts(),
                }
            }

            fn initialize<'__dokime>(
                ctx: &'__dokime mut #sdk::execution::InitializationContext<'_, Self, Self::Own, Self::Parameters>,
            ) -> #sdk::execution::InitializationFuture<'__dokime, (Self, Self::Own)> {
                ::std::boxed::Box::pin(async move { #initialized })
            }

            fn declared_observers() -> ::std::vec::Vec<#sdk::execution::DeclaredObserver> {
                ::std::vec![#(#declarations),*]
            }

            fn observe<'__dokime>(
                observer: &'__dokime str,
                ctx: &'__dokime mut #sdk::execution::ObserverContext<'_, Self, Self::Own, Self::Parameters>,
            ) -> #sdk::execution::StepFuture<'__dokime> {
                let _ = ctx;
                ::std::boxed::Box::pin(async move {
                    match observer {
                        #(#watches)*
                        // The engine only ever names observers read off this
                        // template's own declaration, which is this same list.
                        _ => ::core::result::Result::Ok(()),
                    }
                })
            }

            fn publish_sample<'a, __DokimePlugin: #sdk::__harmos::plugin::Declared>(
                host: &'a #sdk::__harmos::sidecar::Supervisor,
                token: &'a str,
                stream: &'a #sdk::core::Identifier,
                rows: &'a [#sdk::execution::StampedRow],
            ) -> #sdk::transport::TransportFuture<'a, ::core::result::Result<(), ::std::string::String>> {
                ::std::boxed::Box::pin(async move {
                    #(#publications)*
                    Err(format!("template has no native publisher for stream {stream}"))
                })
            }

            #cancellation
            #stated
            #cleanup

            fn run_step<'__dokime>(
                &'__dokime mut self,
                step: &'__dokime str,
                ctx: &'__dokime mut #sdk::execution::StepContext<'_, Self, Self::Own, Self::Parameters>,
            ) -> #sdk::execution::StepFuture<'__dokime> {
                ::std::boxed::Box::pin(async move {
                    match step {
                        #(#phases)*
                        // The engine only ever names steps read off this
                        // template's own declaration, which is this same list.
                        _ => ::core::result::Result::Ok(()),
                    }
                })
            }

            fn evaluate<'__dokime>(
                criterion: &'__dokime str,
                ctx: &'__dokime #sdk::execution::CriterionContext<'_, Self, Self::Parameters>,
            ) -> #sdk::execution::EvaluationFuture<'__dokime, bool> {
                ::std::boxed::Box::pin(async move {
                    match criterion {
                        #(#decisions)*
                        _ => ::core::result::Result::Ok(::core::option::Option::None),
                    }
                })
            }
        }

        #[automatically_derived]
        impl #sdk::catalog::InertCatalog<
            #sdk::__harmos::operations::Contracts,
            #sdk::__harmos::operations::WireDeclaration,
        > for #template {
            const ENTRIES: &'static [#sdk::catalog::Node<#sdk::__harmos::operations::WireDeclaration>] =
                <Self as #sdk::execution::SidecarTemplate>::OPERATIONS;
        }
    })
}

fn table(block: &ItemImpl, steps: &[DeclaredStep]) -> Result<Option<TokenStream>, Error> {
    let sdk = crate::facade::root();
    let Some(first) = steps.first() else {
        return Ok(None);
    };
    let function = declared(block, &first.method.to_string())
        .expect("a declared step came from a method on this block");
    let (context, error) = signature(function)?;
    let entries = steps.iter().map(|step| {
        let declaration = &step.declaration;
        let method = &step.method;

        quote!(#sdk::definition::StepBinding::with_step(#declaration, Self::#method))
    });

    Ok(Some(quote! {
                                pub fn steps() -> ::std::vec::Vec<
            #sdk::definition::StepBinding<Self, #context, #error>,
        > {
            ::std::vec![#(#entries),*]
        }
    }))
}

fn described(
    declaration: TokenStream,
    written: Option<&Expr>,
    attributes: &[syn::Attribute],
) -> TokenStream {
    match arguments::description(written, attributes) {
        None => declaration,
        Some(text) => quote!({
            let mut declared = #declaration;
            declared.description =
                ::core::option::Option::Some(::std::string::String::from(#text));
            declared
        }),
    }
}

fn signature(function: &ImplItemFn) -> Result<(Type, Type), Error> {
    let inputs = function.sig.inputs.iter().collect::<Vec<_>>();
    const SHAPE: &str = "a step must be `fn(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error>` \
        (or `async fn` for one that awaits something): a step takes `&mut self` and one `&mut Context`";
    let context = match inputs.as_slice() {
        [FnArg::Receiver(_), FnArg::Typed(context)] => match context.ty.as_ref() {
            Type::Reference(reference) => reference.elem.as_ref().clone(),
            other => return Err(Error::new(other.span(), SHAPE)),
        },
        _ => return Err(Error::new(function.sig.inputs.span(), SHAPE)),
    };
    let ReturnType::Type(_, reply) = &function.sig.output else {
        return Err(Error::new(
            function.sig.output.span(),
            "a step must answer `Result<(), Error>`",
        ));
    };

    Ok((context, answer(reply, Part::Err)))
}

fn strip(block: &mut ItemImpl) {
    for item in &mut block.items {
        if let ImplItem::Fn(function) = item {
            function.attrs.retain(|attribute| {
                !["step", "criterion", "observer"]
                    .iter()
                    .any(|name| marks(attribute.meta.path(), name))
            });
        }
    }
}

/* === Tests === */
