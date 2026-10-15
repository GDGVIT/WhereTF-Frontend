use crate::api;
use dioxus::prelude::*;
use std::time::Duration;

/// Width of the spotlight window (kept constant; only height changes).
const SPOTLIGHT_WIDTH: u32 = 1230;
const MAX_HEIGHT: u32 = 460;

/// Floating, always-on-top desktop search bar (Spotlight-style).
/// A pill-shaped input with live results as you type. Own window, resized to
/// fit its content (no transparency needed).
#[component]
pub fn SpotlightWindow() -> Element {
    let query = use_signal(String::new);
    let results = use_signal(|| Vec::<api::SearchResult>::new());
    let window = dioxus_desktop::use_window();

    // Size the window to the actual rendered content height (measured in the DOM
    // after layout) so there are no empty gaps regardless of platform scaling.
    let window_resize = window.clone();
    use_effect(move || {
        // Re-run whenever the content changes.
        let _ = query.read().len();
        let _ = results.read().len();
        let wr = window_resize.clone();
        spawn(async move {
            // Let the DOM paint before measuring.
            tokio::time::sleep(Duration::from_millis(60)).await;
            let js = "(() => { const el = document.querySelector('.sw-content'); \
                      if (!el) return 0; \
                      return Math.ceil(el.getBoundingClientRect().height) + 24; })()";
            if let Ok(v) = dioxus::document::eval(js).await {
                if let Some(h) = v.as_u64() {
                    let h = (h as u32).clamp(72, MAX_HEIGHT);
                    let _ = wr.set_inner_size(
                        dioxus_desktop::tao::dpi::PhysicalSize::new(SPOTLIGHT_WIDTH, h),
                    );
                }
            }
        });
    });

    // Live search: runs whenever the query changes (debounced).
    use_effect(move || {
        let q = query.read().clone();
        let mut results = results.clone();
        if q.trim().is_empty() {
            results.set(Vec::new());
            return;
        }
        let qq = q.clone();
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(180)).await;
            if let Ok(resp) = api::search(&qq, "hybrid", 8).await {
                results.set(resp.results);
            }
        });
    });

    rsx! {
        div { class: "sw-root",
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape {
                    window.set_visible(false);
                }
            },
            div { class: "sw-content",
                div { class: "sw-pill",
                    span { class: "sw-icon", "\u{2315}" }
                    input {
                        class: "sw-input",
                        r#type: "text",
                        placeholder: "Search your files\u{2026}",
                        autofocus: true,
                        value: "{query}",
                        oninput: {
                            let mut q = query.clone();
                            move |e| q.set(e.value())
                        },
                    }
                }
                if !query.read().trim().is_empty() {
                    div { class: "sw-results",
                        if results.read().is_empty() {
                            div { class: "sw-empty", "No results" }
                        } else {
                            for r in results.read().iter() {
                                div {
                                    class: "sw-result",
                                    onclick: {
                                        let path = r.file_path.clone();
                                        let window = window.clone();
                                        move |_| {
                                            let _ = open::that(&path);
                                            window.set_visible(false);
                                        }
                                    },
                                    span { class: "sw-result-name", "{r.file_path}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
