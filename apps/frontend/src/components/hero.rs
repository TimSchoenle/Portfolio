use std::cell::RefCell;
use std::rc::Rc;

use i18nrs::yew::use_translation;
use portfolio_data::{CONFIG, EXPERIENCE};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use yew::prelude::*;

use crate::hooks::{scroll_to, use_scroll_y};

use super::sections::{section_id, section_label, section_num};

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

    // A single wheel notch should fully transition the intro <-> the next
    // section (and back). CSS scroll-snap can't commit a small scroll across a
    // whole viewport, so we hijack the wheel near the intro/about boundary,
    // smooth-scroll to the target, and lock briefly to swallow the momentum.
    use_effect_with((), |_| {
        let win = web_sys::window().expect("window available");
        let lock = Rc::new(RefCell::new(false));
        let handler_lock = lock.clone();
        let cb = Closure::<dyn Fn(web_sys::Event)>::wrap(Box::new(move |e: web_sys::Event| {
            let Some(win) = web_sys::window() else { return };
            if *handler_lock.borrow() {
                e.prevent_default();
                return;
            }
            let y = win.scroll_y().unwrap_or(0.0);
            let vh = win
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(800.0);
            let dy = js_sys::Reflect::get(&e, &JsValue::from_str("deltaY"))
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let target = if dy > 0.0 && y < vh * 0.5 {
                Some(section_id("about"))
            } else if dy < 0.0 && y > vh * 0.5 && y < vh * 1.3 {
                Some("top".to_string())
            } else {
                None
            };

            if let Some(id) = target {
                e.prevent_default();
                *handler_lock.borrow_mut() = true;
                scroll_to(&id);
                let release = handler_lock.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(900).await;
                    *release.borrow_mut() = false;
                });
            }
        }));

        let opts = web_sys::AddEventListenerOptions::new();
        opts.set_passive(false);
        win.add_event_listener_with_callback_and_add_event_listener_options(
            "wheel",
            cb.as_ref().unchecked_ref(),
            &opts,
        )
        .ok();

        move || {
            if let Some(win) = web_sys::window() {
                win.remove_event_listener_with_callback("wheel", cb.as_ref().unchecked_ref())
                    .ok();
            }
            drop(cb);
        }
    });

    html! {
        <section id="top" class="hero">
            <div class="hero-eyebrow">
                <div class="bracket-line" />
                <span class="mono text-accent">{ section_label("identity") }</span>
                <span class="mono text-muted">{ i18n.t("hero.eyebrow") }</span>

                <div class="hero-meta">
                    <div class="hero-meta-card">
                        <span class="mono text-muted">{ format!("§ {}.a", section_num("identity")) }</span>
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
            </div>

            <h1 class="hero-name" style={format!("transform: translateY({}px)", scroll * -0.08)}>
                { hero_name_lines() }
            </h1>

            <div class="hero-tagline">
                <div class="tagline-label">
                    <span class="mono text-muted">{ format!("§ {}.b", section_num("identity")) }</span>
                </div>
                <div class="tagline-body">
                    <p class="tagline-main">{ i18n.t("hero.tagline") }</p>
                    <p class="tagline-sub">{ i18n.t("hero.taglineSub") }</p>
                </div>
            </div>

            <a href={ format!("#{}", section_id("about")) } class="scroll-cue" aria-label="Scroll">
                <span class="mono text-muted">{"SCROLL"}</span>
                <span class="scroll-cue-line" />
            </a>
        </section>
    }
}
