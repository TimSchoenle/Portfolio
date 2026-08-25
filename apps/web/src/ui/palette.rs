//! Command palette (⌘K).
//!
//! Search, grouping and keyboard navigation are pure Dioxus state. The actions
//! that need the browser (open external URL, copy email, smooth-scroll to a
//! section) are wired in the hydration phase; navigation and language switching
//! work here.

use dioxus::prelude::*;
use portfolio_data::{CONFIG, resume_file};

use crate::github::ReposState;
use crate::i18n::{other_language, persist_locale, use_i18n};
use crate::routes::Route;
use crate::sections::section_num;
use crate::ui::masthead::SECTIONS;

#[derive(Clone, PartialEq)]
enum Action {
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

/// The ⌘K overlay: one search field over sections, repositories, pages and languages.
///
/// Mounted only while open, so it holds no visibility state of its own and starts each time
/// with an empty query. `on_close` fires on Escape, on a click outside, and after any action
/// that navigates.
#[component]
pub fn CommandPalette(repos: ReposState, on_close: EventHandler<()>) -> Element {
    let ctx = use_i18n();
    let i18n = ctx.i18n;
    let set_language = ctx.set_language;
    let t = move |k: &str| i18n.read().t(k);
    let navigator = use_navigator();
    #[cfg(feature = "web")]
    let on_home = matches!(use_route::<Route>(), Route::Home {});

    let mut q = use_signal(String::new);
    let mut selected = use_signal(|| 0usize);

    let lang = i18n.read().get_current_language().to_string();
    let g_nav = t("palette.groupNav");
    let g_work = t("palette.groupWork");
    let g_act = t("palette.groupActions");

    let mut entries: Vec<Entry> = SECTIONS
        .iter()
        .map(|(slug, key)| Entry {
            group: g_nav.clone(),
            label: t(key),
            hint: format!("§ {}", section_num(slug)),
            action: Action::Section(slug),
        })
        .collect();
    entries.push(Entry {
        group: g_nav.clone(),
        label: t("palette.imprint"),
        hint: "/imprint".into(),
        action: Action::Goto(Route::Imprint {}),
    });
    entries.push(Entry {
        group: g_nav.clone(),
        label: t("palette.privacy"),
        hint: "/privacy".into(),
        action: Action::Goto(Route::Privacy {}),
    });
    entries.push(Entry {
        group: g_nav.clone(),
        label: t("palette.licenses"),
        hint: "/licenses".into(),
        action: Action::Goto(Route::Licenses {}),
    });

    for r in repos.repos().iter().filter(|r| !r.fork && !r.archived) {
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
            label: t("palette.copyEmail"),
            hint: email_hint,
            action: Action::CopyEmail,
        },
        Entry {
            group: g_act.clone(),
            label: t("palette.toggleLang"),
            hint: other_language(&lang).to_uppercase(),
            action: Action::ToggleLang,
        },
        Entry {
            group: g_act.clone(),
            label: t("palette.openGithub"),
            hint: "↗".into(),
            action: Action::Open(CONFIG.github.into()),
        },
        Entry {
            group: g_act.clone(),
            label: t("palette.openLinkedin"),
            hint: "↗".into(),
            action: Action::Open(CONFIG.linkedin.into()),
        },
        Entry {
            group: g_act,
            label: t("palette.downloadResume"),
            hint: "PDF".into(),
            action: Action::Open(format!("/resume/{}", resume_file(&lang))),
        },
    ]);

    let needle = q().to_lowercase();
    let filtered: Vec<Entry> = if needle.is_empty() {
        entries
    } else {
        entries
            .into_iter()
            .filter(|e| {
                e.label.to_lowercase().contains(&needle) || e.group.to_lowercase().contains(&needle)
            })
            .collect()
    };

    let activate = use_callback(move |action: Action| {
        match &action {
            // Smooth-scroll to the section (routing home first if needed).
            Action::Section(_slug) => {
                #[cfg(feature = "web")]
                crate::ui::masthead::goto_section(on_home, crate::sections::section_id(_slug));
            }
            Action::Goto(target) => {
                navigator.push(target.clone());
            }
            Action::Open(_url) => {
                // `window.open` — unlike `<a target="_blank">` — does not imply
                // `noopener`, so without the feature string the opened page
                // could reach back through `window.opener` and navigate this
                // tab (reverse tabnabbing). Every anchor elsewhere in the app
                // sets the same pair.
                #[cfg(feature = "web")]
                if let Some(win) = web_sys::window() {
                    let _ = win.open_with_url_and_target_and_features(
                        _url,
                        "_blank",
                        "noopener,noreferrer",
                    );
                }
            }
            Action::CopyEmail =>
            {
                #[cfg(feature = "web")]
                if let Some(win) = web_sys::window() {
                    let _ = win.navigator().clipboard().write_text(CONFIG.email);
                }
            }
            Action::ToggleLang => {
                let target = other_language(&lang).to_string();
                persist_locale(&target);
                set_language.call(target);
            }
        }
        if !matches!(action, Action::ToggleLang) {
            on_close.call(());
        }
    });

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

    let n = filtered.len().max(1);
    let placeholder = t("palette.placeholder");
    let no_matches = t("palette.noMatches");
    let hint = t("palette.hint");
    let dialog_label = t("palette.dialogLabel");
    let is_empty = filtered.is_empty();

    // The modal element, so Tab can be kept inside it (client only).
    #[cfg(feature = "web")]
    let mut modal_el = use_signal(|| None::<web_sys::Element>);

    rsx! {
        div { class: "cmdk-overlay", onclick: move |_| on_close.call(()),
            div {
                class: "cmdk-modal",
                onclick: move |e| e.stop_propagation(),
                // Announced as a modal dialog, so assistive technology conveys
                // that the page behind it is inert and reads the label instead
                // of dropping the user into unlabelled content.
                role: "dialog",
                "aria-modal": "true",
                "aria-label": "{dialog_label}",
                onmounted: move |_e| {
                    #[cfg(feature = "web")]
                    {
                        use dioxus::web::WebEventExt;
                        if let Some(node) = _e.try_as_web_event() {
                            modal_el.set(Some(node));
                        }
                    }
                },
                // Tab must not walk out of an open dialog into the page behind
                // it; wrap at both ends instead.
                onkeydown: move |_e| {
                    #[cfg(feature = "web")]
                    if _e.key() == Key::Tab
                        && let Some(el) = modal_el()
                        && crate::hooks::trap_tab_focus(&el, _e.modifiers().shift())
                    {
                        _e.prevent_default();
                    }
                },
                div { class: "cmdk-search",
                    span { class: "mono text-accent", ">_" }
                    input {
                        autofocus: true,
                        value: "{q}",
                        placeholder: "{placeholder}",
                        oninput: move |e| {
                            q.set(e.value());
                            selected.set(0);
                        },
                        onkeydown: move |e| {
                            match e.key() {
                                Key::ArrowDown => {
                                    e.prevent_default();
                                    selected.set((selected() + 1) % n);
                                }
                                Key::ArrowUp => {
                                    e.prevent_default();
                                    selected.set((selected() + n - 1) % n);
                                }
                                Key::Enter => {
                                    e.prevent_default();
                                    if let Some(entry) = filtered.get(selected()) {
                                        activate.call(entry.action.clone());
                                    }
                                }
                                Key::Escape => on_close.call(()),
                                _ => {}
                            }
                        },
                    }
                    span { class: "mono text-muted", "ESC" }
                }
                div { class: "cmdk-list",
                    {grouped.into_iter().map(|(g, items)| rsx! {
                        div { key: "{g}", class: "cmdk-group",
                            span { class: "mono text-muted", "{g}" }
                            {items.into_iter().map(|(i, e)| {
                                let is_sel = i == selected();
                                let cls = if is_sel { "cmdk-item active" } else { "cmdk-item" };
                                let action = e.action.clone();
                                rsx! {
                                    button {
                                        key: "{i}",
                                        class: "{cls}",
                                        onclick: move |_| activate.call(action.clone()),
                                        onmouseenter: move |_| selected.set(i),
                                        span { "{e.label}" }
                                        span { class: "mono text-muted", "{e.hint}" }
                                    }
                                }
                            })}
                        }
                    })}
                    if is_empty {
                        div { class: "cmdk-empty",
                            span { class: "mono text-muted", "{no_matches}" }
                        }
                    }
                }
                div { class: "cmdk-footer",
                    span { class: "mono text-muted", "{hint}" }
                }
            }
        }
    }
}
