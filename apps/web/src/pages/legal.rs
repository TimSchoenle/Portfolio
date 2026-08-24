//! Shared layout and rich-text helpers for the legal pages.
//!
//! Legal body texts live in the i18n files as plain strings. Paragraphs are
//! separated by blank lines, list items start with "- ", and URLs are
//! auto-linked.

use dioxus::prelude::*;
use portfolio_data::CONFIG;

use crate::i18n::use_i18n;
use crate::routes::Route;
use crate::ui::canonical::Canonical;

/// Frames a legal page: back link, title, last-updated line, then `children` as its sections.
///
/// `last_updated` is printed verbatim after the localized label, neither parsed nor reformatted.
#[component]
pub fn LegalPage(
    title: String,
    last_updated: &'static str,
    /// This page's own route path, so it declares itself canonical rather than
    /// inheriting a site-wide one that would point at the homepage.
    canonical_path: &'static str,
    children: Element,
) -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);

    let full_title = format!("{} · {}", title, CONFIG.full_name);
    let updated = format!("{}: {}", t("common.lastUpdated"), last_updated);

    rsx! {
        document::Title { "{full_title}" }
        Canonical { path: canonical_path }
        section { class: "legal-page",
            Link { to: Route::Home {}, class: "legal-back mono",
                "← "
                {t("common.backToHome")}
            }
            h1 { class: "legal-title", "{title}" }
            span { class: "mono text-muted", "{updated}" }
            div { class: "legal-sections", {children} }
        }
    }
}

/// One numbered section of a legal page.
///
/// `body` is a raw translation string, run through the paragraph and list splitting below.
/// `children` render after it, for the sections that need markup a translation string cannot
/// carry.
#[component]
pub fn LegalSection(
    heading: String,
    #[props(default)] body: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        section {
            h2 { "{heading}" }
            {children}
            if let Some(b) = body {
                {rich_text(&b)}
            }
        }
    }
}

/// The controller / imprint postal address with localized labels.
#[component]
pub fn AddressBlock() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let email_label = t("legal.emailLabel");
    let form_label = t("legal.contactFormLabel");

    rsx! {
        div { class: "address-block",
            {CONFIG.legal.address_lines.iter().map(|line| rsx! { div { key: "{line}", "{line}" } })}
            div { {t("common.country")} }
            div { class: "pt-3",
                span { class: "address-label", "{email_label}: " }
                a { href: "mailto:{CONFIG.email}", "{CONFIG.email}" }
            }
            div {
                span { class: "address-label", "{form_label}: " }
                a {
                    href: CONFIG.legal.second_contact_url,
                    target: "_blank",
                    rel: "noopener",
                    "{CONFIG.legal.second_contact_url}"
                }
            }
        }
    }
}

/// Splits a translation body into paragraphs and lists, auto-linking URLs.
fn rich_text(body: &str) -> Element {
    rsx! {
        {body.split("\n\n").filter(|b| !b.trim().is_empty()).enumerate().map(|(bi, block)| {
            let lines: Vec<&str> = block.lines().collect();
            let is_list = lines.iter().all(|l| l.trim_start().starts_with("- "));
            if is_list {
                rsx! {
                    ul { key: "{bi}",
                        {lines.iter().enumerate().map(|(li, l)| {
                            let item = l.trim_start().trim_start_matches("- ").to_string();
                            rsx! { li { key: "{li}", {linkify(&item)} } }
                        })}
                    }
                }
            } else {
                rsx! { p { key: "{bi}", {linkify(block)} } }
            }
        })}
    }
}

/// Turns http(s) URLs inside plain text into anchors.
fn linkify(text: &str) -> Element {
    rsx! {
        {text.split(' ').enumerate().map(|(i, token)| {
            let prefix = if i > 0 { " " } else { "" };
            if token.starts_with("http://") || token.starts_with("https://") {
                let trimmed = token.trim_end_matches(['.', ',', ';', ':', ')']).to_string();
                let trailing = token[trimmed.len()..].to_string();
                let _ = i;
                rsx! {
                    "{prefix}"
                    a { href: "{trimmed}", target: "_blank", rel: "noopener", "{trimmed}" }
                    "{trailing}"
                }
            } else {
                rsx! { "{prefix}{token}" }
            }
        })}
    }
}
