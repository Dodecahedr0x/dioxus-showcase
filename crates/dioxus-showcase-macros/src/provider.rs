use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::FnArg;

use crate::utils::parse_showcase_meta;

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
            compile_error!("provider attributes only support index = <integer>");
        };
    }

    let item_ts = item;
    let item_fn: syn::ItemFn = match syn::parse2(item_ts.clone()) {
        Ok(func) => func,
        Err(err) => {
            let err_str = format!("{err:?}");
            return quote! {
                compile_error!(#err_str)
            };
        }
    };

    let signature = item_fn.sig;
    let component_name = signature.ident.clone();
    let wrap_name = format_ident!("__dioxus_showcase_wrap__{}", component_name);
    let index = provider_meta.index.unwrap_or(0);

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

    quote! {
        #item_ts

        #[doc(hidden)]
        #[allow(non_snake_case)]
        pub fn #wrap_name(child: ::dioxus::prelude::Element) -> ::dioxus::prelude::Element {
            let _provider_index: i32 = #index;
            #(#provider_bindings)*
            ::dioxus::prelude::rsx! {
                #component_name {
                    #(#provider_props)*
                    #children_prop
                }
            }
        }
    }
}
