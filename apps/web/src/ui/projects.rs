//! Selected-work section: GitHub filter chips + the paged project grid, fed by
//! the build-time embedded `repos.json`.

use std::fmt::Write as _;
use std::rc::Rc;

use dioxus::prelude::*;
use portfolio_data::{CONFIG, Repo, lang_color};

use crate::embedded::repos::ReposState;
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

/// The repositories worth showing, most-starred first, and the languages found
/// among them — derived once rather than on every render of the section.
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

/// Renders the section from `state` and keeps the chosen chip and page in its own signals.
///
/// Opens on [`Filter::Favorites`], and falls back to the full list when nothing featured
/// survived the build's fetch, so the section is never empty on its own first view. A
/// [`ReposState::Failed`] `state` says so in the status line and renders no cards: the embedded
/// listing did not parse, so there is nothing to fall back to.
#[component]
pub fn Projects(state: ReposState) -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let filter = use_signal(|| Filter::Favorites);
    let page = use_signal(|| 0usize);

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
    let filtered = shown(repos, &current);
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
            let _ = write!(line, " · {}", t("projects.offline"));
        }
        line
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
                            FilterChip { value: Filter::Favorites, label: t("projects.favorites"), filter, page }
                            FilterChip { value: Filter::All, label: t("common.all"), filter, page }
                            {langs.iter().map(|l| rsx! {
                                FilterChip { key: "{l}", value: Filter::Language(l.clone()), label: l.clone(), filter, page }
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
                                        Reveal { key: "{r.name}", delay: u32::try_from((180 + i * 60).min(600)).unwrap_or(600),
                                            ProjectCard { repo: (*r).clone() }
                                        }
                                    })}
                                }
                            })}
                        }
                    }

                    if pages > 1 {
                        Pager { pages, cur, page }
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

/// The repositories `filter` selects, borrowed from `repos` in their order.
///
/// [`Filter::Favorites`] falls back to every repository when none of the featured ones survived
/// the fetch, so the section never opens empty.
fn shown<'a>(repos: &'a [Repo], filter: &Filter) -> Vec<&'a Repo> {
    match filter {
        Filter::Favorites => {
            let favs: Vec<&Repo> = repos.iter().filter(|r| r.is_featured()).collect();
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
    }
}

/// One chip of the single-choice filter group. Choosing it sets `filter` to `value` and returns
/// to the first page.
#[component]
fn FilterChip(
    value: Filter,
    label: String,
    filter: Signal<Filter>,
    page: Signal<usize>,
) -> Element {
    let is_active = filter() == value;
    let cls = if is_active {
        "filter-chip active"
    } else {
        "filter-chip"
    };
    rsx! {
        button {
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
}

/// The previous and next buttons and one dot per page. `cur` is the page shown, out of `pages`.
#[component]
fn Pager(pages: usize, cur: usize, page: Signal<usize>) -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    rsx! {
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
                    let page_label = t("projects.pageLabel")
                        .replace("{n}", &(pi + 1).to_string())
                        .replace("{total}", &pages.to_string());
                    rsx! {
                        button {
                            key: "{pi}",
                            class: "{cls}",
                            "aria-label": "{page_label}",
                            "aria-current": if pi == cur { "true" } else { "false" },
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
}

/// One repository as a card linking to it on GitHub.
#[component]
fn ProjectCard(repo: Repo) -> Element {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A repository named `name` that the configuration does not feature.
    fn plain(name: &str, language: Option<&str>, stars: u32) -> Repo {
        let repo = Repo {
            name: name.to_owned(),
            full_name: format!("owner/{name}"),
            description: None,
            html_url: format!("https://github.com/owner/{name}"),
            language: language.map(str::to_owned),
            stargazers_count: stars,
            forks_count: 0,
            updated_at: String::new(),
            topics: Vec::new(),
            fork: false,
            archived: false,
            homepage: None,
        };
        assert!(
            !repo.is_featured(),
            "{name} is featured in the configuration"
        );
        repo
    }

    /// The first repository the configuration features.
    fn featured() -> Repo {
        let name = CONFIG
            .featured_repos
            .first()
            .expect("the configuration features a repository");
        Repo {
            name: (*name).to_owned(),
            ..plain("unfeatured-placeholder", Some("Rust"), 1)
        }
    }

    /// A parsed listing holding `repos`.
    fn listing(repos: Vec<Repo>) -> ReposState {
        ReposState::Ready(std::sync::Arc::new(portfolio_data::ReposFile {
            generated_at: String::new(),
            user: "owner".to_owned(),
            repos,
        }))
    }

    fn names(repos: &[&Repo]) -> Vec<String> {
        repos.iter().map(|r| r.name.clone()).collect()
    }

    #[test]
    fn drops_forks_and_archived_repositories_and_sorts_by_stars() {
        let mut fork = plain("fork", Some("Go"), 99);
        fork.fork = true;
        let mut archived = plain("archived", Some("Go"), 98);
        archived.archived = true;
        let state = listing(vec![
            plain("low", Some("Rust"), 1),
            fork,
            archived,
            plain("high", Some("Go"), 10),
            plain("none", None, 5),
        ]);

        let (repos, langs) = base_lists(&state);
        assert_eq!(
            repos.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            ["high", "none", "low"]
        );
        assert_eq!(langs, ["Go", "Rust"]);
    }

    #[test]
    fn a_failed_listing_has_no_repositories_and_no_languages() {
        let (repos, langs) = base_lists(&ReposState::Failed);
        assert_eq!(repos, Vec::<Repo>::new());
        assert_eq!(langs, Vec::<String>::new());
    }

    #[test]
    fn favorites_shows_the_featured_repositories_only() {
        let repos = [plain("a", Some("Go"), 3), featured(), plain("b", None, 1)];
        assert_eq!(names(&shown(&repos, &Filter::Favorites)), [featured().name]);
    }

    #[test]
    fn favorites_falls_back_to_every_repository_when_none_is_featured() {
        let repos = [plain("a", Some("Go"), 3), plain("b", None, 1)];
        assert_eq!(names(&shown(&repos, &Filter::Favorites)), ["a", "b"]);
    }

    #[test]
    fn a_language_filter_keeps_that_language_only() {
        let repos = [
            plain("a", Some("Go"), 3),
            plain("b", Some("Rust"), 2),
            plain("c", None, 1),
        ];
        assert_eq!(names(&shown(&repos, &Filter::All)), ["a", "b", "c"]);
        assert_eq!(
            names(&shown(&repos, &Filter::Language("Rust".into()))),
            ["b"]
        );
        assert_eq!(
            names(&shown(&repos, &Filter::Language("All".into()))),
            Vec::<String>::new()
        );
    }

    #[cfg(feature = "server")]
    mod rendered {
        use super::*;
        use crate::ui::test_render::{render, text};

        fn seven_unfeatured() -> ReposState {
            listing(
                (0..7)
                    .map(|i| plain(&format!("repo-{i}"), Some("Rust"), i))
                    .collect(),
            )
        }

        #[test]
        fn opens_on_the_featured_repositories() {
            let html = render("en", || {
                rsx! {
                    Projects { state: listing(vec![plain("plain", Some("Go"), 9), featured()]) }
                }
            });
            assert_eq!(html.matches("project-card featured").count(), 1);
            assert!(!html.contains("owner/plain"));
            assert!(html.contains(&format!("1 {}", text("en", "projects.repoOne"))));
            // One chip per pseudo-filter and per language, Favorites pressed.
            assert_eq!(html.matches(r#"aria-pressed="true""#).count(), 1);
            assert_eq!(html.matches("aria-pressed=").count(), 2 + 2);
        }

        #[test]
        fn pages_more_than_six_repositories() {
            let html = render("en", || rsx! { Projects { state: seven_unfeatured() } });
            assert_eq!(html.matches(r#"class="project-card""#).count(), 7);
            assert_eq!(html.matches(r#"class="project-grid""#).count(), 2);
            assert!(html.contains(r#"class="slider-nav""#));
            assert_eq!(html.matches("aria-current=").count(), 2);
            assert_eq!(html.matches(r#"aria-current="true""#).count(), 1);
            assert!(html.contains(&format!("7 {}", text("en", "projects.repoMany"))));
            let first_page = text("en", "projects.pageLabel")
                .replace("{n}", "1")
                .replace("{total}", "2");
            assert!(html.contains(&format!(r#"aria-label="{first_page}""#)));
        }

        #[test]
        fn six_repositories_fit_one_page_without_a_pager() {
            let html = render("en", || {
                rsx! {
                    Projects {
                        state: listing((0..6).map(|i| plain(&format!("r{i}"), None, i)).collect()),
                    }
                }
            });
            assert_eq!(html.matches(r#"class="project-grid""#).count(), 1);
            assert!(!html.contains("slider-nav"));
        }

        #[test]
        fn a_failed_listing_says_so_and_shows_no_card() {
            let html = render("de", || rsx! { Projects { state: ReposState::Failed } });
            assert!(html.contains(&text("de", "projects.offline")));
            assert!(!html.contains("project-card"));
        }

        #[test]
        fn a_card_shows_its_topics_and_counts() {
            let html = render("en", || {
                let mut repo = plain("tagged", Some("Rust"), 4);
                repo.forks_count = 2;
                repo.topics = ["a", "b", "c", "d", "e"].map(String::from).to_vec();
                rsx! { ProjectCard { repo } }
            });
            assert_eq!(html.matches(r#"class="project-tag""#).count(), 4);
            assert!(html.contains("★ 4"));
            assert!(html.contains("⑂ 2"));
            // No description renders as a dash rather than an empty line.
            assert!(html.contains(r#"<div class="project-desc">—</div>"#));
        }
    }
}
