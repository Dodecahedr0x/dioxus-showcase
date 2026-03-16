use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse::Parser, ExprPath, FnArg, Meta, Type};

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
        if let Some(control) = render_story_control(ident, ty) {
            has_controls = true;
            state_bindings.push(quote! {
                let mut #ident = ::dioxus::prelude::use_signal(|| <#ty as ::dioxus_showcase::StoryArg>::story_arg());
            });
            render_args.push(quote! { #ident() });
            component_props.push(quote! { #ident: #ident(), });
            controls.push(control);
        } else {
            state_bindings.push(quote! {
                let #ident: #ty = <#ty as ::dioxus_showcase::StoryArg>::story_arg();
            });
            render_args.push(quote! { #ident });
            component_props.push(quote! { #ident: #ident, });
        }
    }

    Ok(StoryArgBindings { state_bindings, render_args, component_props, controls, has_controls })
}

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

fn is_type_ident(ty: &Type, expected: &str) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    type_path.qself.is_none()
        && type_path.path.segments.last().is_some_and(|segment| segment.ident == expected)
}

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
    pub index: Option<i32>,
}

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

        if named.path.is_ident("index") {
            let syn::Expr::Lit(expr_lit) = named.value else {
                return Err("showcase index must be an integer literal".to_owned());
            };
            let syn::Lit::Int(lit) = expr_lit.lit else {
                return Err("showcase index must be an integer literal".to_owned());
            };
            out.index = Some(
                lit.base10_parse::<i32>()
                    .map_err(|_| "showcase index must fit in i32".to_owned())?,
            );
        }
    }

    Ok(out)
}

fn component_name_from_path(expr_path: &ExprPath) -> Result<String, String> {
    expr_path
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
        .ok_or_else(|| "showcase component path must not be empty".to_owned())
}

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
