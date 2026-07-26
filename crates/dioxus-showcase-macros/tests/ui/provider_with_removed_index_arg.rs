// The 0.0.x spelling of `#[provider(order = N)]`. This is the exact source a
// user upgrading from 0.0.7 still has, so the diagnostic it produces is their
// migration aid.
use dioxus_showcase::prelude::*;

#[provider(index = 0)]
fn Theme(children: Element) -> Element {
    rsx! { div { class: "theme", {children} } }
}

fn main() {}
