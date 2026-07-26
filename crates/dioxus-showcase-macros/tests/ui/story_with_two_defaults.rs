use dioxus_showcase::prelude::*;

#[story(title = "Bad/TwoDefaults")]
fn two_defaults(#[default = 1.0] #[default = 2.0] size: f64) -> Element {
    rsx! { "{size}" }
}

fn main() {}
