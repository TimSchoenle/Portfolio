//! Identity hero: eyebrow meta card, oversized name, tagline, scroll cue.
//!
//! The name parallax is a CSS scroll-driven animation (`input.css`). The hero ↔ about hand-off
//! is a client-only wheel handler: one wheel notch at the boundary carries the reader the whole
//! way across it, which CSS `scroll-snap` cannot do.

use dioxus::prelude::*;
use portfolio_data::{CONFIG, EXPERIENCE};

use crate::clock::{current_month, current_year};
use crate::i18n::use_i18n;
use crate::sections::{section_id, section_label, section_num};

/// Returns the full years since the earliest experience entry started, or `0` when there is
/// none.
fn years_of_experience() -> u32 {
    let Some(earliest) = EXPERIENCE
        .iter()
        .min_by_key(|e| (e.start.year, e.start.month))
    else {
        return 0;
    };
    let mut years = current_year() - i32::from(earliest.start.year);
    if current_month() < earliest.start.month {
        years -= 1;
    }
    u32::try_from(years).unwrap_or(0)
}

/// Renders the full name as two hero lines, coloring "ö" and the trailing dot.
///
/// The first whitespace-separated token is the first line, and everything after it forms the
/// second, so a middle name or a multi-word family name is kept.
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

/// Where one wheel notch at the hero ↔ about boundary carries the reader.
#[cfg(any(feature = "web", test))]
#[derive(Debug, PartialEq, Eq)]
enum BoundarySnap {
    /// Down from the upper half of the hero, to the about section.
    ToAbout,
    /// Up from the about section's opening screen, back to the hero.
    ToHero,
}

/// One wheel event, as the boundary snap reads it.
#[cfg(any(feature = "web", test))]
struct WheelStep {
    /// The page's vertical scroll offset in CSS pixels.
    scroll_y: f64,
    /// The viewport height in CSS pixels.
    viewport_height: f64,
    /// The about section's top edge in document coordinates, in CSS pixels.
    about_top: f64,
    /// The wheel's vertical delta: positive scrolls down, `0.0` is a horizontal step.
    delta_y: f64,
    /// Whether ctrl was held, which makes the step a zoom or a trackpad pinch.
    ctrl: bool,
    /// Whether the reader asks for reduced motion, read per step so a change after load applies.
    reduced_motion: bool,
    /// Whether a modal dialog such as the command palette is open, so the step belongs to it.
    modal_open: bool,
}

/// Decides which wheel steps snap across the boundary, and holds off another snap until
/// [`release`](Self::release).
///
/// Down snaps only from the upper half of the first viewport. Up snaps only from half a viewport
/// to 0.3 viewports past the about section's top, so a reader further down the page scrolls freely
/// and a hero taller than the viewport, as in a short window, still hands back.
// Kept apart from the listener so the decision is testable without a browser, like `FrameSlot`
// in `hooks.rs`.
#[cfg(any(feature = "web", test))]
#[derive(Default)]
struct SnapGate {
    locked: std::cell::Cell<bool>,
}

#[cfg(any(feature = "web", test))]
impl SnapGate {
    /// Returns the snap `step` triggers and locks the gate, or `None` while locked, for a
    /// ctrl-wheel, under reduced motion, while a modal is open, and for ordinary scrolling away
    /// from the boundary.
    fn try_snap(&self, step: &WheelStep) -> Option<BoundarySnap> {
        if self.locked.get() || step.ctrl || step.reduced_motion || step.modal_open {
            return None;
        }
        let half = step.viewport_height * 0.5;
        let snap = if step.delta_y > 0.0 && step.scroll_y < half {
            BoundarySnap::ToAbout
        } else if step.delta_y < 0.0
            && step.scroll_y > half
            && step.scroll_y < step.about_top + step.viewport_height * 0.3
        {
            BoundarySnap::ToHero
        } else {
            return None;
        };
        self.locked.set(true);
        Some(snap)
    }

    /// Lets the next wheel step snap again.
    fn release(&self) {
        self.locked.set(false);
    }
}

/// How long after a snap another one is suppressed, so the trailing momentum of the same gesture
/// does not immediately snap back.
#[cfg(feature = "web")]
const SNAP_LOCK_MS: u32 = 900;

/// Registers the wheel handler that snaps across the hero ↔ about boundary, for the component's
/// lifetime.
///
/// While locked the event is not cancelled: cancelling it would freeze the page for the whole
/// lock and drop the input of a reader who wants to keep going or turn straight back. The lock
/// only suppresses another snap.
#[cfg(feature = "web")]
fn use_boundary_snap(about_id: String) {
    use crate::hooks::{
        ListenerGuard, add_window_listener, document_top, modal_open, prefers_reduced_motion,
        scroll_to, scroll_y, viewport_height,
    };
    use std::rc::Rc;
    use web_sys::wasm_bindgen::JsCast;

    let _wheel: Rc<Option<ListenerGuard>> = use_hook(move || {
        let gate = Rc::new(SnapGate::default());
        let on_wheel = move |e: web_sys::Event| {
            let Some(wheel) = e.dyn_ref::<web_sys::WheelEvent>() else {
                return;
            };
            let (scroll_y, viewport_height) = (scroll_y(), viewport_height());
            let step = WheelStep {
                scroll_y,
                viewport_height,
                about_top: document_top(&about_id).unwrap_or(viewport_height),
                delta_y: wheel.delta_y(),
                ctrl: wheel.ctrl_key(),
                reduced_motion: prefers_reduced_motion(),
                modal_open: modal_open(),
            };
            let Some(snap) = gate.try_snap(&step) else {
                return;
            };
            e.prevent_default();
            scroll_to(match snap {
                BoundarySnap::ToAbout => &about_id,
                BoundarySnap::ToHero => "top",
            });
            let gate = gate.clone();
            wasm_bindgen_futures::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(SNAP_LOCK_MS).await;
                gate.release();
            });
        };
        // Not passive: the handler cancels the event that triggers a snap.
        Rc::new(add_window_listener("wheel", false, on_wheel))
    });
}

/// Renders the hero, with the years-of-experience figure derived from [`EXPERIENCE`] and the
/// current date.
#[component]
pub fn Hero() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let years = years_of_experience();
    let identity_num = section_num("identity");
    let about_id = section_id("about");

    #[cfg(feature = "web")]
    use_boundary_snap(about_id.clone());

    rsx! {
        section { id: "top", class: "hero",
            div { class: "hero-eyebrow",
                div { class: "bracket-line" }
                span { class: "mono text-accent", {section_label("identity")} }
                // Not a translation key: a tool's name is the same in both languages, and when
                // this was one the two files held identical values that could still drift.
                span { class: "mono text-muted", {CONFIG.headline_tech.join(" · ")} }

                div { class: "hero-meta",
                    div { class: "hero-meta-card",
                        span { class: "mono text-muted", "§ {identity_num}.a" }
                        dl { class: "meta-dl",
                            dt { span { class: "mono text-muted", {t("hero.roleLabel")} } }
                            dd { {t("common.jobTitle")} }
                            dt { span { class: "mono text-muted", {t("hero.locationLabel")} } }
                            dd { {t("common.country")} }
                            dt { span { class: "mono text-muted", {t("hero.yearsLabel")} } }
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

            // The parallax on the name is a CSS scroll-driven animation (`.hero-name` in
            // `input.css`): it runs on the compositor, costs the wasm client nothing, and is off
            // under reduced motion and in browsers without scroll timelines.
            h1 { class: "hero-name", {hero_name_lines()} }

            div { class: "hero-tagline",
                div { class: "tagline-label",
                    span { class: "mono text-muted", "§ {identity_num}.b" }
                }
                div { class: "tagline-body",
                    p { class: "tagline-main", {t("hero.tagline")} }
                    p { class: "tagline-sub", {t("hero.taglineSub")} }
                }
            }

            a { href: "#{about_id}", class: "scroll-cue", "aria-label": t("hero.scrollLabel"),
                span { class: "mono text-muted", {t("hero.scroll")} }
                span { class: "scroll-cue-line" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BoundarySnap, SnapGate, WheelStep};

    const VH: f64 = 800.0;

    fn step(scroll_y: f64, delta_y: f64) -> WheelStep {
        WheelStep {
            scroll_y,
            viewport_height: VH,
            // A hero exactly one viewport tall.
            about_top: VH,
            delta_y,
            ctrl: false,
            reduced_motion: false,
            modal_open: false,
        }
    }

    /// Runs `step` through a fresh, unlocked gate.
    fn snap(scroll_y: f64, delta_y: f64) -> Option<BoundarySnap> {
        SnapGate::default().try_snap(&step(scroll_y, delta_y))
    }

    #[test]
    fn the_first_notch_down_from_the_hero_snaps_to_about() {
        assert_eq!(snap(0.0, 100.0), Some(BoundarySnap::ToAbout));
        assert_eq!(snap(399.0, 1.0), Some(BoundarySnap::ToAbout));
    }

    #[test]
    fn the_first_notch_up_from_about_snaps_to_the_hero() {
        assert_eq!(snap(VH, -100.0), Some(BoundarySnap::ToHero));
        assert_eq!(snap(401.0, -1.0), Some(BoundarySnap::ToHero));
    }

    #[test]
    fn scrolling_further_down_the_page_never_snaps() {
        assert_eq!(snap(VH * 1.3, -100.0), None);
        assert_eq!(snap(VH * 3.0, -100.0), None);
        assert_eq!(snap(VH, 100.0), None);
        assert_eq!(snap(VH * 3.0, 100.0), None);
    }

    /// In a short window the hero outgrows the viewport, so the about section starts well below it.
    #[test]
    fn a_hero_taller_than_the_viewport_still_snaps_back_up() {
        // About starts at 1600 px, so up snaps until 1600 + 0.3 * 800 = 1840 px.
        let tall = |scroll_y| WheelStep {
            about_top: 1600.0,
            ..step(scroll_y, -100.0)
        };
        let gate = || SnapGate::default();
        assert_eq!(gate().try_snap(&tall(1528.0)), Some(BoundarySnap::ToHero));
        assert_eq!(gate().try_snap(&tall(1839.0)), Some(BoundarySnap::ToHero));
        assert_eq!(gate().try_snap(&tall(1840.0)), None);
    }

    #[test]
    fn horizontal_steps_and_up_at_the_top_never_snap() {
        assert_eq!(snap(0.0, 0.0), None);
        assert_eq!(snap(VH, 0.0), None);
        assert_eq!(snap(0.0, -100.0), None);
    }

    #[test]
    fn a_ctrl_wheel_zooms_instead_of_snapping() {
        let zoom = WheelStep {
            ctrl: true,
            ..step(0.0, 100.0)
        };
        assert_eq!(SnapGate::default().try_snap(&zoom), None);
    }

    #[test]
    fn reduced_motion_never_snaps() {
        let step = WheelStep {
            reduced_motion: true,
            ..step(0.0, 100.0)
        };
        assert_eq!(SnapGate::default().try_snap(&step), None);
    }

    /// A wheel over the open command palette scrolls its results, not the page.
    #[test]
    fn an_open_modal_never_snaps() {
        let step = WheelStep {
            modal_open: true,
            ..step(VH, -100.0)
        };
        assert_eq!(SnapGate::default().try_snap(&step), None);
    }

    /// The momentum of the gesture that snapped must not snap straight back.
    #[test]
    fn a_snap_locks_the_gate_until_it_is_released() {
        let gate = SnapGate::default();
        assert_eq!(
            gate.try_snap(&step(0.0, 100.0)),
            Some(BoundarySnap::ToAbout)
        );
        assert_eq!(
            gate.try_snap(&step(VH, -100.0)),
            None,
            "a locked gate snapped"
        );

        gate.release();
        assert_eq!(gate.try_snap(&step(VH, -100.0)), Some(BoundarySnap::ToHero));
    }

    /// Only a snap takes the lock, so ordinary scrolling never blocks the next one.
    #[test]
    fn a_step_that_does_not_snap_leaves_the_gate_open() {
        let gate = SnapGate::default();
        assert_eq!(gate.try_snap(&step(VH * 3.0, 100.0)), None);
        assert_eq!(
            gate.try_snap(&step(0.0, 100.0)),
            Some(BoundarySnap::ToAbout)
        );
    }
}
