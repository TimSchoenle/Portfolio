//! Shared layout and rich-text helpers for the legal pages.
//!
//! Legal body texts live in the i18n files as plain strings. Paragraphs are
//! separated by blank lines, list items start with "- ", and URLs are
//! auto-linked.

use i18nrs::yew::use_translation;
use portfolio_data::CONFIG;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::i18n::set_document_title;
use crate::router::Route;

#[derive(Properties, PartialEq)]
pub struct LegalPageProps {
    pub title: String,
    /// ISO date of the last content change.
    pub last_updated: &'static str,
    #[prop_or_default]
    pub children: Html,
}

#[function_component(LegalPage)]
pub fn legal_page(p: &LegalPageProps) -> Html {
    let (i18n, _) = use_translation();

    {
        let title = format!("{} · {}", p.title, CONFIG.full_name);
        use_effect_with(title, |title| set_document_title(title));
    }

    html! {
        <section class="legal-page">
            <Link<Route> to={Route::Home} classes="legal-back mono">
                {"← "}{ i18n.t("common.backToHome") }
            </Link<Route>>

            <h1 class="legal-title">{ p.title.clone() }</h1>
            <span class="mono text-muted">
                { format!("{}: {}", i18n.t("common.lastUpdated"), p.last_updated) }
            </span>

            <div class="legal-sections">
                { p.children.clone() }
            </div>
        </section>
    }
}

#[derive(Properties, PartialEq)]
pub struct LegalSectionProps {
    pub heading: String,
    #[prop_or_default]
    pub body: Option<String>,
    #[prop_or_default]
    pub children: Html,
}

#[function_component(LegalSection)]
pub fn legal_section(p: &LegalSectionProps) -> Html {
    html! {
        <section>
            <h2>{ p.heading.clone() }</h2>
            { p.children.clone() }
            if let Some(body) = &p.body {
                { rich_text(body) }
            }
        </section>
    }
}

/// Renders the controller / imprint postal address with localized labels.
#[function_component(AddressBlock)]
pub fn address_block() -> Html {
    let (i18n, _) = use_translation();
    html! {
        <div class="address-block">
            { for CONFIG.legal.address_lines.iter().map(|line| html!{ <div>{*line}</div> }) }
            <div>{ i18n.t("common.country") }</div>
            <div class="pt-3">
                <span class="address-label">{ i18n.t("legal.emailLabel") }{": "}</span>
                <a href={format!("mailto:{}", CONFIG.email)}>{CONFIG.email}</a>
            </div>
            <div>
                <span class="address-label">{ i18n.t("legal.contactFormLabel") }{": "}</span>
                <a href={CONFIG.legal.second_contact_url} target="_blank" rel="noopener">
                    {CONFIG.legal.second_contact_url}
                </a>
            </div>
        </div>
    }
}

/// Splits a translation body into paragraphs and lists, auto-linking URLs.
pub fn rich_text(body: &str) -> Html {
    let blocks = body.split("\n\n").filter(|b| !b.trim().is_empty());
    html! {
        <>
            { for blocks.map(|block| {
                let lines: Vec<&str> = block.lines().collect();
                let is_list = lines.iter().all(|l| l.trim_start().starts_with("- "));
                if is_list {
                    html! {
                        <ul>
                            { for lines.iter().map(|l| html!{
                                <li>{ linkify(l.trim_start().trim_start_matches("- ")) }</li>
                            })}
                        </ul>
                    }
                } else {
                    html! { <p>{ linkify(block) }</p> }
                }
            })}
        </>
    }
}

/// Turns http(s) URLs inside plain text into anchors.
fn linkify(text: &str) -> Html {
    let mut parts: Vec<Html> = Vec::new();
    for (i, token) in text.split(' ').enumerate() {
        if i > 0 {
            parts.push(html! { {" "} });
        }
        if token.starts_with("http://") || token.starts_with("https://") {
            let trimmed = token.trim_end_matches(['.', ',', ';', ':', ')']);
            let trailing = &token[trimmed.len()..];
            parts.push(html! {
                <a href={trimmed.to_string()} target="_blank" rel="noopener">{trimmed.to_string()}</a>
            });
            if !trailing.is_empty() {
                parts.push(html! { {trailing.to_string()} });
            }
        } else {
            parts.push(html! { {token.to_string()} });
        }
    }
    html! { <>{ for parts.into_iter() }</> }
}
