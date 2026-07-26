use dioxus_showcase::prelude::*;

#[provider(order = "first")]
fn Theme(children: Element) -> Element {
    rsx! { div { {children} } }
}

fn main() {}
