use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::FnArg;

use crate::utils::{parse_showcase_meta, provider_registration};

/// Expands `#[provider]` into a wrapper function used by the generated showcase shell.
pub fn expand(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let provider_meta = match parse_showcase_meta(attr) {
        Ok(meta) => meta,
        Err(err) => {
            return quote! {
                compile_error!(#err);
            };
        }
    };
    if provider_meta.title.is_some()
        || provider_meta.component.is_some()
        || provider_meta.name.is_some()
        || !provider_meta.tags.is_empty()
    {
        return quote! {
            compile_error!("provider attributes only support order = <integer>");
        };
    }

    let item_ts = item;
    let item_fn: syn::ItemFn = match syn::parse2(item_ts.clone()) {
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

    let signature = item_fn.sig;
    let component_name = signature.ident.clone();
    let wrap_name = format_ident!("__dioxus_showcase_wrap__{}", component_name);
    // Ascending; the lowest order wraps outermost. Defaults to 0.
    let order = provider_meta.order.unwrap_or(0);

    let mut child_prop = None;
    let mut provider_props = Vec::new();
    let mut provider_bindings = Vec::new();

    for input in &signature.inputs {
        let FnArg::Typed(typed) = input else {
            return quote! {
                compile_error!("provider functions must not take a receiver argument");
            };
        };
        let syn::Pat::Ident(pattern) = typed.pat.as_ref() else {
            return quote! {
                compile_error!("provider function parameters must use simple identifier names");
            };
        };

        let ident = &pattern.ident;
        let ty = &typed.ty;
        if ident == "children" {
            if child_prop.is_some() {
                return quote! {
                    compile_error!("provider components may only declare one `children` parameter");
                };
            }
            child_prop = Some(quote! { { child } });
            let _ = ty;
            continue;
        }

        provider_bindings.push(quote! {
            let #ident: #ty = <#ty as ::dioxus_showcase::StoryArg>::story_arg();
        });
        provider_props.push(quote! {
            #ident: #ident,
        });
    }

    let Some(children_prop) = child_prop else {
        return quote! {
            compile_error!("provider components must declare a `children` parameter explicitly");
        };
    };

    let registration = provider_registration(&component_name, &wrap_name, order);

    quote! {
        #item_ts

        #[doc(hidden)]
        #[allow(non_snake_case)]
        pub fn #wrap_name(child: ::dioxus::prelude::Element) -> ::dioxus::prelude::Element {
            #(#provider_bindings)*
            ::dioxus::prelude::rsx! {
                #component_name {
                    #(#provider_props)*
                    #children_prop
                }
            }
        }

        #registration
    }
}

#[cfg(test)]
mod tests {
    use super::expand;
    use quote::quote;

    /// Renders an expansion to text so error strings can be asserted on.
    fn rendered(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> String {
        expand(attr, item).to_string()
    }

    /// Asserts an expansion is a `compile_error!` carrying `needle`.
    fn assert_rejected(
        attr: proc_macro2::TokenStream,
        item: proc_macro2::TokenStream,
        needle: &str,
    ) {
        let text = rendered(attr, item);
        assert!(text.contains("compile_error !"), "expected a rejection, got: {text}");
        assert!(text.contains(needle), "expected {needle:?} in: {text}");
    }

    fn valid_provider() -> proc_macro2::TokenStream {
        quote! {
            fn Theme(children: Element) -> Element {
                rsx! { div { {children} } }
            }
        }
    }

    #[test]
    fn missing_children_parameter_is_rejected() {
        assert_rejected(
            quote! {},
            quote! { fn Theme() -> Element { rsx! { div {} } } },
            "provider components must declare a `children` parameter explicitly",
        );
    }

    #[test]
    fn duplicate_children_parameters_are_rejected() {
        assert_rejected(
            quote! {},
            quote! { fn Theme(children: Element, children: Element) -> Element { rsx! {} } },
            "provider components may only declare one `children` parameter",
        );
    }

    #[test]
    fn receiver_arguments_are_rejected() {
        assert_rejected(
            quote! {},
            quote! { fn Theme(&self, children: Element) -> Element { rsx! {} } },
            "provider functions must not take a receiver argument",
        );
    }

    #[test]
    fn destructuring_parameters_are_rejected() {
        assert_rejected(
            quote! {},
            quote! { fn Theme((a, b): (u8, u8), children: Element) -> Element { rsx! {} } },
            "provider function parameters must use simple identifier names",
        );
    }

    #[test]
    fn story_only_attributes_are_rejected() {
        for attr in [
            quote! { title = "Atoms/Theme" },
            quote! { component = Theme },
            quote! { name = "theme" },
            quote! { tags = ["atoms"] },
        ] {
            assert_rejected(
                attr,
                valid_provider(),
                "provider attributes only support order = <integer>",
            );
        }
    }

    #[test]
    fn the_removed_index_spelling_is_rejected() {
        // The 0.0.x spelling. It must not silently keep working, or the rename
        // never actually happens for anyone.
        let text = rendered(quote! { index = 2 }, valid_provider());

        assert!(text.contains("compile_error !"), "got {text}");
        assert!(text.contains("`index` was renamed to `order`"), "got {text}");
        assert!(!text.contains("ProviderRegistration"), "got {text}");
    }

    #[test]
    fn non_function_items_are_rejected_with_a_display_formatted_error() {
        let text = rendered(quote! {}, quote! { struct NotAFunction; });

        // `Display` gives `expected `fn``; the `Debug` this used to use gives
        // `Error("expected `fn`")`, which is an unstable internal dump.
        assert_eq!(text, "compile_error ! (\"expected `fn`\")");
    }

    #[test]
    fn a_valid_provider_registers_itself_with_the_default_order() {
        let text = rendered(quote! {}, valid_provider());

        assert!(!text.contains("compile_error !"), "got {text}");
        assert!(text.contains(":: dioxus_showcase :: ProviderRegistration"), "got {text}");
        assert!(text.contains("order : 0i32"), "got {text}");
        assert!(text.contains("wrap : __dioxus_showcase_wrap__Theme"), "got {text}");
    }

    #[test]
    fn an_explicit_negative_order_reaches_the_registration() {
        let text = rendered(quote! { order = -10 }, valid_provider());

        assert!(text.contains("order : - 10i32"), "got {text}");
    }
}
