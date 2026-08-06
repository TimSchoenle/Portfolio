//! Identity hero: eyebrow meta card, oversized name, tagline, scroll cue.
//!
//! The wheel-hijack that snaps between the hero and the about section, and the
//! scroll-driven name parallax, are client-only enhancements re-added in the
//! hydration phase.

use dioxus::prelude::*;
use portfolio_data::{CONFIG, EXPERIENCE};

use crate::i18n::use_i18n;
use crate::sections::{section_id, section_label, section_num};
use crate::util::{current_month, current_year};

/// Full years since the earliest experience entry, e.g. 7 -> "7+".
fn years_of_experience() -> u32 {
    let Some(earliest) = EXPERIENCE
        .iter()
        .min_by_key(|e| (e.start.year, e.start.month))
    else {
        return 0;
    };
    let mut years = current_year() - earliest.start.year as i32;
    if current_month() < earliest.start.month {
        years -= 1;
    }
    years.max(0) as u32
}

/// The full name split into hero lines, coloring "ö" and the trailing dot.
///
/// The first whitespace-separated token is the given name; everything after it
/// forms the second line, so a middle name or a multi-word family name is
/// rendered rather than silently dropped.
fn hero_name_lines() -> Element {
    let mut parts = CONFIG.full_name.split_whitespace();
    let first = parts.next().unwrap_or_default();
    let last = parts.collect::<Vec<_>>().join(" ");
    rsx! {
        span { class: "hero-name-line", "{first}" }
        span { class: "hero-name-line",
            {last.chars().map(|c| {
                if c == 'ö' {
                    rsx! { span { class: "hero-accent-char", "ö" } }
                } else {
                    rsx! { "{c}" }
                }
            })}
            span { class: "hero-accent-char", "." }
        }
    }
}

#[component]
pub fn Hero() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let years = years_of_experience();
    let identity_num = section_num("identity");
    let about_id = section_id("about");

    // Scroll offset driving the name parallax. Stays `0.0` on the server and
    // under reduced motion, so the name renders untransformed (SSR-safe).
    let scroll = use_signal(|| 0.0_f64);

    #[cfg(feature = "web")]
    {
        use crate::hooks::{
            FrameListenerGuard, ListenerGuard, add_window_listener, add_window_listener_per_frame,
            prefers_reduced_motion, scroll_to, scroll_y, viewport_height,
        };
        use std::cell::RefCell;
        use std::rc::Rc;
        use web_sys::wasm_bindgen::JsCast;

        let mut scroll = scroll;
        // Parallax: track the capped scroll offset unless reduced motion is on.
        // Coalesced onto the animation frame — the signal drives a transform
        // that can only change once per painted frame, so writing it per scroll
        // event just re-rendered the hero for output nobody could see.
        let _parallax: Rc<RefCell<Option<FrameListenerGuard>>> = use_hook(move || {
            if prefers_reduced_motion() {
                return Rc::new(RefCell::new(None));
            }
            Rc::new(RefCell::new(add_window_listener_per_frame(
                "scroll",
                move || {
                    scroll.set(scroll_y().min(500.0));
                },
            )))
        });

        // Wheel-hijack: a single wheel notch fully transitions the intro <->
        // about boundary. CSS scroll-snap can't commit a small scroll across a
        // whole viewport, so we hijack the wheel there, smooth-scroll to the
        // target and lock briefly so the trailing momentum does not immediately
        // re-trigger it. Reduced-motion off.
        //
        // While locked the event is deliberately *not* cancelled: doing so froze
        // the page for the whole 900 ms, so a reader who wanted to keep going —
        // or turn straight back — found their input silently dropped. The lock
        // now only suppresses another snap; ordinary scrolling continues.
        let about_target = about_id.clone();
        let lock: Rc<RefCell<bool>> = use_hook(|| Rc::new(RefCell::new(false)));
        let _wheel: Rc<RefCell<Option<ListenerGuard>>> = use_hook(move || {
            if prefers_reduced_motion() {
                return Rc::new(RefCell::new(None));
            }
            Rc::new(RefCell::new(add_window_listener(
                "wheel",
                false,
                move |e| {
                    if *lock.borrow() {
                        return;
                    }
                    let Some(wheel) = e.dyn_ref::<web_sys::WheelEvent>() else {
                        return;
                    };
                    let y = scroll_y();
                    let vh = viewport_height();
                    let dy = wheel.delta_y();
                    let target = if dy > 0.0 && y < vh * 0.5 {
                        Some(about_target.clone())
                    } else if dy < 0.0 && y > vh * 0.5 && y < vh * 1.3 {
                        Some("top".to_string())
                    } else {
                        None
                    };
                    if let Some(id) = target {
                        e.prevent_default();
                        *lock.borrow_mut() = true;
                        scroll_to(&id);
                        let lock = lock.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            gloo_timers::future::TimeoutFuture::new(900).await;
                            *lock.borrow_mut() = false;
                        });
                    }
                },
            )))
        });
    }

    // Parallax transform; identity (`translateY(0px)`) on the server and under
    // reduced motion, so it never diverges from the SSR markup. The `+ 0.0`
    // normalises IEEE negative zero so the resting value renders as `0px`.
    let name_offset = scroll() * -0.08 + 0.0;

    rsx! {
        section { id: "top", class: "hero",
            div { class: "hero-eyebrow",
                div { class: "bracket-line" }
                span { class: "mono text-accent", {section_label("identity")} }
                span { class: "mono text-muted", {t("hero.eyebrow")} }

                div { class: "hero-meta",
                    div { class: "hero-meta-card",
                        span { class: "mono text-muted", "§ {identity_num}.a" }
                        dl { class: "meta-dl",
                            dt { span { class: "mono text-muted", "ROLE" } }
                            dd { {t("hero.jobTitle")} }
                            dt { span { class: "mono text-muted", "LOC" } }
                            dd { {t("common.country")} }
                            dt { span { class: "mono text-muted", "YRS" } }
                            dd { "{years}+" }
                            dt { span { class: "mono text-muted", {t("hero.statusLabel")} } }
                            dd { class: "text-accent flex items-center gap-2",
                                span { class: "pulse-dot" }
                                {t("hero.status")}
                            }
                        }
                    }
                }
            }

            h1 {
                class: "hero-name",
                style: "transform: translateY({name_offset}px)",
                {hero_name_lines()}
            }

            div { class: "hero-tagline",
                div { class: "tagline-label",
                    span { class: "mono text-muted", "§ {identity_num}.b" }
                }
                div { class: "tagline-body",
                    p { class: "tagline-main", {t("hero.tagline")} }
                    p { class: "tagline-sub", {t("hero.taglineSub")} }
                }
            }

            a { href: "#{about_id}", class: "scroll-cue", "aria-label": "Scroll",
                span { class: "mono text-muted", "SCROLL" }
                span { class: "scroll-cue-line" }
            }
        }
    }
}
