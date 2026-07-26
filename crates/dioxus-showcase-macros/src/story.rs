use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::utils::{
    parse_showcase_meta, render_controlled_story_component, render_story_frame, slugify_title,
    story_arg_bindings, story_registration, strip_param_attrs,
};

/// Expands `#[story]` into generated renderer, factory, and story constructor helpers.
pub fn expand(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let story_meta = match parse_showcase_meta(attr) {
        Ok(meta) => meta,
        Err(err) => {
            return quote! {
                compile_error!(#err);
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

    let registration = story_registration(&story_name, &story_symbol_name);

    quote! {
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

        #registration
    }
}

/// Resolves the final story title, preserving the current title-only public API.
fn derive_story_title(
    story_meta: &crate::utils::ShowcaseMeta,
    fallback_story_name: &str,
) -> Result<String, String> {
    if story_meta.component.is_some() {
        return Err("#[story(...)] no longer accepts component = ...; use title = \"...\" instead"
            .to_owned());
    }

    if story_meta.name.is_some() {
        return Err(
            "#[story(...)] no longer accepts name = ...; include the full story path in title = \"...\""
                .to_owned(),
        );
    }

    if let Some(title) = &story_meta.title {
        return Ok(title.clone());
    }

    Ok(fallback_story_name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::expand;
    use quote::quote;

    #[test]
    fn zero_arg_story_renders_inside_story_frame() {
        let expanded = expand(
            quote! { title = "Atoms/Button/Default" },
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

#[cfg(test)]
mod error_tests {
    use super::expand;
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;

    fn rendered(attr: TokenStream2, item: TokenStream2) -> String {
        expand(attr, item).to_string()
    }

    fn assert_rejected(attr: TokenStream2, item: TokenStream2, needle: &str) {
        let text = rendered(attr, item);
        assert!(text.contains("compile_error !"), "expected a rejection, got: {text}");
        assert!(text.contains(needle), "expected {needle:?} in: {text}");
    }

    fn zero_arg_story() -> TokenStream2 {
        quote! { fn button_default() -> Element { rsx! { button {} } } }
    }

    #[test]
    fn unparseable_attribute_arguments_are_rejected() {
        assert_rejected(quote! { title = }, zero_arg_story(), "invalid #[showcase(...)] arguments");
    }

    #[test]
    fn the_removed_component_argument_is_rejected_with_a_replacement() {
        assert_rejected(
            quote! { component = Button },
            zero_arg_story(),
            "no longer accepts component = ...; use title",
        );
    }

    #[test]
    fn the_removed_name_argument_is_rejected_with_a_replacement() {
        assert_rejected(
            quote! { name = "Default" },
            zero_arg_story(),
            "no longer accepts name = ...; include the full story path in title",
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
            quote! { fn button_default(&self, label: String) -> Element { rsx! {} } },
            "showcase functions must not take a receiver argument",
        );
    }

    #[test]
    fn destructuring_parameters_are_rejected() {
        assert_rejected(
            quote! {},
            quote! { fn button_default((a, b): (u8, u8)) -> Element { rsx! {} } },
            "showcase function parameters must use simple identifier names",
        );
    }

    #[test]
    fn a_zero_arg_story_registers_itself_at_its_call_site() {
        let text = rendered(quote! { title = "Atoms/Button/Default" }, zero_arg_story());

        assert!(!text.contains("compile_error !"), "got {text}");
        assert!(text.contains(":: dioxus_showcase :: ShowcaseRegistration"), "got {text}");
        assert!(text.contains("source_path : file ! ()"), "got {text}");
        assert!(
            text.contains("concat ! (module_path ! () , \"::\" , stringify ! (button_default))"),
            "got {text}"
        );
        assert!(text.contains("factory : __dioxus_showcase_story__button_default"), "got {text}");
    }

    #[test]
    fn a_controlled_story_registers_itself_too() {
        let text = rendered(
            quote! { title = "Atoms/Button/Controlled" },
            quote! { fn button_controlled(label: String) -> Element { rsx! {} } },
        );

        assert!(!text.contains("compile_error !"), "got {text}");
        assert!(text.contains(":: dioxus_showcase :: ShowcaseRegistration"), "got {text}");
    }
}
