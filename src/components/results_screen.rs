use crate::api::SearchResult;
use dioxus::prelude::*;
use std::path::Path;

#[derive(Props, Clone, PartialEq)]
pub struct ResultsScreenProps {
    pub query: String,
    pub search_time: f64,
    pub results: Vec<SearchResult>,
    pub on_open_file: EventHandler<String>,
    pub on_search: Signal<String>,
}

#[component]
pub fn ResultsScreen(props: ResultsScreenProps) -> Element {
    // Deduplicate by file path (keeping the best-scoring chunk) and order by
    // score, so "Most relevant" is genuinely the highest and the list descends.
    let (best, related) = {
        let mut by_path: std::collections::HashMap<String, SearchResult> =
            std::collections::HashMap::new();
        for r in &props.results {
            by_path
                .entry(r.file_path.clone())
                .and_modify(|e| {
                    if r.score > e.score {
                        *e = r.clone();
                    }
                })
                .or_insert_with(|| r.clone());
        }
        let mut unique: Vec<SearchResult> = by_path.into_values().collect();
        unique.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let best = unique.first().cloned();
        let related: Vec<SearchResult> = unique.into_iter().skip(1).collect();
        (best, related)
    };
    let max_score = best.as_ref().map(|b| b.score).unwrap_or(1.0);

    rsx! {
        div { class: "results-screen",
            super::search_bar::SearchBar {
                on_search: props.on_search,
                placeholder: props.query.clone(),
            }
            div { class: "results-time", "{props.search_time:.2}s" }

            if let Some(best) = best.as_ref() {
                div { class: "primary-result",
                    div { class: "primary-result-heading",
                        div { class: "primary-result-label", "Most relevant file" }
                        div { class: "primary-result-score", "{score_fmt(best.score, max_score)}%" }
                    }
                    div {
                        class: "primary-result-path",
                        onclick: {
                            let path = best.file_path.clone();
                            let h = props.on_open_file.clone();
                            move |_| h.call(path.clone())
                        },
                        "{best.file_path}"
                    }
                }
            }

            if !related.is_empty() {
                hr { class: "section-divider" }
                div { class: "related-heading", "RELATED \u{2014} {related.len()}" }

                for r in related.iter() {
                    div {
                        class: "result-row",
                        onclick: {
                            let path = r.file_path.clone();
                            let h = props.on_open_file.clone();
                            move |_| h.call(path.clone())
                        },
                        div { class: "result-row-name", "{file_name(&r.file_path)}" }
                        div { class: "result-row-score", "{score_fmt(r.score, max_score)}%" }
                    }
                }
            }
        }
    }
}

/// Display score as a percentage *relative to the best hit* (best = 100%).
fn score_fmt(score: f64, max_score: f64) -> String {
    if max_score <= 0.0 {
        return "0".to_string();
    }
    let pct = (score / max_score) * 100.0;
    format!("{:.0}", pct.clamp(1.0, 100.0))
}

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}
