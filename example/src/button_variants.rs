use dioxus::prelude::*;
use dioxus_showcase::prelude::*;

#[provider(index = 0)]
#[component]
pub fn ExampleStoryShell(children: Element) -> Element {
    rsx! {
        div {
            style: "padding: 24px; background: #f8fafc; border-radius: 18px;",
            {children}
        }
    }
}

/// Interactive component discovered via `#[showcase]`.
#[showcase(tags = ["examples", "workspace"])]
#[component]
pub fn PillButtonControllable(label: String, disabled: bool) -> Element {
    let border = if disabled { "#cbd5e1" } else { "#0f766e" };
    let background = if disabled { "#ffffff" } else { "#0f766e" };
    let color = if disabled { "#0f172a" } else { "#ffffff" };
    let opacity = if disabled { "0.55" } else { "1" };
    let cursor = if disabled { "not-allowed" } else { "pointer" };
    let label = if label.is_empty() { "Action".to_string() } else { label };

    rsx! {
        button {
            disabled: disabled,
            style: "
                border-radius: 999px;
                border: 1px solid {border};
                background: {background};
                color: {color};
                padding: 8px 14px;
                font: 600 14px/1.2 ui-sans-serif, -apple-system, Segoe UI, sans-serif;
                opacity: {opacity};
                cursor: {cursor};
            ",
            "{label}"
        }
    }
}

/// Story state defined as a plain function with an optional control parameter.
#[story(title = "PillButtonControllable/Primary", tags = ["examples", "workspace"])]
pub fn pill_button_primary(label: String) -> Element {
    let label = if label.is_empty() { "Save Changes".to_string() } else { label };

    rsx! {
        PillButtonControllable { label, disabled: false }
    }
}

/// Another fixed story state defined without wrapping it as a component.
#[story(title = "PillButtonControllable/Disabled", tags = ["examples", "workspace"])]
pub fn pill_button_disabled() -> Element {
    rsx! {
        PillButtonControllable { label: "Unavailable".to_string(), disabled: true }
    }
}
