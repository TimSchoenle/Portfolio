use i18nrs::yew::use_translation;
use portfolio_data::{CONFIG, EXPERIENCE};
use yew::prelude::*;

use crate::hooks::use_scroll_y;

/// Full years since the earliest experience entry, e.g. 7 -> "7+".
fn years_of_experience() -> u32 {
    let Some(earliest) = EXPERIENCE
        .iter()
        .min_by_key(|e| (e.start.year, e.start.month))
    else {
        return 0;
    };
    let now = js_sys::Date::new_0();
    let mut years = now.get_full_year() as i32 - earliest.start.year as i32;
    if (now.get_month() as u8 + 1) < earliest.start.month {
        years -= 1;
    }
    years.max(0) as u32
}

/// Splits the full name into hero lines, coloring "ö" and the trailing dot
/// like the v4 design ("Tim" / "Sch<ö>nle<.>").
fn hero_name_lines() -> Html {
    let mut parts = CONFIG.full_name.split_whitespace();
    let first = parts.next().unwrap_or_default();
    let last = parts.next().unwrap_or_default();
    html! {
        <>
            <span class="hero-name-line">{first}</span>
            <span class="hero-name-line">
                { for last.chars().map(|c| {
                    if c == 'ö' {
                        html!{ <span class="hero-accent-char">{"ö"}</span> }
                    } else {
                        html!{ {c.to_string()} }
                    }
                })}
                <span class="hero-accent-char">{"."}</span>
            </span>
        </>
    }
}

#[function_component(Hero)]
pub fn hero() -> Html {
    let (i18n, _) = use_translation();
    let scroll = use_scroll_y(500.0);
    let years = years_of_experience();

    html! {
        <section id="top" class="hero">
            <div class="hero-eyebrow">
                <div class="bracket-line" />
                <span class="mono text-accent">{"§ 00 — identity"}</span>
                <span class="mono text-muted">{ i18n.t("hero.eyebrow") }</span>
            </div>

            <h1 class="hero-name" style={format!("transform: translateY({}px)", scroll * -0.08)}>
                { hero_name_lines() }
            </h1>

            <div class="hero-meta">
                <div class="hero-meta-card">
                    <span class="mono text-muted">{"§ 00.a"}</span>
                    <dl class="meta-dl">
                        <dt><span class="mono text-muted">{"ROLE"}</span></dt>
                        <dd>{ i18n.t("hero.jobTitle") }</dd>
                        <dt><span class="mono text-muted">{"LOC"}</span></dt>
                        <dd>{ i18n.t("common.country") }</dd>
                        <dt><span class="mono text-muted">{"YRS"}</span></dt>
                        <dd>{ format!("{years}+") }</dd>
                        <dt><span class="mono text-muted">{ i18n.t("hero.statusLabel") }</span></dt>
                        <dd class="text-accent flex items-center gap-2">
                            <span class="pulse-dot" />{ i18n.t("hero.status") }
                        </dd>
                    </dl>
                </div>
            </div>

            <div class="hero-tagline">
                <div class="tagline-label">
                    <span class="mono text-muted">{"§ 01 — intro"}</span>
                </div>
                <div class="tagline-body">
                    <p class="tagline-main">{ i18n.t("hero.tagline") }</p>
                    <p class="tagline-sub">{ i18n.t("hero.taglineSub") }</p>
                </div>
            </div>

            <a href="#s1" class="scroll-cue" aria-label="Scroll">
                <span class="mono text-muted">{"SCROLL"}</span>
                <span class="scroll-cue-line" />
            </a>
        </section>
    }
}
