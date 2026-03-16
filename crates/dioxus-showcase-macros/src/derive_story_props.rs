use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Expands `#[derive(StoryProps)]` into `StoryArg` and `StoryProps` impls backed by `Default`.
pub fn expand(input: TokenStream2) -> TokenStream2 {
    let item: syn::DeriveInput = match syn::parse2(input) {
        Ok(item) => item,
        Err(_err) => {
            return quote! {
                compile_error!("StoryProps derive failed to parse input")
            };
        }
    };
    let type_name = item.ident;

    quote! {
        impl ::dioxus_showcase::StoryArg for #type_name
        where #type_name: ::core::default::Default {
            fn story_arg() -> Self {
                <Self as ::core::default::Default>::default()
            }
        }

        impl ::dioxus_showcase::StoryProps for #type_name
        where #type_name: ::core::default::Default {
            fn stories() -> ::std::vec::Vec<::dioxus_showcase::StoryVariant<Self>> {
                ::std::vec![::dioxus_showcase::StoryVariant::unnamed(
                    <Self as ::core::default::Default>::default()
                )]
            }
        }
    }
}
