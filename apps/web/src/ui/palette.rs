//! Command palette (⌘K).
//!
//! Search, grouping and keyboard navigation are pure Dioxus state. The actions
//! that need the browser (open external URL, copy email, smooth-scroll to a
//! section) are wired in the hydration phase; navigation and language switching
//! work here.

use dioxus::prelude::*;
use i18nrs::I18n;
use portfolio_data::{CONFIG, next_language, resume_file};

use crate::embedded::repos::ReposState;
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
    // Everything the palette can do, with each entry's lowercased search text. It depends on
    // the language and on nothing typed, so it is built when the language changes rather than
    // on every keystroke.
    let entries = use_memo(move || searchable_entries(i18n, legal_indexes.as_ref(), &repos));

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
            Action::Open(url) => open_in_new_tab(url),
            Action::CopyEmail => copy_email(),
            Action::ToggleLang => {
                switch_language(&ctx, next_language(&lang).code);
            }
        }
        if !matches!(action, Action::ToggleLang) {
            on_close.call(());
        }
    });

    let groups = group_in_first_seen_order(&filtered);
    let placeholder = t("palette.placeholder");
    let no_matches = t("palette.noMatches");
    let hint = t("palette.hint");
    let dialog_label = t("palette.dialogLabel");

    let active_id = (!filtered.is_empty()).then(|| format!("cmdk-opt-{}", selected()));
    let list_label = dialog_label.clone();

    rsx! {
        PaletteDialog { label: dialog_label, on_close,
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
                    onkeydown: move |e| on_search_key(&e, selected, &filtered, activate, on_close),
                }
                span { class: "mono text-muted", "ESC" }
            }
            PaletteResults {
                groups,
                selected,
                activate,
                label: list_label,
                empty_text: no_matches,
            }
            div { class: "cmdk-footer",
                span { class: "mono text-muted", "{hint}" }
            }
        }
    }
}

/// The modal frame around the palette: the overlay that closes it on an outside click, the
/// dialog role and label, Tab kept inside, and focus handed back on close to whatever held it
/// before the palette opened.
#[component]
fn PaletteDialog(label: String, on_close: EventHandler<()>, children: Element) -> Element {
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
                "aria-label": "{label}",
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
                {children}
            }
        }
    }
}

/// The listbox under the search field: the matching entries in their groups, or the empty state.
///
/// `groups` carries each entry's index in the flat filtered list, which is the index the arrow
/// keys move `selected` through.
#[component]
fn PaletteResults(
    groups: Vec<(String, Vec<(usize, Entry)>)>,
    selected: Signal<usize>,
    activate: Callback<Action>,
    label: String,
    empty_text: String,
) -> Element {
    let is_empty = groups.is_empty();
    rsx! {
        div {
            id: "cmdk-list",
            class: "cmdk-list",
            role: "listbox",
            "aria-label": "{label}",
            {groups.into_iter().map(|(g, items)| rsx! {
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
                    span { class: "mono text-muted", "{empty_text}" }
                }
            }
        }
    }
}

/// Moves the selection with the arrow keys, runs the selected entry on Enter, and closes on
/// Escape. Arrow keys wrap at both ends of `filtered`.
fn on_search_key(
    e: &KeyboardEvent,
    mut selected: Signal<usize>,
    filtered: &[Entry],
    activate: Callback<Action>,
    on_close: EventHandler<()>,
) {
    let n = filtered.len().max(1);
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
}

/// Groups entries by `group` in the order each group first appears, keeping every entry's flat
/// index so the highlight matches keyboard-navigation order.
fn group_in_first_seen_order(filtered: &[Entry]) -> Vec<(String, Vec<(usize, Entry)>)> {
    let mut grouped: Vec<(String, Vec<(usize, Entry)>)> = Vec::new();
    for (flat_i, e) in filtered.iter().enumerate() {
        if let Some(slot) = grouped.iter_mut().find(|(g, _)| *g == e.group) {
            slot.1.push((flat_i, e.clone()));
        } else {
            grouped.push((e.group.clone(), vec![(flat_i, e.clone())]));
        }
    }
    grouped
}

/// Every entry the palette offers in the current language, each paired with the lowercased text
/// a query is matched against.
fn searchable_entries(
    i18n: Signal<I18n>,
    legal_indexes: Option<&LegalIndexes>,
    repos: &ReposState,
) -> Vec<(String, Entry)> {
    let t = move |k: &str| i18n.read().t(k);
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
    entries.extend(legal_entries(i18n, legal_indexes, &lang, &g_nav));
    entries.push(Entry {
        group: g_nav,
        label: t("palette.licenses"),
        hint: "/licenses".into(),
        action: Action::Goto(Route::Licenses {}),
    });
    entries.extend(
        repos
            .repos()
            .iter()
            .filter(|r| !r.fork && !r.archived)
            .map(|r| Entry {
                group: g_work.clone(),
                label: r.name.clone(),
                hint: r.language.clone().unwrap_or_else(|| "—".into()),
                action: Action::Open(r.html_url.clone()),
            }),
    );
    entries.extend(action_entries(&t, &lang, &g_act));

    entries
        .into_iter()
        .map(|entry| {
            (
                format!("{}\n{}", entry.label, entry.group).to_lowercase(),
                entry,
            )
        })
        .collect()
}

/// Every published legal document, under the operator's title and in their order. A document
/// hosted elsewhere opens in a new tab; one this site hosts is a route.
fn legal_entries(
    i18n: Signal<I18n>,
    legal_indexes: Option<&LegalIndexes>,
    lang: &str,
    group: &str,
) -> Vec<Entry> {
    let legal_text = SiteText::new(i18n);
    let docs = legal_indexes
        .and_then(|indexes| indexes.get(lang).cloned())
        .unwrap_or_default();
    docs.into_iter()
        .map(|doc| {
            let label = legal_title(&legal_text, &doc);
            let (hint, action) = match doc.url.clone().filter(|_| !is_hosted(&doc)) {
                Some(url) => ("↗".to_owned(), Action::Open(url)),
                None => (
                    document_path(&doc.slug),
                    Action::Goto(Route::LegalDocument { slug: doc.slug }),
                ),
            };
            Entry {
                group: group.to_owned(),
                label,
                hint,
                action,
            }
        })
        .collect()
}

/// The fixed actions: copy the email, switch the language, open the profiles, get the resume.
fn action_entries(t: &impl Fn(&str) -> String, lang: &str, group: &str) -> [Entry; 5] {
    let entry = |label: &str, hint: String, action: Action| Entry {
        group: group.to_owned(),
        label: t(label),
        hint,
        action,
    };
    let email_hint = CONFIG.email.split('@').next().unwrap_or("").to_string() + "@…";
    [
        entry("palette.copyEmail", email_hint, Action::CopyEmail),
        entry(
            "palette.toggleLang",
            next_language(lang).code.to_uppercase(),
            Action::ToggleLang,
        ),
        entry(
            "palette.openGithub",
            "↗".into(),
            Action::Open(CONFIG.github.into()),
        ),
        entry(
            "palette.openLinkedin",
            "↗".into(),
            Action::Open(CONFIG.linkedin.into()),
        ),
        entry(
            "palette.downloadResume",
            "PDF".into(),
            Action::Open(format!("/resume/{}", resume_file(lang))),
        ),
    ]
}

/// Opens `url` in a new tab with `noopener,noreferrer`. A no-op outside the wasm client.
///
/// `window.open` — unlike `<a target="_blank">` — does not imply `noopener`, so without the
/// feature string the opened page could reach back through `window.opener` and navigate this
/// tab (reverse tabnabbing). Every anchor elsewhere in the app sets the same pair.
fn open_in_new_tab(url: &str) {
    #[cfg(feature = "web")]
    if let Some(win) = web_sys::window() {
        let _ = win.open_with_url_and_target_and_features(url, "_blank", "noopener,noreferrer");
    }
    #[cfg(not(feature = "web"))]
    let _ = url;
}

/// Copies the contact address to the clipboard. A no-op outside the wasm client.
fn copy_email() {
    #[cfg(feature = "web")]
    if let Some(win) = web_sys::window() {
        let _ = win.navigator().clipboard().write_text(CONFIG.email);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(group: &str, label: &str) -> Entry {
        Entry {
            group: group.to_owned(),
            label: label.to_owned(),
            hint: String::new(),
            action: Action::CopyEmail,
        }
    }

    #[test]
    fn groups_in_first_seen_order_and_keeps_each_flat_index() {
        let filtered = [
            entry("nav", "a"),
            entry("work", "b"),
            entry("nav", "c"),
            entry("act", "d"),
        ];
        let grouped: Vec<(String, Vec<(usize, String)>)> = group_in_first_seen_order(&filtered)
            .into_iter()
            .map(|(g, items)| (g, items.into_iter().map(|(i, e)| (i, e.label)).collect()))
            .collect();
        assert_eq!(
            grouped,
            [
                (
                    "nav".to_owned(),
                    vec![(0, "a".to_owned()), (2, "c".to_owned())]
                ),
                ("work".to_owned(), vec![(1, "b".to_owned())]),
                ("act".to_owned(), vec![(3, "d".to_owned())]),
            ]
        );
    }

    #[test]
    fn grouping_nothing_yields_no_group() {
        assert_eq!(group_in_first_seen_order(&[]).len(), 0);
    }

    #[test]
    fn the_fixed_actions_hint_at_their_target() {
        let t = |key: &str| key.to_owned();
        let [copy, toggle, github, linkedin, resume] = action_entries(&t, "en", "act");
        let local = CONFIG.email.split('@').next().unwrap_or_default();
        assert_eq!(copy.hint, format!("{local}@…"));
        assert!(copy.action == Action::CopyEmail);
        assert_eq!(toggle.hint, next_language("en").code.to_uppercase());
        assert!(toggle.action == Action::ToggleLang);
        assert!(github.action == Action::Open(CONFIG.github.into()));
        assert!(linkedin.action == Action::Open(CONFIG.linkedin.into()));
        assert!(resume.action == Action::Open(format!("/resume/{}", resume_file("en"))));
        assert!(
            [copy, toggle, github, linkedin, resume]
                .iter()
                .all(|e| e.group == "act")
        );
    }

    #[cfg(feature = "server")]
    mod rendered {
        use super::*;
        use crate::ui::test_render::{render, text};
        use portfolio_data::{Repo, ReposFile};
        use terrace_legal_model::LegalIndexEntry;

        fn repo(name: &str, fork: bool, archived: bool) -> Repo {
            Repo {
                name: name.to_owned(),
                full_name: format!("owner/{name}"),
                description: None,
                html_url: format!("https://github.com/owner/{name}"),
                language: (name == "kept").then(|| "Rust".to_owned()),
                stargazers_count: 0,
                forks_count: 0,
                updated_at: String::new(),
                topics: Vec::new(),
                fork,
                archived,
                homepage: None,
            }
        }

        fn legal(slug: &str, kind: &str, url: Option<&str>) -> LegalIndexEntry {
            serde_json::from_value(serde_json::json!({
                "slug": slug,
                "title": format!("{slug} title"),
                "updated": null,
                "kind": kind,
                "url": url,
                "locales": ["en"],
            }))
            .expect("a well-formed index entry")
        }

        /// Renders every entry the palette offers in `en` as `group|label|hint|haystack`.
        fn entries_html() -> String {
            render("en", || {
                let i18n = use_i18n().i18n;
                let repos = ReposState::Ready(std::sync::Arc::new(ReposFile {
                    generated_at: String::new(),
                    user: "owner".to_owned(),
                    repos: vec![
                        repo("kept", false, false),
                        repo("forked", true, false),
                        repo("shelved", false, true),
                    ],
                }));
                let indexes = crate::legal::PerLanguage(vec![(
                    "en".to_owned(),
                    vec![
                        legal("imprint", "inline", None),
                        legal("privacy", "external", Some("https://example.com/privacy")),
                    ],
                )]);
                let entries = searchable_entries(i18n, Some(&indexes), &repos);
                rsx! {
                    for (haystack, e) in entries {
                        p { "{e.group}|{e.label}|{e.hint}|{haystack}" }
                    }
                }
            })
        }

        #[test]
        fn offers_every_section_then_the_licenses_page() {
            let html = entries_html();
            let nav = text("en", "palette.groupNav");
            for (slug, key) in SECTIONS {
                let line = format!("{nav}|{}|§ {}|", text("en", key), section_num(slug));
                assert!(html.contains(&line), "{line} missing");
            }
            assert!(html.contains(&format!(
                "{nav}|{}|/licenses|",
                text("en", "palette.licenses")
            )));
        }

        #[test]
        fn links_a_hosted_document_by_route_and_an_external_one_by_url() {
            let html = entries_html();
            assert!(html.contains("|imprint title|/legal/imprint|"));
            assert!(html.contains("|privacy title|↗|"));
        }

        #[test]
        fn lists_repositories_but_not_forks_or_archived_ones() {
            let html = entries_html();
            let work = text("en", "palette.groupWork");
            assert!(html.contains(&format!("{work}|kept|Rust|")));
            assert!(!html.contains("|forked|"));
            assert!(!html.contains("|shelved|"));
        }

        #[test]
        fn matches_a_query_against_the_lowercased_label_and_group() {
            let html = entries_html();
            let work = text("en", "palette.groupWork").to_lowercase();
            assert!(html.contains(&format!("|kept\n{work}</p>")));
        }

        #[test]
        fn highlights_the_selected_option_and_names_each_group() {
            let html = render("en", || {
                let selected = use_signal(|| 1usize);
                let activate = use_callback(|_: Action| {});
                let groups = group_in_first_seen_order(&[entry("nav", "a"), entry("nav", "b")]);
                rsx! {
                    PaletteResults {
                        groups,
                        selected,
                        activate,
                        label: "Commands".to_owned(),
                        empty_text: "Nothing".to_owned(),
                    }
                }
            });
            assert!(html.contains(r#"aria-label="nav""#));
            assert_eq!(html.matches(r#"aria-selected="true""#).count(), 1);
            assert!(html.contains(r#"id="cmdk-opt-1" class="cmdk-item active""#));
            assert!(!html.contains("Nothing"));
        }

        #[test]
        fn says_so_when_nothing_matches() {
            let html = render("en", || {
                let selected = use_signal(|| 0usize);
                let activate = use_callback(|_: Action| {});
                rsx! {
                    PaletteResults {
                        groups: Vec::new(),
                        selected,
                        activate,
                        label: "Commands".to_owned(),
                        empty_text: "Nothing".to_owned(),
                    }
                }
            });
            assert!(html.contains("cmdk-empty"));
            assert!(html.contains("Nothing"));
        }
    }
}
