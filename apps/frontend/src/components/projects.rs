//! Selected-work section from the v4 design: GitHub stats strip, language
//! filter, skeleton cards while repos.json loads, and the project grid.

use i18nrs::yew::use_translation;
use portfolio_data::{CONFIG, Repo, lang_color};
use yew::prelude::*;

use crate::github::ReposState;

use super::reveal::Reveal;
use super::section_header::SectionHeader;

#[derive(Properties, PartialEq)]
pub struct ProjProps {
    pub state: ReposState,
}

#[function_component(Projects)]
pub fn projects(p: &ProjProps) -> Html {
    let (i18n, _) = use_translation();
    let filter = use_state(|| "All".to_string());

    let loading = matches!(p.state, ReposState::Loading);
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
    let filtered: Vec<&Repo> = repos
        .iter()
        .filter(|r| active == "All" || r.language.as_deref() == Some(active.as_str()))
        .collect();

    let total_stars: u32 = repos.iter().map(|r| r.stargazers_count).sum();
    let total_forks: u32 = repos.iter().map(|r| r.forks_count).sum();
    let updated = match &p.state {
        ReposState::Ready(file) => file.generated_at.get(..10).unwrap_or("—").to_string(),
        _ => "—".to_string(),
    };

    let count_line = {
        let unit = if filtered.len() == 1 {
            i18n.t("projects.repoOne")
        } else {
            i18n.t("projects.repoMany")
        };
        let mut line = format!("{} {unit}", filtered.len());
        if loading {
            line.push_str(&format!(" · {}", i18n.t("projects.loading")));
        }
        if offline {
            line.push_str(&format!(" · {}", i18n.t("projects.offline")));
        }
        line
    };

    html! {
        <section id="s3" class="sec">
            <Reveal>
                <SectionHeader num="§ 03 — work"
                    title={ i18n.t("projects.title") }
                    intro={ i18n.t("projects.intro") } />
            </Reveal>

            <div class="grid-12">
                <div class="col-label" />
                <div class="col-body">
                    <Reveal delay={80}>
                        <div class="stats-strip">
                            <StatCell label={i18n.t("projects.statRepos")}
                                value={ if loading { "—".to_string() } else { repos.len().to_string() } } />
                            <StatCell label={i18n.t("projects.statStars")}
                                value={ if loading { "—".to_string() } else { total_stars.to_string() } } />
                            <StatCell label={i18n.t("projects.statForks")}
                                value={ if loading { "—".to_string() } else { total_forks.to_string() } } />
                            <StatCell label={i18n.t("projects.statSync")} value={updated} small=true />
                        </div>
                    </Reveal>

                    <Reveal delay={120}>
                        <div class="work-filter">
                            <span class="mono text-muted">{ i18n.t("common.filter") }</span>
                            <button class={classes!("filter-chip", (active == "All").then_some("active"))}
                                    onclick={{
                                        let filter = filter.clone();
                                        Callback::from(move |_| filter.set("All".to_string()))
                                    }}>
                                <span class="mono">{ i18n.t("common.all") }</span>
                            </button>
                            { for langs.iter().map(|l| {
                                let is_active = active == *l;
                                let onclick = {
                                    let filter = filter.clone();
                                    let l = l.clone();
                                    Callback::from(move |_| filter.set(l.clone()))
                                };
                                html!{
                                    <button class={classes!("filter-chip", is_active.then_some("active"))} {onclick}>
                                        <span class="mono">{l.clone()}</span>
                                    </button>
                                }
                            })}
                            <span class="flex-1" />
                            <span class="mono text-muted">{count_line}</span>
                        </div>
                    </Reveal>

                    if loading {
                        <Reveal delay={180}>
                            <div class="project-grid">
                                { for (0..6).map(|_| html!{ <div class="project-card skeleton" /> }) }
                            </div>
                        </Reveal>
                    } else if filtered.is_empty() {
                        <div class="empty-state">
                            <span class="mono text-muted">{"// awaiting_sync"}</span>
                            <h3 class="mt-3 mb-1.5">{ i18n.t("projects.emptyTitle") }</h3>
                            <p class="text-muted max-w-md mx-auto m-0">{ i18n.t("projects.emptyBody") }</p>
                        </div>
                    } else {
                        <div class="project-grid">
                            { for filtered.iter().enumerate().map(|(i, r)| html!{
                                <Reveal delay={(180 + i * 60).min(600) as u32}>
                                    <ProjectCard repo={(*r).clone()} index={i} />
                                </Reveal>
                            })}
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
                <span class="mono text-accent">
                    { if featured { i18n.t("projects.featured") } else { format!("REPO_{:02}", p.index + 1) } }
                </span>
                <span class="mono text-muted">{"↗"}</span>
            </div>
            <div class="project-card-body">
                <div class="project-name">{r.name.clone()}</div>
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
