use dioxus_showcase::prelude::*;

#[story(title = "Bad/Default")]
fn bad_default(#[default] size: f64) -> Element {
    rsx! { "{size}" }
}

fn main() {}
