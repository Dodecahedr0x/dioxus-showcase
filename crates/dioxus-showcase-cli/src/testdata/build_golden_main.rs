// Generated once by dioxus-showcase. Safe to edit; never regenerated.
use dioxus::prelude::*;
use showcase_entry as _; // LOAD-BEARING: forces rlib linkage so registrations survive

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        dioxus_showcase_ui::ShowcaseApp { base_path: "/" }
    }
}
