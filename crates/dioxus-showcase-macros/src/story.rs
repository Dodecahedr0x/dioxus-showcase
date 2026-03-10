use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::utils::{
    parse_showcase_meta, render_controlled_story_component, render_story_frame, slugify_title,
    story_arg_bindings,
};

pub fn expand(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let story_meta = match parse_showcase_meta(attr) {
        Ok(meta) => meta,
        Err(err) => {
            return quote! {
                compile_error!(#err);
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
    let story_name = signature.ident.clone();
    let story_title = match derive_story_title(&story_meta, &story_name.to_string()) {
        Ok(title) => title,
        Err(err) => {
            return quote! {
                compile_error!(#err);
            };
        }
    };
    let story_id = slugify_title(&story_title);
    let story_tags = story_meta.tags;
    let tags_literal = story_tags.iter().map(|tag| quote! { #tag.to_owned() });

    let renderer_name = format_ident!("__dioxus_showcase_render__{}", story_name);
    let story_factory_name = format_ident!("__dioxus_showcase_factory__{}", story_name);
    let story_symbol_name = format_ident!("__dioxus_showcase_story__{}", story_name);
    let controls_component_name = format_ident!("__dioxus_showcase_controls__{}", story_name);

    let (controls_component, renderer_body) = if signature.inputs.is_empty() {
        let framed_preview = render_story_frame(quote! { { #story_name() } });
        (quote! {}, quote! { ::dioxus::prelude::rsx! { #framed_preview } })
    } else {
        let story_args = match story_arg_bindings(&signature.inputs) {
            Ok(tokens) => tokens,
            Err(err) => {
                return quote! {
                    compile_error!(#err);
                };
            }
        };
        let render_args = story_args.render_args.clone();
        (
            render_controlled_story_component(
                &controls_component_name,
                story_args,
                quote! { { #story_name(#(#render_args),*) } },
            ),
            quote! { ::dioxus::prelude::rsx! { #controls_component_name {} } },
        )
    };

    quote! {
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

#[cfg(test)]
mod tests {
    use super::expand;
    use quote::quote;

    #[test]
    fn zero_arg_story_renders_inside_story_frame() {
        let expanded = expand(
            quote! { component = button_component, name = "Default" },
            quote! {
                fn button_default() -> &'static str {
                    "ok"
                }
            },
        );

        let rendered = expanded.to_string();
        assert!(rendered.contains("story-preview"));
        assert!(rendered.contains("story-canvas"));
    }
}

fn derive_story_title(
    story_meta: &crate::utils::ShowcaseMeta,
    fallback_story_name: &str,
) -> Result<String, String> {
    if let Some(title) = &story_meta.title {
        return Ok(title.clone());
    }

    match (&story_meta.component, &story_meta.name) {
        (Some(component), Some(name)) => Ok(format!("{component}/{name}")),
        (Some(_), None) => {
            Err("#[story(component = ComponentName)] also requires name = \"...\"".to_owned())
        }
        (None, Some(_)) => Err("#[story(name = ...)] also requires component = \"...\"".to_owned()),
        (None, None) => Ok(fallback_story_name.to_owned()),
    }
}
