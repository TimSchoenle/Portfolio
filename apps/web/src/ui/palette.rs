//! Command palette (⌘K).
//!
//! Search, grouping and keyboard navigation are pure Dioxus state. The actions
//! that need the browser (open external URL, copy email, smooth-scroll to a
//! section) are wired in the hydration phase; navigation and language switching
//! work here.

use dioxus::prelude::*;
use portfolio_data::{CONFIG, next_language, resume_file};

use crate::github::ReposState;
use terrace_legal_dioxus::legal_title;

use crate::i18n::{switch_language, use_i18n};
use crate::legal::{LegalIndexes, SiteText, document_path, is_hosted};
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
    let t = move |k: &str| i18n.read().t(k);
    let navigator = use_navigator();
    #[cfg(feature = "web")]
    let on_home = matches!(use_route::<Route>(), Route::Home {});

    let mut q = use_signal(String::new);
    let mut selected = use_signal(|| 0usize);

    let lang = i18n.read().get_current_language().to_string();
    let legal_indexes = try_consume_context::<LegalIndexes>();
    let entry_repos = repos.clone();
    // Everything the palette can do, with each entry's lowercased search text. It depends on
    // the language and on nothing typed, so it is built when the language changes rather than
    // on every keystroke.
    let entries = use_memo(move || {
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
        // Every published legal document, under the operator's title and in their order.
        let legal_text = SiteText::new(i18n);
        let docs = legal_indexes
            .as_ref()
            .and_then(|indexes| indexes.get(&lang).cloned())
            .unwrap_or_default();
        for doc in docs {
            let label = legal_title(&legal_text, &doc);
            let (hint, action) = match doc.url.clone().filter(|_| !is_hosted(&doc)) {
                Some(url) => ("↗".to_owned(), Action::Open(url)),
                None => (
                    document_path(&doc.slug),
                    Action::Goto(Route::LegalDocument { slug: doc.slug }),
                ),
            };
            entries.push(Entry {
                group: g_nav.clone(),
                label,
                hint,
                action,
            });
        }
        entries.push(Entry {
            group: g_nav.clone(),
            label: t("palette.licenses"),
            hint: "/licenses".into(),
            action: Action::Goto(Route::Licenses {}),
        });

        for r in entry_repos
            .repos()
            .iter()
            .filter(|r| !r.fork && !r.archived)
        {
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
                hint: next_language(&lang).code.to_uppercase(),
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

        entries
            .into_iter()
            .map(|entry| {
                (
                    format!("{}\n{}", entry.label, entry.group).to_lowercase(),
                    entry,
                )
            })
            .collect::<Vec<_>>()
    });

    let needle = q().to_lowercase();
    let filtered: Vec<Entry> = entries
        .read()
        .iter()
        .filter(|(haystack, _)| haystack.contains(&needle))
        .map(|(_, entry)| entry.clone())
        .collect();

    let activate = use_callback(move |action: Action| {
        match &action {
            // Smooth-scroll to the section (routing home first if needed).
            Action::Section(slug) => {
                #[cfg(feature = "web")]
                crate::ui::masthead::goto_section(on_home, crate::sections::section_id(slug));
                #[cfg(not(feature = "web"))]
                let _ = slug;
            }
            Action::Goto(target) => {
                navigator.push(target.clone());
            }
            Action::Open(url) => {
                // `window.open` — unlike `<a target="_blank">` — does not imply
                // `noopener`, so without the feature string the opened page
                // could reach back through `window.opener` and navigate this
                // tab (reverse tabnabbing). Every anchor elsewhere in the app
                // sets the same pair.
                #[cfg(feature = "web")]
                if let Some(win) = web_sys::window() {
                    let _ = win.open_with_url_and_target_and_features(
                        url,
                        "_blank",
                        "noopener,noreferrer",
                    );
                }
                #[cfg(not(feature = "web"))]
                let _ = url;
            }
            Action::CopyEmail =>
            {
                #[cfg(feature = "web")]
                if let Some(win) = web_sys::window() {
                    let _ = win.navigator().clipboard().write_text(CONFIG.email);
                }
            }
            Action::ToggleLang => {
                switch_language(&ctx, next_language(&lang).code);
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

    // Focus goes back to whatever held it before the palette opened — the trigger button, or
    // the page for the keyboard shortcut — rather than to the top of the document.
    #[cfg(feature = "web")]
    {
        use web_sys::wasm_bindgen::JsCast;
        let opener = use_hook(|| {
            web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.active_element())
                .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
        });
        use_drop(move || {
            if let Some(el) = &opener {
                let _ = el.focus();
            }
        });
    }

    let active_id = (!is_empty).then(|| format!("cmdk-opt-{}", selected()));
    let list_label = dialog_label.clone();

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
                onmounted: move |e| {
                    #[cfg(feature = "web")]
                    {
                        use dioxus::web::WebEventExt;
                        if let Some(node) = e.try_as_web_event() {
                            modal_el.set(Some(node));
                        }
                    }
                    #[cfg(not(feature = "web"))]
                    let _ = e;
                },
                // Tab must not walk out of an open dialog into the page behind
                // it; wrap at both ends instead.
                onkeydown: move |e| {
                    #[cfg(feature = "web")]
                    if e.key() == Key::Tab
                        && let Some(el) = modal_el()
                        && crate::hooks::trap_tab_focus(&el, e.modifiers().shift())
                    {
                        e.prevent_default();
                    }
                    #[cfg(not(feature = "web"))]
                    let _ = e;
                },
                div { class: "cmdk-search",
                    span { class: "mono text-accent", ">_" }
                    // The ARIA combobox pattern: focus stays in the field, and the highlighted
                    // option is announced through `aria-activedescendant` as the arrows move it.
                    input {
                        autofocus: true,
                        role: "combobox",
                        "aria-expanded": "true",
                        "aria-controls": "cmdk-list",
                        "aria-autocomplete": "list",
                        "aria-activedescendant": active_id,
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
                div {
                    id: "cmdk-list",
                    class: "cmdk-list",
                    role: "listbox",
                    "aria-label": "{list_label}",
                    {grouped.into_iter().map(|(g, items)| rsx! {
                        div { key: "{g}", class: "cmdk-group", role: "group", "aria-label": "{g}",
                            span { class: "mono text-muted", "aria-hidden": "true", "{g}" }
                            {items.into_iter().map(|(i, e)| {
                                let is_sel = i == selected();
                                let cls = if is_sel { "cmdk-item active" } else { "cmdk-item" };
                                let action = e.action.clone();
                                rsx! {
                                    // Not in the Tab order: the field owns focus, and the
                                    // arrows move the selection.
                                    button {
                                        key: "{i}",
                                        id: "cmdk-opt-{i}",
                                        class: "{cls}",
                                        role: "option",
                                        tabindex: "-1",
                                        "aria-selected": if is_sel { "true" } else { "false" },
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
