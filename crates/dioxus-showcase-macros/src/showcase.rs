use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::FnArg;

use crate::utils::{
    is_single_props_argument, parse_showcase_meta, render_controlled_story_component,
    slugify_title, story_arg_bindings,
};

/// Expands `#[showcase]` into generated renderer, factory, and story constructor helpers.
pub fn expand(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let showcase_meta = match parse_showcase_meta(attr) {
        Ok(meta) => meta,
        Err(err) => {
            let err_str = err;
            return quote! {
                compile_error!(#err_str);
            };
        }
    };

    let item_ts = item;
    let item_fn: syn::ItemFn = match syn::parse2(item_ts.clone()) {
        Ok(func) => func,
        Err(err) => {
            let err_str = format!("{:?}", err);
            return quote! {
                compile_error!(#err_str)
            };
        }
    };

    let signature = item_fn.sig;
    let component_name = signature.ident.clone();
    let story_title = showcase_meta.title.unwrap_or_else(|| component_name.to_string());
    let story_id = slugify_title(&story_title);
    let story_tags = showcase_meta.tags;

    let renderer_name = format_ident!("__dioxus_showcase_render__{}", component_name);
    let story_factory_name = format_ident!("__dioxus_showcase_factory__{}", component_name);
    let story_symbol_name = format_ident!("__dioxus_showcase_story__{}", component_name);
    let controls_component_name = format_ident!("__dioxus_showcase_controls__{}", component_name);
    let tags_literal = story_tags.iter().map(|tag| quote! { #tag.to_owned() });

    let (controls_component, renderer_body, generated_stories) = if signature.inputs.is_empty() {
        (
            quote! {},
            quote! { ::dioxus::prelude::rsx! { #component_name {} } },
            quote! {
                vec![::dioxus_showcase::GeneratedStory {
                    definition: ::dioxus_showcase::core::StoryDefinition {
                        id: #story_id.to_owned(),
                        title: #story_title.to_owned(),
                        source_path: source_path.to_owned(),
                        module_path: module_path.to_owned(),
                        renderer_symbol: stringify!(#renderer_name).to_owned(),
                        tags: vec![#(#tags_literal),*],
                    },
                    render: ::std::boxed::Box::new(|| #renderer_name()),
                }]
            },
        )
    } else if is_single_props_argument(&signature.inputs) {
        let props_type = match &signature.inputs[0] {
            FnArg::Typed(typed) => typed.ty.clone(),
            _ => unreachable!("single props argument should always be typed"),
        };
        (
            quote! {},
            quote! {
                {
                    let props: #props_type =
                        <#props_type as ::dioxus_showcase::StoryProps>::stories()
                            .into_iter()
                            .next()
                            .map(|story| story.value)
                            .expect("showcase props stories must not be empty");
                    ::dioxus::prelude::rsx! { #component_name { ..props } }
                }
            },
            quote! {
                <#props_type as ::dioxus_showcase::StoryProps>::stories()
                    .into_iter()
                    .map(|story_variant| {
                        let variant_name = story_variant.name;
                        let variant_props = story_variant.value;
                        let title = match variant_name.as_deref() {
                            Some(name) if !name.is_empty() => format!("{}/{}", #story_title, name),
                            _ => #story_title.to_owned(),
                        };
                        let props = variant_props.clone();
                        ::dioxus_showcase::GeneratedStory {
                            definition: ::dioxus_showcase::core::StoryDefinition {
                                id: ::dioxus_showcase::slugify_title(&title),
                                title,
                                source_path: source_path.to_owned(),
                                module_path: module_path.to_owned(),
                                renderer_symbol: stringify!(#renderer_name).to_owned(),
                                tags: vec![#(#tags_literal),*],
                            },
                            render: ::std::boxed::Box::new(move || {
                                let props = props.clone();
                                ::dioxus::prelude::rsx! { #component_name { ..props } }
                            }),
                        }
                    })
                    .collect()
            },
        )
    } else {
        let story_args = match story_arg_bindings(&signature.inputs) {
            Ok(tokens) => tokens,
            Err(err) => {
                return quote! {
                    compile_error!(#err)
                };
            }
        };
        let preview_props = story_args.component_props.clone();
        (
            render_controlled_story_component(
                &controls_component_name,
                story_args,
                quote! { #component_name { #(#preview_props)* } },
            ),
            quote! { ::dioxus::prelude::rsx! { #controls_component_name {} } },
            quote! {
                vec![::dioxus_showcase::GeneratedStory {
                    definition: ::dioxus_showcase::core::StoryDefinition {
                        id: #story_id.to_owned(),
                        title: #story_title.to_owned(),
                        source_path: source_path.to_owned(),
                        module_path: module_path.to_owned(),
                        renderer_symbol: stringify!(#renderer_name).to_owned(),
                        tags: vec![#(#tags_literal),*],
                    },
                    render: ::std::boxed::Box::new(|| #renderer_name()),
                }]
            },
        )
    };

    quote! {
        #[warn(non_camel_case_types)]
        #item_ts
        #controls_component

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub struct #story_factory_name;

        #[doc(hidden)]
        #[allow(non_snake_case)]
        pub fn #renderer_name() -> ::dioxus::prelude::Element {
            #renderer_body
        }

        #[doc(hidden)]
        impl ::dioxus_showcase::ShowcaseStoryFactory for #story_factory_name {
            fn create(
                source_path: &str,
                module_path: &str,
            ) -> ::std::vec::Vec<::dioxus_showcase::GeneratedStory> {
                #generated_stories
            }
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        pub fn #story_symbol_name(
            source_path: &str,
            module_path: &str,
        ) -> ::std::vec::Vec<::dioxus_showcase::GeneratedStory> {
            <#story_factory_name as ::dioxus_showcase::ShowcaseStoryFactory>::create(
                source_path,
                module_path,
            )
        }
    }
}
