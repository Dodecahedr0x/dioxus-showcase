use dioxus_showcase::prelude::*;

#[provider]
fn NoChildrenProvider() -> Element {
    rsx! { div {} }
}

fn main() {}
