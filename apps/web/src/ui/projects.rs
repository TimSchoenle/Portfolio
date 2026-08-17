//! Selected-work section: GitHub filter chips + the paged project grid, fed by
//! the build-time embedded `repos.json`.

use std::rc::Rc;

use dioxus::prelude::*;
use portfolio_data::{CONFIG, Repo, lang_color};

use crate::github::ReposState;
use crate::i18n::use_i18n;
use crate::sections::{section_id, section_label};
use crate::ui::reveal::Reveal;
use crate::ui::section_header::SectionHeader;

/// Cards shown per slide window: 2 rows of the 3-column grid.
const PAGE_SIZE: usize = 6;

/// The active filter chip.
///
/// Modelled as a type rather than a bare string so the two pseudo-filters cannot
/// collide with a real language: the previous `String` state compared the
/// selection against the sentinels `"Favorites"` and `"All"` *and* against
/// `Repo::language`, so a repository whose language was literally named "All"
/// would have silently taken over that chip.
#[derive(Clone, PartialEq)]
enum Filter {
    /// The hand-picked `CONFIG.featured_repos`.
    Favorites,
    /// Every non-fork, non-archived repository.
    All,
    /// Repositories whose primary language is this one.
    Language(String),
}

impl Filter {
    /// A key that is unique across both pseudo-filters and every language name,
    /// for the rendered chip's `key`.
    fn key(&self) -> String {
        match self {
            Filter::Favorites => "favorites".to_string(),
            Filter::All => "all".to_string(),
            Filter::Language(lang) => format!("lang:{lang}"),
        }
    }
}

/// The repositories worth showing, most-starred first, and the languages found
/// among them — the two derivations every render of this section used to redo.
///
/// Forks and archived repositories are dropped here rather than at each use
/// site, so the chip counts, the pager and the grid all agree on what "a project"
/// is.
fn base_lists(state: &ReposState) -> (Vec<Repo>, Vec<String>) {
    let mut repos: Vec<Repo> = state
        .repos()
        .iter()
        .filter(|r| !r.fork && !r.archived)
        .cloned()
        .collect();
    repos.sort_by_key(|repo| std::cmp::Reverse(repo.stargazers_count));

    let mut langs: Vec<String> = repos.iter().filter_map(|r| r.language.clone()).collect();
    langs.sort();
    langs.dedup();

    (repos, langs)
}

#[component]
pub fn Projects(state: ReposState) -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let mut filter = use_signal(|| Filter::Favorites);
    let mut page = use_signal(|| 0usize);

    let offline = matches!(state, ReposState::Failed);

    // The shown repositories and the languages among them: both are pure
    // functions of the build-time embedded `repos.json`, so they are derived once
    // per mounted section rather than on every render — the same treatment the
    // radar's dot layout gets, and for the same reason. `filter` and `page` below
    // write signals, and before this each chip click and each page step re-cloned
    // the whole repository list three times over, re-sorted it and rebuilt the
    // language list, for data that cannot change while the page is open.
    let base = use_hook(|| Rc::new(base_lists(&state)));
    let (repos, langs) = (&base.0, &base.1);

    let current = filter();
    // Borrowed, not cloned: this is re-derived on every render (it depends on the
    // active chip) and only the handful of cards actually rendered below need an
    // owned `Repo`.
    let filtered: Vec<&Repo> = match &current {
        Filter::Favorites => {
            let favs: Vec<&Repo> = repos.iter().filter(|r| r.is_featured()).collect();
            // Fall back to the full list rather than showing an empty section
            // when none of the featured repositories survived the fetch.
            if favs.is_empty() {
                repos.iter().collect()
            } else {
                favs
            }
        }
        Filter::All => repos.iter().collect(),
        Filter::Language(lang) => repos
            .iter()
            .filter(|r| r.language.as_deref() == Some(lang.as_str()))
            .collect(),
    };

    let pages = filtered.len().div_ceil(PAGE_SIZE).max(1);
    let cur = page().min(pages - 1);

    let count_line = {
        let unit = if filtered.len() == 1 {
            t("projects.repoOne")
        } else {
            t("projects.repoMany")
        };
        let mut line = format!("{} {unit}", filtered.len());
        if offline {
            line.push_str(&format!(" · {}", t("projects.offline")));
        }
        line
    };

    let make_filter = move |value: Filter, label: String, is_active: bool| {
        let cls = if is_active {
            "filter-chip active"
        } else {
            "filter-chip"
        };
        let key = value.key();
        rsx! {
            button {
                key: "{key}",
                class: "{cls}",
                // The chips form a single-choice group, so each one reports
                // whether it is the current selection instead of leaving that
                // information in the styling alone.
                "aria-pressed": "{is_active}",
                onclick: move |_| {
                    filter.set(value.clone());
                    page.set(0);
                },
                span { class: "mono", "{label}" }
            }
        }
    };

    rsx! {
        section { id: section_id("work"), class: "sec",
            Reveal {
                SectionHeader {
                    num: section_label("work"),
                    title: t("projects.title"),
                    intro: t("projects.intro"),
                }
            }

            div { class: "grid-12",
                div { class: "col-label" }
                div { class: "col-body",
                    Reveal { delay: 120,
                        div { class: "work-filter",
                            span { class: "mono text-muted", {t("common.filter")} }
                            {make_filter(Filter::Favorites, t("projects.favorites"), current == Filter::Favorites)}
                            {make_filter(Filter::All, t("common.all"), current == Filter::All)}
                            {langs.iter().map(|l| {
                                let value = Filter::Language(l.clone());
                                let is_active = current == value;
                                make_filter(value, l.clone(), is_active)
                            })}
                            span { class: "flex-1" }
                            span { class: "mono text-muted", "{count_line}" }
                        }
                    }

                    div { class: "project-slider",
                        div { class: "project-track", style: "transform: translateX(-{cur * 100}%);",
                            {filtered.chunks(PAGE_SIZE).enumerate().map(|(pi, chunk)| rsx! {
                                div { key: "p{pi}", class: "project-grid",
                                    {chunk.iter().enumerate().map(|(i, r)| rsx! {
                                        Reveal { key: "{r.name}", delay: (180 + i * 60).min(600) as u32,
                                            ProjectCard { repo: (*r).clone(), index: pi * PAGE_SIZE + i }
                                        }
                                    })}
                                }
                            })}
                        }
                    }

                    if pages > 1 {
                        div { class: "slider-nav",
                            button {
                                class: "slider-btn",
                                disabled: cur == 0,
                                "aria-label": t("projects.prevPage"),
                                onclick: move |_| { let c = page(); if c > 0 { page.set(c - 1); } },
                                "←"
                            }
                            div { class: "slider-dots",
                                {(0..pages).map(|pi| {
                                    let cls = if pi == cur { "slider-dot active" } else { "slider-dot" };
                                    rsx! {
                                        button {
                                            key: "{pi}",
                                            class: "{cls}",
                                            "aria-label": "{pi + 1}",
                                            onclick: move |_| page.set(pi),
                                        }
                                    }
                                })}
                            }
                            button {
                                class: "slider-btn",
                                disabled: cur + 1 >= pages,
                                "aria-label": t("projects.nextPage"),
                                onclick: move |_| { let c = page(); if c + 1 < pages { page.set(c + 1); } },
                                "→"
                            }
                        }
                    }

                    div { class: "view-all-wrap",
                        a {
                            href: CONFIG.github,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "view-all-link",
                            span { class: "mono text-accent", {format!("{} →", t("projects.viewAll"))} }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ProjectCard(repo: Repo, index: usize) -> Element {
    let _ = index;
    let lang_label = repo.language.clone().unwrap_or_else(|| "—".to_string());
    let color = lang_color(&lang_label);
    let card_cls = if repo.is_featured() {
        "project-card featured"
    } else {
        "project-card"
    };
    let desc = repo.description.clone().unwrap_or_else(|| "—".into());
    let stars = repo.stargazers_count;
    let forks = repo.forks_count;
    let has_topics = !repo.topics.is_empty();

    rsx! {
        a {
            href: "{repo.html_url}",
            target: "_blank",
            rel: "noopener noreferrer",
            class: "{card_cls}",
            div { class: "project-card-head",
                div { class: "project-name", "{repo.name}" }
                span { class: "mono text-muted", "↗" }
            }
            div { class: "project-card-body",
                div { class: "project-desc", "{desc}" }
                if has_topics {
                    div { class: "project-tags",
                        {repo.topics.iter().take(4).map(|tag| rsx! {
                            span { key: "{tag}", class: "project-tag", "{tag}" }
                        })}
                    }
                }
            }
            div { class: "project-card-foot",
                div { class: "flex items-center gap-1.5",
                    span { class: "lang-dot", style: "background: {color}" }
                    span { class: "mono text-fg", "{lang_label}" }
                }
                span { class: "mono text-muted", "★ {stars}" }
                span { class: "mono text-muted", "⑂ {forks}" }
            }
        }
    }
}
