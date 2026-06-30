//! Selected-work section from the v4 design: GitHub stats strip, language
//! filter, and the project grid, fed by the build-time embedded repos.json.

use i18nrs::yew::use_translation;
use portfolio_data::{CONFIG, Repo, lang_color};
use yew::prelude::*;

use crate::github::ReposState;

use super::reveal::Reveal;
use super::section_header::SectionHeader;
use super::sections::{section_id, section_label};

/// Cards shown per slide window: 2 rows of the 3-column grid.
const PAGE_SIZE: usize = 6;

#[derive(Properties, PartialEq)]
pub struct ProjProps {
    pub state: ReposState,
}

#[function_component(Projects)]
pub fn projects(p: &ProjProps) -> Html {
    let (i18n, _) = use_translation();
    // "Favorites" is the special, default filter: the config-pinned repos are
    // shown first, before the rest can be reached through the language filters.
    let filter = use_state(|| "Favorites".to_string());
    let page = use_state(|| 0usize);

    let offline = matches!(p.state, ReposState::Failed);

    // Non-fork, non-archived repos sorted by stars (v4 behavior).
    let mut repos: Vec<Repo> = match &p.state {
        ReposState::Ready(file) => file
            .repos
            .iter()
            .filter(|r| !r.fork && !r.archived)
            .cloned()
            .collect(),
        _ => Vec::new(),
    };
    repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));

    let mut langs: Vec<String> = repos.iter().filter_map(|r| r.language.clone()).collect();
    langs.sort();
    langs.dedup();

    let active = filter.as_str().to_string();
    let filtered: Vec<&Repo> = if active == "Favorites" {
        let favs: Vec<&Repo> = repos.iter().filter(|r| r.is_featured()).collect();
        // Gracefully fall back to every repo when nothing is pinned (or while
        // the data is still loading), so the section is never empty.
        if favs.is_empty() {
            repos.iter().collect()
        } else {
            favs
        }
    } else {
        repos
            .iter()
            .filter(|r| active == "All" || r.language.as_deref() == Some(active.as_str()))
            .collect()
    };

    let pages = filtered.len().div_ceil(PAGE_SIZE).max(1);
    let cur = (*page).min(pages - 1);

    let count_line = {
        let unit = if filtered.len() == 1 {
            i18n.t("projects.repoOne")
        } else {
            i18n.t("projects.repoMany")
        };
        let mut line = format!("{} {unit}", filtered.len());
        if offline {
            line.push_str(&format!(" · {}", i18n.t("projects.offline")));
        }
        line
    };

    // Helper that builds a filter chip and resets the slide window to page 0.
    let make_filter = {
        let filter = filter.clone();
        let page = page.clone();
        move |value: String, label: String, is_active: bool| {
            let onclick = {
                let filter = filter.clone();
                let page = page.clone();
                let value = value.clone();
                Callback::from(move |_| {
                    filter.set(value.clone());
                    page.set(0);
                })
            };
            html! {
                <button class={classes!("filter-chip", is_active.then_some("active"))} {onclick}>
                    <span class="mono">{label}</span>
                </button>
            }
        }
    };

    let on_prev = {
        let page = page.clone();
        Callback::from(move |_| {
            let c = *page;
            if c > 0 {
                page.set(c - 1);
            }
        })
    };
    let on_next = {
        let page = page.clone();
        Callback::from(move |_| {
            let c = *page;
            if c + 1 < pages {
                page.set(c + 1);
            }
        })
    };

    html! {
        <section id={ section_id("work") } class="sec">
            <Reveal>
                <SectionHeader num={ section_label("work") }
                    title={ i18n.t("projects.title") }
                    intro={ i18n.t("projects.intro") } />
            </Reveal>

            <div class="grid-12">
                <div class="col-label" />
                <div class="col-body">
                    <Reveal delay={120}>
                        <div class="work-filter">
                            <span class="mono text-muted">{ i18n.t("common.filter") }</span>
                            { make_filter("Favorites".to_string(), i18n.t("projects.favorites"), active == "Favorites") }
                            { make_filter("All".to_string(), i18n.t("common.all"), active == "All") }
                            { for langs.iter().map(|l| make_filter(l.clone(), l.clone(), active == *l)) }
                            <span class="flex-1" />
                            <span class="mono text-muted">{count_line}</span>
                        </div>
                    </Reveal>

                    <div class="project-slider">
                        <div class="project-track"
                             style={format!("transform: translateX(-{}%);", cur * 100)}>
                            { for filtered.chunks(PAGE_SIZE).enumerate().map(|(pi, chunk)| html!{
                                <div class="project-grid">
                                    { for chunk.iter().enumerate().map(|(i, r)| html!{
                                        <Reveal delay={(180 + i * 60).min(600) as u32}>
                                            <ProjectCard repo={(*r).clone()} index={pi * PAGE_SIZE + i} />
                                        </Reveal>
                                    })}
                                </div>
                            })}
                        </div>
                    </div>

                    if pages > 1 {
                        <div class="slider-nav">
                            <button class="slider-btn" disabled={cur == 0} onclick={on_prev}
                                    aria-label={ i18n.t("projects.prevPage") }>{"←"}</button>
                            <div class="slider-dots">
                                { for (0..pages).map(|pi| {
                                    let onclick = {
                                        let page = page.clone();
                                        Callback::from(move |_| page.set(pi))
                                    };
                                    html!{
                                        <button class={classes!("slider-dot", (pi == cur).then_some("active"))}
                                                aria-label={format!("{}", pi + 1)} {onclick} />
                                    }
                                })}
                            </div>
                            <button class="slider-btn" disabled={cur + 1 >= pages} onclick={on_next}
                                    aria-label={ i18n.t("projects.nextPage") }>{"→"}</button>
                        </div>
                    }

                    <div class="view-all-wrap">
                        <a href={CONFIG.github} target="_blank" rel="noopener noreferrer" class="view-all-link">
                            <span class="mono text-accent">{ format!("{} →", i18n.t("projects.viewAll")) }</span>
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct StatCellProps {
    label: AttrValue,
    value: AttrValue,
    #[prop_or(false)]
    small: bool,
}

#[function_component(StatCell)]
fn stat_cell(p: &StatCellProps) -> Html {
    html! {
        <div class="stat-cell">
            <span class="mono text-muted">{ p.label.clone() }</span>
            <div class="stat-value"
                 style={ p.small.then_some("font-size: clamp(15px, 1.2vw, 18px)") }>
                { p.value.clone() }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct CardProps {
    repo: Repo,
    index: usize,
}

#[function_component(ProjectCard)]
fn project_card(p: &CardProps) -> Html {
    let (i18n, _) = use_translation();
    let r = &p.repo;
    let lang_label = r.language.clone().unwrap_or_else(|| "—".to_string());
    let color = lang_color(&lang_label);
    let featured = r.is_featured();

    html! {
        <a href={r.html_url.clone()} target="_blank" rel="noopener noreferrer"
           class={classes!("project-card", featured.then_some("featured"))}>
            <div class="project-card-head">
                <div class="project-name">{r.name.clone()}</div>
                <span class="mono text-muted">{"↗"}</span>
            </div>
            <div class="project-card-body">
                <div class="project-desc">{ r.description.clone().unwrap_or_else(|| "—".into()) }</div>
                if !r.topics.is_empty() {
                    <div class="project-tags">
                        { for r.topics.iter().take(4).map(|t| html!{
                            <span class="project-tag">{t.clone()}</span>
                        })}
                    </div>
                }
            </div>
            <div class="project-card-foot">
                <div class="flex items-center gap-1.5">
                    <span class="lang-dot" style={format!("background: {color}")} />
                    <span class="mono text-fg">{lang_label.clone()}</span>
                </div>
                <span class="mono text-muted">{ format!("★ {}", r.stargazers_count) }</span>
                <span class="mono text-muted">{ format!("⑂ {}", r.forks_count) }</span>
            </div>
        </a>
    }
}
