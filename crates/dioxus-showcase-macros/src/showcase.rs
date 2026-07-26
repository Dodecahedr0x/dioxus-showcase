use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::FnArg;

use crate::utils::{
    is_single_props_argument, parse_showcase_meta, render_controlled_story_component,
    slugify_title, story_arg_bindings, story_registration, strip_param_attrs,
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

    let mut item_fn: syn::ItemFn = match syn::parse2::<syn::ItemFn>(item) {
        Ok(func) => func,
        Err(err) => {
            // `Display`, not `Debug`: the `Debug` form of a `syn::Error` is an
            // unstable internal dump rather than something a user can act on.
            let err_str = err.to_string();
            return quote! {
                compile_error!(#err_str)
            };
        }
    };

    // Clone before stripping: the bindings below read `#[default = …]` off the
    // parameters, and the re-emitted function must not carry it.
    let signature = item_fn.sig.clone();
    strip_param_attrs(&mut item_fn);
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

    let registration = story_registration(&component_name, &story_symbol_name);

    quote! {
        #[warn(non_camel_case_types)]
        #item_fn
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

        #registration
    }
}

#[cfg(test)]
mod tests {
    use super::{expand, TokenStream2};
    use quote::quote;

    fn rendered(attr: TokenStream2, item: TokenStream2) -> String {
        expand(attr, item).to_string()
    }

    fn assert_rejected(attr: TokenStream2, item: TokenStream2, needle: &str) {
        let text = rendered(attr, item);
        assert!(text.contains("compile_error !"), "expected a rejection, got: {text}");
        assert!(text.contains(needle), "expected {needle:?} in: {text}");
    }

    fn zero_arg_component() -> TokenStream2 {
        quote! { fn Button() -> Element { rsx! { button {} } } }
    }

    #[test]
    fn unparseable_attribute_arguments_are_rejected() {
        assert_rejected(
            quote! { title = },
            zero_arg_component(),
            "invalid #[showcase(...)] arguments",
        );
    }

    #[test]
    fn non_literal_title_is_rejected() {
        assert_rejected(
            quote! { title = 12 },
            zero_arg_component(),
            "showcase title must be a string literal",
        );
    }

    #[test]
    fn non_array_tags_are_rejected() {
        assert_rejected(
            quote! { tags = "atoms" },
            zero_arg_component(),
            "showcase tags must be an array of string literals",
        );
    }

    #[test]
    fn non_function_items_are_rejected_with_a_display_formatted_error() {
        // `Display`, not the `Debug` this used to use, which rendered as
        // `Error("expected `fn`")`.
        assert_eq!(
            rendered(quote! {}, quote! { struct NotAFunction; }),
            "compile_error ! (\"expected `fn`\")"
        );
    }

    #[test]
    fn receiver_arguments_are_rejected() {
        assert_rejected(
            quote! {},
            quote! { fn Button(&self, label: String) -> Element { rsx! {} } },
            "showcase functions must not take a receiver argument",
        );
    }

    #[test]
    fn destructuring_parameters_are_rejected() {
        assert_rejected(
            quote! {},
            quote! { fn Button((a, b): (u8, u8)) -> Element { rsx! {} } },
            "showcase function parameters must use simple identifier names",
        );
    }

    #[test]
    fn a_zero_arg_component_registers_itself_at_its_call_site() {
        let text = rendered(quote! { title = "Atoms/Button" }, zero_arg_component());

        assert!(!text.contains("compile_error !"), "got {text}");
        assert!(text.contains(":: dioxus_showcase :: ShowcaseRegistration"), "got {text}");
        assert!(text.contains("source_path : file ! ()"), "got {text}");
        assert!(
            text.contains("concat ! (module_path ! () , \"::\" , stringify ! (Button))"),
            "got {text}"
        );
        assert!(text.contains("factory : __dioxus_showcase_story__Button"), "got {text}");
        // A23: the generated symbol names are unchanged.
        assert!(text.contains("__dioxus_showcase_render__Button"), "got {text}");
        assert!(text.contains("__dioxus_showcase_factory__Button"), "got {text}");
    }

    #[test]
    fn a_props_component_registers_itself_too() {
        let text =
            rendered(quote! {}, quote! { fn Button(props: ButtonProps) -> Element { rsx! {} } });

        assert!(!text.contains("compile_error !"), "got {text}");
        assert!(text.contains(":: dioxus_showcase :: ShowcaseRegistration"), "got {text}");
    }

    #[test]
    fn a_multi_arg_component_registers_itself_too() {
        let text = rendered(
            quote! {},
            quote! { fn Button(label: String, disabled: bool) -> Element { rsx! {} } },
        );

        assert!(!text.contains("compile_error !"), "got {text}");
        assert!(text.contains(":: dioxus_showcase :: ShowcaseRegistration"), "got {text}");
    }
}
