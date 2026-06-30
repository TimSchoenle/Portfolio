//! Command palette (⌘K) from the v4 design.

use i18nrs::yew::use_translation;
use portfolio_data::{CONFIG, Repo, resume_file};
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::i18n::other_language;
use crate::router::Route;

use super::masthead::{SECTIONS, goto_section};
use super::sections::{section_id, section_num};

#[derive(Clone, PartialEq)]
enum Action {
    /// Holds the section slug; the DOM id is derived via [`section_id`].
    Section(&'static str),
    Goto(Route),
    Open(String),
    CopyEmail,
    ToggleLang,
}

#[derive(Clone, PartialEq)]
struct Entry {
    group: String,
    label: String,
    hint: String,
    action: Action,
}

#[derive(Properties, PartialEq)]
pub struct PaletteProps {
    pub repos: Vec<Repo>,
    pub on_close: Callback<()>,
}

#[function_component(CommandPalette)]
pub fn palette(p: &PaletteProps) -> Html {
    let (i18n, set_language) = use_translation();
    let navigator = use_navigator().expect("palette rendered inside router");
    let route = use_route::<Route>().unwrap_or(Route::Home);

    let q = use_state(String::new);
    let selected = use_state(|| 0_usize);

    let g_nav = i18n.t("palette.groupNav");
    let g_work = i18n.t("palette.groupWork");
    let g_act = i18n.t("palette.groupActions");
    let lang = i18n.get_current_language().to_string();

    let mut entries: Vec<Entry> = SECTIONS
        .iter()
        .map(|(slug, key)| Entry {
            group: g_nav.clone(),
            label: i18n.t(key),
            hint: format!("§ {}", section_num(slug)),
            action: Action::Section(slug),
        })
        .collect();
    entries.push(Entry {
        group: g_nav.clone(),
        label: i18n.t("palette.imprint"),
        hint: "/imprint".into(),
        action: Action::Goto(Route::Imprint),
    });
    entries.push(Entry {
        group: g_nav.clone(),
        label: i18n.t("palette.privacy"),
        hint: "/privacy".into(),
        action: Action::Goto(Route::Privacy),
    });

    for r in p.repos.iter().filter(|r| !r.fork && !r.archived) {
        entries.push(Entry {
            group: g_work.clone(),
            label: r.name.clone(),
            hint: r.language.clone().unwrap_or_else(|| "—".into()),
            action: Action::Open(r.html_url.clone()),
        });
    }

    let email_hint = CONFIG.email.split('@').next().unwrap_or("").to_string() + "@…";
    entries.extend([
        Entry {
            group: g_act.clone(),
            label: i18n.t("palette.copyEmail"),
            hint: email_hint,
            action: Action::CopyEmail,
        },
        Entry {
            group: g_act.clone(),
            label: i18n.t("palette.toggleLang"),
            hint: other_language(&lang).to_uppercase(),
            action: Action::ToggleLang,
        },
        Entry {
            group: g_act.clone(),
            label: i18n.t("palette.openGithub"),
            hint: "↗".into(),
            action: Action::Open(CONFIG.github.into()),
        },
        Entry {
            group: g_act.clone(),
            label: i18n.t("palette.openLinkedin"),
            hint: "↗".into(),
            action: Action::Open(CONFIG.linkedin.into()),
        },
        Entry {
            group: g_act,
            label: i18n.t("palette.downloadResume"),
            hint: "PDF".into(),
            action: Action::Open(format!("/resume/{}", resume_file(&lang))),
        },
    ]);

    let filtered: Vec<Entry> = if q.is_empty() {
        entries
    } else {
        let needle = q.to_lowercase();
        entries
            .into_iter()
            .filter(|e| {
                e.label.to_lowercase().contains(&needle) || e.group.to_lowercase().contains(&needle)
            })
            .collect()
    };

    let activate = {
        let navigator = navigator.clone();
        let set_language = set_language.clone();
        let on_close = p.on_close.clone();
        Callback::from(move |action: Action| {
            match &action {
                Action::Section(slug) => goto_section(&navigator, route, section_id(slug)),
                Action::Goto(target) => navigator.push(target),
                Action::Open(url) => {
                    if let Some(win) = web_sys::window() {
                        let _ = win.open_with_url_and_target(url, "_blank");
                    }
                }
                Action::CopyEmail => {
                    wasm_bindgen_futures::spawn_local(async {
                        if let Some(win) = web_sys::window() {
                            let _ = wasm_bindgen_futures::JsFuture::from(
                                win.navigator().clipboard().write_text(CONFIG.email),
                            )
                            .await;
                        }
                    });
                }
                Action::ToggleLang => {
                    set_language.emit(other_language(&lang).to_string());
                }
            }
            if !matches!(action, Action::ToggleLang) {
                on_close.emit(());
            }
        })
    };

    let on_input = {
        let q = q.clone();
        let selected = selected.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target().unwrap().unchecked_into();
            q.set(input.value());
            selected.set(0);
        })
    };

    let on_key = {
        let selected = selected.clone();
        let filtered = filtered.clone();
        let activate = activate.clone();
        Callback::from(move |e: KeyboardEvent| {
            let n = filtered.len().max(1);
            match e.key().as_str() {
                "ArrowDown" => {
                    e.prevent_default();
                    selected.set((*selected + 1) % n);
                }
                "ArrowUp" => {
                    e.prevent_default();
                    selected.set((*selected + n - 1) % n);
                }
                "Enter" => {
                    e.prevent_default();
                    if let Some(entry) = filtered.get(*selected) {
                        activate.emit(entry.action.clone());
                    }
                }
                _ => {}
            }
        })
    };

    let backdrop_close = {
        let on_close = p.on_close.clone();
        Callback::from(move |_: MouseEvent| on_close.emit(()))
    };

    // Group entries preserving first-seen order, keeping the flat index so the
    // highlight matches keyboard-navigation order.
    let mut grouped: Vec<(String, Vec<(usize, Entry)>)> = Vec::new();
    for (flat_i, e) in filtered.iter().enumerate() {
        if let Some(slot) = grouped.iter_mut().find(|(g, _)| *g == e.group) {
            slot.1.push((flat_i, e.clone()));
        } else {
            grouped.push((e.group.clone(), vec![(flat_i, e.clone())]));
        }
    }

    html! {
        <div class="cmdk-overlay" onclick={backdrop_close}>
            <div class="cmdk-modal" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                <div class="cmdk-search">
                    <span class="mono text-accent">{">_"}</span>
                    <input autofocus=true
                        oninput={on_input}
                        onkeydown={on_key}
                        value={(*q).clone()}
                        placeholder={ i18n.t("palette.placeholder") } />
                    <span class="mono text-muted">{"ESC"}</span>
                </div>
                <div class="cmdk-list">
                    { for grouped.iter().map(|(g, items)| html! {
                        <div class="cmdk-group">
                            <span class="mono text-muted">{g.clone()}</span>
                            { for items.iter().map(|(i, e)| {
                                let i = *i;
                                let is_sel = i == *selected;
                                let action = e.action.clone();
                                let activate = activate.clone();
                                let click = Callback::from(move |_| activate.emit(action.clone()));
                                let hover = {
                                    let selected = selected.clone();
                                    Callback::from(move |_| selected.set(i))
                                };
                                html! {
                                    <button onclick={click} onmouseenter={hover}
                                        class={classes!("cmdk-item", is_sel.then_some("active"))}>
                                        <span>{e.label.clone()}</span>
                                        <span class="mono text-muted">{e.hint.clone()}</span>
                                    </button>
                                }
                            })}
                        </div>
                    })}
                    if filtered.is_empty() {
                        <div class="cmdk-empty">
                            <span class="mono text-muted">{ i18n.t("palette.noMatches") }</span>
                        </div>
                    }
                </div>
                <div class="cmdk-footer">
                    <span class="mono text-muted">{ i18n.t("palette.hint") }</span>
                </div>
            </div>
        </div>
    }
}
