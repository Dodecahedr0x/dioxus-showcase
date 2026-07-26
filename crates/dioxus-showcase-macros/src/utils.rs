use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse::Parser, ExprPath, FnArg, Meta, Type};

/// Returns `true` when the function signature is a single aggregate `props` argument.
pub fn is_single_props_argument(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>,
) -> bool {
    if inputs.len() != 1 {
        return false;
    }

    let Some(FnArg::Typed(typed)) = inputs.first() else {
        return false;
    };
    let syn::Pat::Ident(ident) = typed.pat.as_ref() else {
        return false;
    };
    ident.ident == "props"
}

pub struct StoryArgBindings {
    pub state_bindings: Vec<TokenStream2>,
    pub render_args: Vec<TokenStream2>,
    pub component_props: Vec<TokenStream2>,
    pub controls: Vec<TokenStream2>,
    pub has_controls: bool,
}

/// Converts a function signature into bindings, render args, props syntax, and optional controls.
pub fn story_arg_bindings(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>,
) -> Result<StoryArgBindings, String> {
    let mut state_bindings = Vec::new();
    let mut render_args = Vec::new();
    let mut component_props = Vec::new();
    let mut controls = Vec::new();
    let mut has_controls = false;

    for input in inputs {
        let FnArg::Typed(typed) = input else {
            return Err("showcase functions must not take a receiver argument".to_owned());
        };
        let syn::Pat::Ident(pattern) = typed.pat.as_ref() else {
            return Err(
                "showcase function parameters must use simple identifier names when not using a props struct"
                    .to_owned(),
            );
        };

        let ident = &pattern.ident;
        let ty = &typed.ty;
        let initial = param_default(&typed.attrs, ty)?;
        if let Some(control) = render_story_control(ident, ty) {
            has_controls = true;
            state_bindings.push(quote! {
                let mut #ident = ::dioxus::prelude::use_signal(|| #initial);
            });
            render_args.push(quote! { #ident() });
            component_props.push(quote! { #ident: #ident(), });
            controls.push(control);
        } else {
            state_bindings.push(quote! {
                let #ident: #ty = #initial;
            });
            render_args.push(quote! { #ident });
            component_props.push(quote! { #ident: #ident, });
        }
    }

    Ok(StoryArgBindings { state_bindings, render_args, component_props, controls, has_controls })
}

/// The expression a parameter's control opens on.
///
/// `#[default = <expr>]` on the parameter when present, otherwise
/// `StoryArg::story_arg()` — `0`, `false`, `"Lorem Ipsum"`. Without a default a
/// control opens on a value the preview is usually not rendering, because the
/// seed exists to be *a* value rather than a meaningful one: a story that wants
/// a 32px spinner has nowhere to say so, and the number beside it reads `0`.
///
/// String literals are widened with `String::from`, so the common case is
/// `#[default = "currentColor"]` rather than `#[default = "currentColor".to_owned()]`.
/// Every other type takes the expression as written, which keeps integer
/// literals inferring against the parameter's own type instead of going through
/// an ambiguous `Into`.
fn param_default(attrs: &[syn::Attribute], ty: &Type) -> Result<TokenStream2, String> {
    let mut found: Option<syn::Expr> = None;

    for attr in attrs {
        if !attr.path().is_ident("default") {
            continue;
        }
        let Meta::NameValue(named) = &attr.meta else {
            return Err(
                "showcase parameter defaults are written #[default = <expression>]".to_owned()
            );
        };
        if found.is_some() {
            return Err("a showcase parameter takes at most one #[default]".to_owned());
        }
        found = Some(named.value.clone());
    }

    let Some(expr) = found else {
        return Ok(quote! { <#ty as ::dioxus_showcase::StoryArg>::story_arg() });
    };

    if is_type_ident(ty, "String") {
        return Ok(quote! { ::std::string::String::from(#expr) });
    }

    Ok(quote! { #expr })
}

/// Strips `#[default = …]` from a function's parameters.
///
/// The attribute is read by [`story_arg_bindings`] and is not a real attribute
/// anywhere else, so it has to come off before the function is re-emitted —
/// rustc rejects an unknown attribute on a parameter.
pub fn strip_param_attrs(item: &mut syn::ItemFn) {
    for input in &mut item.sig.inputs {
        if let FnArg::Typed(typed) = input {
            typed.attrs.retain(|attr| !attr.path().is_ident("default"));
        }
    }
}

/// Renders a hidden Dioxus component that hosts the preview and any generated controls.
pub fn render_controlled_story_component(
    controls_component_name: &syn::Ident,
    bindings: StoryArgBindings,
    preview: TokenStream2,
) -> TokenStream2 {
    let StoryArgBindings { state_bindings, controls, has_controls, .. } = bindings;
    let has_controls_lit = syn::LitBool::new(has_controls, Span::call_site());
    let framed_preview = render_story_frame(preview);

    quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[::dioxus::prelude::component]
        fn #controls_component_name() -> ::dioxus::prelude::Element {
            #(#state_bindings)*
            ::dioxus::prelude::rsx! {
                #framed_preview
                if #has_controls_lit {
                    div { class: "story-controls",
                        h3 { class: "story-controls-title", "Controls" }
                        div { class: "story-controls-list",
                            #(#controls)*
                        }
                    }
                }
            }
        }
    }
}

/// Wraps preview content in the common showcase frame markup.
pub fn render_story_frame(preview: TokenStream2) -> TokenStream2 {
    quote! {
        div { class: "story-preview",
            div { class: "story-canvas",
                ::dioxus_showcase::StoryPreviewContent {
                    #preview
                }
            }
        }
    }
}

/// Emits a control widget for supported interactive parameter types.
fn render_story_control(ident: &syn::Ident, ty: &Type) -> Option<TokenStream2> {
    let name = ident.to_string();

    if is_type_ident(ty, "String") {
        return Some(quote! {
            label { class: "story-control",
                span { class: "story-control-label", #name }
                input {
                    class: "story-control-input",
                    r#type: "text",
                    value: #ident(),
                    oninput: move |event| #ident.set(event.value()),
                }
            }
        });
    }

    if is_type_ident(ty, "bool") {
        return Some(quote! {
            label { class: "story-control story-control-checkbox",
                input {
                    class: "story-control-checkbox-input",
                    r#type: "checkbox",
                    checked: #ident(),
                    onchange: move |event| #ident.set(event.checked()),
                }
                span { class: "story-control-label", #name }
            }
        });
    }

    if is_numeric_type(ty) {
        return Some(quote! {
            label { class: "story-control",
                span { class: "story-control-label", #name }
                input {
                    class: "story-control-input",
                    r#type: "number",
                    value: #ident().to_string(),
                    oninput: move |event| {
                        if let Ok(next_value) = event.value().parse::<#ty>() {
                            #ident.set(next_value);
                        }
                    },
                }
            }
        });
    }

    None
}

/// Checks whether a type path ends with a specific identifier.
fn is_type_ident(ty: &Type, expected: &str) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    type_path.qself.is_none()
        && type_path.path.segments.last().is_some_and(|segment| segment.ident == expected)
}

/// Returns `true` when the type is one of the supported numeric control types.
fn is_numeric_type(ty: &Type) -> bool {
    [
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64",
    ]
    .iter()
    .any(|name| is_type_ident(ty, name))
}

#[derive(Default)]
pub struct ShowcaseMeta {
    pub title: Option<String>,
    pub component: Option<String>,
    pub name: Option<String>,
    pub tags: Vec<String>,
    pub order: Option<i32>,
}

/// Parses supported macro metadata keys shared by story, showcase, and provider attributes.
pub fn parse_showcase_meta(attr: TokenStream2) -> Result<ShowcaseMeta, String> {
    let metas = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
        .parse2(attr)
        .map_err(|err| format!("invalid #[showcase(...)] arguments: {err}"))?;
    let mut out = ShowcaseMeta::default();

    for meta in metas {
        let Meta::NameValue(named) = meta else {
            continue;
        };

        if named.path.is_ident("title") {
            let syn::Expr::Lit(expr_lit) = named.value else {
                return Err("showcase title must be a string literal".to_owned());
            };
            let syn::Lit::Str(lit) = expr_lit.lit else {
                return Err("showcase title must be a string literal".to_owned());
            };
            out.title = Some(lit.value());
            continue;
        }

        if named.path.is_ident("component") {
            let syn::Expr::Path(expr_path) = named.value else {
                return Err("showcase component must be a component path".to_owned());
            };
            out.component = Some(component_name_from_path(&expr_path)?);
            continue;
        }

        if named.path.is_ident("name") {
            let syn::Expr::Lit(expr_lit) = named.value else {
                return Err("showcase name must be a string literal".to_owned());
            };
            let syn::Lit::Str(lit) = expr_lit.lit else {
                return Err("showcase name must be a string literal".to_owned());
            };
            out.name = Some(lit.value());
            continue;
        }

        if named.path.is_ident("tags") {
            let syn::Expr::Array(array) = named.value else {
                return Err("showcase tags must be an array of string literals".to_owned());
            };
            out.tags = parse_tags_array(&array)?;
            continue;
        }

        if named.path.is_ident("order") {
            out.order = Some(parse_order_value(&named.value)?);
            continue;
        }

        // `index` was the 0.0.x spelling of `order`. Rejecting it by name, with
        // the replacement spelled out, is the only migration aid a user gets
        // besides the CHANGELOG — so it is worth saying more than "unknown key".
        if named.path.is_ident("index") {
            return Err(
                "`index` was renamed to `order` in 0.1.0: write #[provider(order = <integer>)]. \
                 The meaning is unchanged — providers apply in ascending order and the lowest \
                 order wraps outermost."
                    .to_owned(),
            );
        }
    }

    Ok(out)
}

/// Parses an `order = <integer>` value, including negative literals.
///
/// `order = -10` is a unary negation expression rather than a literal, so
/// matching only `Expr::Lit` would silently drop every negative order.
fn parse_order_value(value: &syn::Expr) -> Result<i32, String> {
    let (negative, expr) = match value {
        syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
            (true, unary.expr.as_ref())
        }
        other => (false, other),
    };

    let syn::Expr::Lit(expr_lit) = expr else {
        return Err("showcase order must be an integer literal".to_owned());
    };
    let syn::Lit::Int(lit) = &expr_lit.lit else {
        return Err("showcase order must be an integer literal".to_owned());
    };

    let digits = lit.base10_digits();
    let signed = if negative { format!("-{digits}") } else { digits.to_owned() };
    signed.parse::<i32>().map_err(|_| "showcase order must fit in i32".to_owned())
}

/// Extracts the last path segment from a component path expression.
fn component_name_from_path(expr_path: &ExprPath) -> Result<String, String> {
    expr_path
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
        .ok_or_else(|| "showcase component path must not be empty".to_owned())
}

/// Parses the `tags = ["..."]` array into owned strings.
fn parse_tags_array(array: &syn::ExprArray) -> Result<Vec<String>, String> {
    array
        .elems
        .iter()
        .map(|expr| {
            let syn::Expr::Lit(expr_lit) = expr else {
                return Err("showcase tags must contain string literals only".to_owned());
            };
            let syn::Lit::Str(lit) = &expr_lit.lit else {
                return Err("showcase tags must contain string literals only".to_owned());
            };
            Ok(lit.value())
        })
        .collect()
}

/// Emits the link-time story registration for one annotated item.
///
/// This expands at the **user's** call site, so `file!()` and `module_path!()`
/// bake in the user's source file and module rather than this crate's.
/// `module_path!()` names only the module, hence the `concat!` that appends the
/// item — that reproduces the `krate::module::item` form the CLI already used.
///
/// `factory` must stay a plain `fn` item: `inventory::submit!` stores the value
/// in a `static`, which a closure could not inhabit.
pub fn story_registration(item_name: &syn::Ident, story_symbol_name: &syn::Ident) -> TokenStream2 {
    quote! {
        ::dioxus_showcase::__private::inventory::submit! {
            ::dioxus_showcase::ShowcaseRegistration {
                source_path: file!(),
                module_path: concat!(module_path!(), "::", stringify!(#item_name)),
                factory: #story_symbol_name,
            }
        }
    }
}

/// Emits the link-time provider registration for one annotated component.
pub fn provider_registration(
    component_name: &syn::Ident,
    wrap_name: &syn::Ident,
    order: i32,
) -> TokenStream2 {
    quote! {
        ::dioxus_showcase::__private::inventory::submit! {
            ::dioxus_showcase::ProviderRegistration {
                module_path: concat!(module_path!(), "::", stringify!(#component_name)),
                order: #order,
                wrap: #wrap_name,
            }
        }
    }
}

/// Normalizes a title into the slug format used by generated story ids.
pub fn slugify_title(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut prev_dash = false;

    for ch in title.chars() {
        let normalized = ch.to_ascii_lowercase();
        if normalized.is_ascii_alphanumeric() {
            out.push(normalized);
            prev_dash = false;
            continue;
        }

        if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    out.trim_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::{format_ident, quote};

    /// Returns the error message `parse_showcase_meta` produced, or panics.
    fn meta_error(attr: TokenStream2) -> String {
        parse_showcase_meta(attr).err().expect("expected the attribute to be rejected")
    }

    #[test]
    fn unparseable_attribute_arguments_are_rejected() {
        assert!(meta_error(quote! { title = }).starts_with("invalid #[showcase(...)] arguments:"));
    }

    #[test]
    fn non_literal_title_is_rejected() {
        assert_eq!(
            meta_error(quote! { title = some_ident }),
            "showcase title must be a string literal"
        );
        assert_eq!(meta_error(quote! { title = 12 }), "showcase title must be a string literal");
    }

    #[test]
    fn non_literal_name_is_rejected() {
        assert_eq!(meta_error(quote! { name = 12 }), "showcase name must be a string literal");
    }

    #[test]
    fn non_path_component_is_rejected() {
        assert_eq!(
            meta_error(quote! { component = "Button" }),
            "showcase component must be a component path"
        );
    }

    #[test]
    fn non_array_tags_are_rejected() {
        assert_eq!(
            meta_error(quote! { tags = "atoms" }),
            "showcase tags must be an array of string literals"
        );
    }

    #[test]
    fn non_string_tag_elements_are_rejected() {
        assert_eq!(
            meta_error(quote! { tags = [1, 2] }),
            "showcase tags must contain string literals only"
        );
    }

    #[test]
    fn the_removed_index_spelling_is_rejected_and_names_its_replacement() {
        let message = meta_error(quote! { index = 2 });

        assert!(message.contains("`index` was renamed to `order`"), "got {message}");
        assert!(message.contains("#[provider(order = <integer>)]"), "got {message}");
        // The wrap direction is the part a migrating user cannot guess.
        assert!(message.contains("lowest order wraps outermost"), "got {message}");
    }

    #[test]
    fn index_is_rejected_even_when_order_is_also_present() {
        // Silently preferring `order` here would let a half-migrated attribute
        // compile, which is how the old spelling would survive unnoticed.
        assert!(meta_error(quote! { order = 5, index = 2 }).contains("`index` was renamed"));
    }

    #[test]
    fn non_integer_order_is_rejected() {
        assert_eq!(
            meta_error(quote! { order = "first" }),
            "showcase order must be an integer literal"
        );
    }

    #[test]
    fn out_of_range_order_is_rejected() {
        assert_eq!(meta_error(quote! { order = 99999999999 }), "showcase order must fit in i32");
    }

    #[test]
    fn order_accepts_negative_literals() {
        // `order = -10` is a unary negation expression, not a literal, so this
        // is the case a naive `Expr::Lit` match drops on the floor.
        let meta = parse_showcase_meta(quote! { order = -10 }).expect("negative order is valid");
        assert_eq!(meta.order, Some(-10));
    }

    #[test]
    fn order_accepts_positive_literals_and_defaults_to_absent() {
        assert_eq!(parse_showcase_meta(quote! { order = 7 }).unwrap().order, Some(7));
        assert_eq!(parse_showcase_meta(quote! {}).unwrap().order, None);
    }

    #[test]
    fn title_and_tags_parse_together() {
        let meta = parse_showcase_meta(quote! { title = "Atoms/Button", tags = ["a", "b"] })
            .expect("valid attribute");
        assert_eq!(meta.title.as_deref(), Some("Atoms/Button"));
        assert_eq!(meta.tags, vec!["a".to_owned(), "b".to_owned()]);
    }

    /// Returns the error `story_arg_bindings` produced for a signature.
    fn binding_error(sig: TokenStream2) -> String {
        let item_fn: syn::ItemFn = syn::parse2(sig).expect("test signature should parse");
        story_arg_bindings(&item_fn.sig.inputs).err().expect("expected a rejection")
    }

    #[test]
    fn receiver_arguments_are_rejected() {
        assert_eq!(
            binding_error(quote! { fn demo(&self, label: String) {} }),
            "showcase functions must not take a receiver argument"
        );
    }

    #[test]
    fn destructuring_parameter_patterns_are_rejected() {
        assert!(binding_error(quote! { fn demo((a, b): (u8, u8)) {} })
            .starts_with("showcase function parameters must use simple identifier names"));
    }

    #[test]
    fn story_registration_targets_the_facade_and_the_call_site() {
        let tokens = story_registration(
            &format_ident!("button_default"),
            &format_ident!("__dioxus_showcase_story__button_default"),
        )
        .to_string();

        assert!(tokens.contains(":: dioxus_showcase :: __private :: inventory :: submit"));
        assert!(tokens.contains(":: dioxus_showcase :: ShowcaseRegistration"));
        // `file!()`/`module_path!()` must stay unexpanded here so they resolve
        // at the user's call site, not in this crate.
        assert!(tokens.contains("source_path : file ! ()"));
        assert!(
            tokens.contains("concat ! (module_path ! () , \"::\" , stringify ! (button_default))")
        );
        assert!(tokens.contains("factory : __dioxus_showcase_story__button_default"));
    }

    #[test]
    fn provider_registration_carries_the_order() {
        let tokens = provider_registration(
            &format_ident!("Theme"),
            &format_ident!("__dioxus_showcase_wrap__Theme"),
            -10,
        )
        .to_string();

        assert!(tokens.contains(":: dioxus_showcase :: ProviderRegistration"));
        assert!(tokens.contains("order : - 10i32"), "got {tokens}");
        assert!(tokens.contains("wrap : __dioxus_showcase_wrap__Theme"));
    }
}
