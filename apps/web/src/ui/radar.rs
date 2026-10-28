//! Skill radar (radar-v3): four quadrants with named, hoverable dots plus
//! seeded filler dots, concentric rings and a rotating sweep wedge.

use std::sync::LazyLock;

use dioxus::prelude::*;
use portfolio_data::{Quadrant, SKILLS, Skill, matrix_skills};

use crate::i18n::use_i18n;

const SIZE: f32 = 480.0;
const CX: f32 = SIZE / 2.0;
const CY: f32 = SIZE / 2.0;
const MAX_R: f32 = 200.0;

/// Start/end angle (degrees) of each quadrant arc.
fn quadrant_arc(q: Quadrant) -> (f32, f32) {
    match q {
        Quadrant::Languages => (180.0, 270.0),
        Quadrant::Frameworks => (270.0, 360.0),
        Quadrant::Build => (0.0, 90.0),
        Quadrant::Infra => (90.0, 180.0),
    }
}

/// The design seeds its PRNG from the category's key.
fn quadrant_seed(q: Quadrant) -> f64 {
    let key = match q {
        Quadrant::Languages => "languages",
        Quadrant::Frameworks => "frameworks",
        Quadrant::Build => "build",
        Quadrant::Infra => "infra",
    };
    // Every key is a short literal, so its length fits a `u8` and converts losslessly.
    let len = u8::try_from(key.len()).expect("quadrant keys are short");
    f64::from(key.as_bytes()[0]) * 131.0 + f64::from(len) * 17.0
}

/// Deterministic PRNG from the design: s = (s * 9301 + 49297) % 233280.
struct Seeded(f64);

impl Seeded {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a value in [0, 1) only loses precision it does not need for a jitter"
    )]
    fn next(&mut self) -> f32 {
        self.0 = (self.0 * 9_301.0 + 49_297.0) % 233_280.0;
        (self.0 / 233_280.0) as f32
    }
}

struct NamedDot {
    x: f32,
    y: f32,
    size: f32,
    skill: Skill,
}

struct FillerDot {
    x: f32,
    y: f32,
    size: f32,
    opacity: f32,
    skill: Skill,
    near_named: bool,
}

fn polar(angle_deg: f32, r: f32) -> (f32, f32) {
    let rad = angle_deg.to_radians();
    (CX + r * rad.cos(), CY + r * rad.sin())
}

fn radius_for(confidence: f32, jitter: f32) -> f32 {
    (confidence + (jitter - 0.5) * 0.04).clamp(0.05, 0.97)
}

/// The dot layout, a pure function of compile-time data, computed once per process.
///
/// Recomputing it per render would re-run `matrix_skills()`, the layout of ~170 dots and an
/// O(named x filler) proximity pass on every pointer move, since those write the hover signals.
static DOTS: LazyLock<(Vec<NamedDot>, Vec<FillerDot>)> = LazyLock::new(build_dots);

fn build_dots() -> (Vec<NamedDot>, Vec<FillerDot>) {
    let skills = matrix_skills();
    let mut named = Vec::new();
    let mut filler = Vec::new();

    for q in Quadrant::all() {
        let (a0, a1) = quadrant_arc(q);
        let span = a1 - a0;
        let mut rnd = Seeded(quadrant_seed(q));
        let items: Vec<Skill> = skills.iter().filter(|s| s.quadrant == q).copied().collect();

        for (i, skill) in items.iter().enumerate() {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a quadrant holds a handful of skills, far below f32's exact-integer range"
            )]
            let t = if items.len() > 1 {
                i as f32 / (items.len() - 1) as f32
            } else {
                0.5
            };
            let angle = a0 + 10.0 + t * (span - 20.0);
            let r_norm = radius_for(skill.confidence, rnd.next());
            let (x, y) = polar(angle, MAX_R * r_norm);
            named.push(NamedDot {
                x,
                y,
                size: 5.0 + f32::from(skill.level()) * 0.4,
                skill: *skill,
            });
        }

        let extras: Vec<Skill> = SKILLS
            .iter()
            .filter(|s| s.radar_only && s.quadrant == q)
            .copied()
            .collect();
        for skill in extras {
            let angle = a0 + 6.0 + rnd.next() * (span - 12.0);
            let r_norm = radius_for(skill.confidence, rnd.next());
            let (x, y) = polar(angle, MAX_R * r_norm);
            filler.push(FillerDot {
                x,
                y,
                size: 2.4 + f32::from(skill.level()) * 0.3,
                opacity: 0.35 + skill.confidence * 0.4,
                skill,
                near_named: false,
            });
        }
    }

    for f in &mut filler {
        f.near_named = named.iter().any(|n| {
            n.skill.quadrant == f.skill.quadrant
                && (n.x - f.x).hypot(n.y - f.y) <= n.size + f.size + 2.0
        });
    }

    (named, filler)
}

/// The radar itself: named dots for real skills, seeded filler dots for texture.
///
/// `active` dims every quadrant but one; `None` shows all four at full strength. `on_hover`
/// fires with the skill under the cursor and with `None` when it leaves, so the caller owns the
/// tooltip and this component owns only the geometry.
#[component]
pub fn Radar(active: Option<Quadrant>, on_hover: EventHandler<Option<Skill>>) -> Element {
    let i18n = use_i18n().i18n;
    let view_box = format!("0 0 {SIZE} {SIZE}");
    let radar_label = i18n.read().t("skills.radarLabel");

    rsx! {
        div { class: "radar-wrap",
            svg {
                "viewBox": "{view_box}",
                class: "radar-svg",
                // Announced as a single labeled image: the rings, sweep and
                // scatter carry no information on their own, and every skill
                // plotted here is also reachable as text in the adjacent list.
                role: "img",
                "aria-label": "{radar_label}",
                RadarBackdrop {}
                QuadrantLabels { active }
                FillerDots { active, on_hover }
                NamedDots { active, on_hover }
                circle { cx: "{CX}", cy: "{CY}", r: "2", fill: "var(--accent)" }
            }
        }
    }
}

/// The parts of the radar that carry no data: the glow, the rings, the axes and the sweep.
#[component]
fn RadarBackdrop() -> Element {
    let wedge_end = polar(-30.0, MAX_R);
    let sweep_path = format!(
        "M {CX} {CY} L {} {CY} A {MAX_R} {MAX_R} 0 0 0 {} {} Z",
        CX + MAX_R,
        wedge_end.0,
        wedge_end.1
    );
    rsx! {
        defs {
            radialGradient { id: "radarGlow", cx: "50%", cy: "50%", r: "50%",
                stop { offset: "0%", "stop-color": "rgba(96,165,250,0.06)" }
                stop { offset: "100%", "stop-color": "rgba(96,165,250,0)" }
            }
            linearGradient { id: "sweepGrad", x1: "0%", y1: "50%", x2: "100%", y2: "50%",
                stop { offset: "0%", "stop-color": "var(--accent)", "stop-opacity": "0" }
                stop { offset: "100%", "stop-color": "var(--accent)", "stop-opacity": "0.55" }
            }
        }

        circle { cx: "{CX}", cy: "{CY}", r: "{MAX_R}", fill: "url(#radarGlow)" }

        {[0.3_f32, 0.5, 0.7, 0.9].iter().enumerate().map(|(i, f)| {
            let r = MAX_R * f;
            let dash = if i == 3 { "" } else { "2 3" };
            rsx! {
                circle {
                    key: "{i}",
                    cx: "{CX}", cy: "{CY}", r: "{r}",
                    fill: "none", stroke: "rgba(200,220,255,0.08)",
                    "stroke-width": "0.6", "stroke-dasharray": "{dash}",
                }
            }
        })}
        circle {
            cx: "{CX}", cy: "{CY}", r: "{MAX_R}",
            fill: "none", stroke: "rgba(200,220,255,0.14)", "stroke-width": "0.8",
        }

        line {
            x1: "{CX}", y1: "{CY - MAX_R}", x2: "{CX}", y2: "{CY + MAX_R}",
            stroke: "rgba(200,220,255,0.1)", "stroke-width": "0.6",
        }
        line {
            x1: "{CX - MAX_R}", y1: "{CY}", x2: "{CX + MAX_R}", y2: "{CY}",
            stroke: "rgba(200,220,255,0.1)", "stroke-width": "0.6",
        }

        g { class: "radar-sweep-g", style: "transform-origin: {CX}px {CY}px",
            path { d: "{sweep_path}", fill: "url(#sweepGrad)", opacity: "0.7" }
            line {
                x1: "{CX}", y1: "{CY}", x2: "{CX + MAX_R}", y2: "{CY}",
                stroke: "var(--accent)", "stroke-width": "1.3", opacity: "0.55",
            }
        }
    }
}

/// The four quadrant names, one per corner, dimmed unless their quadrant is `active` or none is.
#[component]
fn QuadrantLabels(active: Option<Quadrant>) -> Element {
    let i18n = use_i18n().i18n;
    let label = move |q: Quadrant| i18n.read().t(q.i18n_key()).to_uppercase();
    let label_opacity = move |q: Quadrant| {
        if active.is_none() || active == Some(q) {
            "1"
        } else {
            "0.3"
        }
    };
    rsx! {
        {Quadrant::all().into_iter().map(|q| {
            let (tx, ty) = match q {
                Quadrant::Languages => (CX - MAX_R + 8.0, CY - MAX_R + 16.0),
                Quadrant::Frameworks => (CX + MAX_R - 90.0, CY - MAX_R + 16.0),
                Quadrant::Build => (CX + MAX_R - 100.0, CY + MAX_R - 6.0),
                Quadrant::Infra => (CX - MAX_R + 8.0, CY + MAX_R - 6.0),
            };
            let color = q.color();
            let op = label_opacity(q);
            let text = label(q);
            rsx! {
                text {
                    key: "{color}",
                    x: "{tx}", y: "{ty}", fill: "{color}", "font-size": "10",
                    "font-family": "JetBrains Mono, monospace", "letter-spacing": "0.14em",
                    opacity: "{op}",
                    "{text}"
                }
            }
        })}
    }
}

/// The seeded scatter of radar-only skills: hoverable for sighted users, hidden from assistive
/// technology.
#[component]
fn FillerDots(active: Option<Quadrant>, on_hover: EventHandler<Option<Skill>>) -> Element {
    let mut hovered = use_signal(|| None::<usize>);
    rsx! {
        {DOTS.1.iter().enumerate().map(|(i, d)| {
            let dim = active.is_some() && active != Some(d.skill.quadrant);
            let is_hover = hovered() == Some(i);
            let skill = d.skill;
            let color = d.skill.quadrant.color();
            let r = if is_hover { d.size + 1.5 } else { d.size };
            let opacity = if dim { 0.15 } else if is_hover { 0.95 } else { d.opacity };
            let style = if d.near_named { "pointer-events: none" } else { "cursor: pointer" };
            rsx! {
                circle {
                    key: "f{i}",
                    cx: "{d.x}", cy: "{d.y}", r: "{r}", fill: "{color}",
                    opacity: "{opacity:.2}", style: "{style}",
                    // Decorative scatter: hovering one is a sighted-only
                    // affordance, and turning 80-odd dots into tab stops
                    // would bury the rest of the page. The labeled
                    // `role="img"` on the svg covers them.
                    "aria-hidden": "true",
                    onmouseenter: move |_| { hovered.set(Some(i)); on_hover.call(Some(skill)); },
                    onmouseleave: move |_| { hovered.set(None); on_hover.call(None); },
                }
            }
        })}
    }
}

/// The matrix skills, one labeled dot each: the radar's interactive layer, reachable by mouse
/// and by keyboard.
#[component]
fn NamedDots(active: Option<Quadrant>, on_hover: EventHandler<Option<Skill>>) -> Element {
    let mut hovered = use_signal(|| None::<usize>);
    rsx! {
        {DOTS.0.iter().enumerate().map(|(i, d)| {
            let dim = active.is_some() && active != Some(d.skill.quadrant);
            let is_hover = hovered() == Some(i);
            let skill = d.skill;
            let color = d.skill.quadrant.color();
            let r = if is_hover { d.size + 3.0 } else { d.size };
            let stroke = if is_hover { "var(--fg)" } else { "none" };
            let g_opacity = if dim { "0.15" } else { "1" };
            // Named dots are the radar's interactive layer, so they are
            // reachable by keyboard: focusing one announces its label
            // and drives the same live-region readout as hovering.
            let dot_label = format!("{} — {}%", skill.name, (skill.confidence * 100.0).round());
            rsx! {
                g {
                    key: "n{i}",
                    style: "cursor: pointer", opacity: "{g_opacity}",
                    tabindex: "0",
                    role: "img",
                    "aria-label": "{dot_label}",
                    onmouseenter: move |_| { hovered.set(Some(i)); on_hover.call(Some(skill)); },
                    onmouseleave: move |_| { hovered.set(None); on_hover.call(None); },
                    onfocus: move |_| { hovered.set(Some(i)); on_hover.call(Some(skill)); },
                    onblur: move |_| { hovered.set(None); on_hover.call(None); },
                    circle {
                        cx: "{d.x}", cy: "{d.y}", r: "{r}", fill: "{color}",
                        stroke: "{stroke}", "stroke-width": "1.3",
                    }
                    if active.is_none() {
                        circle {
                            cx: "{d.x}", cy: "{d.y}", r: "{d.size + 3.0}", fill: "none",
                            stroke: "{color}", "stroke-width": "0.6", opacity: "0.35",
                        }
                    }
                }
            }
        })}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The angle of `(x, y)` around the center, in degrees in `[0, 360)`, measured the way
    /// [`polar`] lays dots out.
    fn angle_of(x: f32, y: f32) -> f32 {
        (y - CY).atan2(x - CX).to_degrees().rem_euclid(360.0)
    }

    #[test]
    fn gives_every_matrix_skill_one_named_dot() {
        let (named, _) = build_dots();
        let mut plotted: Vec<&str> = named.iter().map(|d| d.skill.name).collect();
        let mut expected: Vec<&str> = matrix_skills().iter().map(|s| s.name).collect();
        plotted.sort_unstable();
        expected.sort_unstable();
        assert_eq!(plotted, expected);
    }

    #[test]
    fn scatters_exactly_the_radar_only_skills_as_filler() {
        let (_, filler) = build_dots();
        let mut plotted: Vec<&str> = filler.iter().map(|d| d.skill.name).collect();
        let mut expected: Vec<&str> = SKILLS
            .iter()
            .filter(|s| s.radar_only)
            .map(|s| s.name)
            .collect();
        plotted.sort_unstable();
        expected.sort_unstable();
        assert_eq!(plotted, expected);
    }

    #[test]
    fn keeps_every_dot_inside_its_quadrant_and_inside_the_outer_ring() {
        let (named, filler) = build_dots();
        let dots = named
            .iter()
            .map(|d| (d.x, d.y, d.skill.quadrant))
            .chain(filler.iter().map(|d| (d.x, d.y, d.skill.quadrant)));
        for (x, y, quadrant) in dots {
            let (a0, a1) = quadrant_arc(quadrant);
            let angle = angle_of(x, y);
            // `rem_euclid` maps the Frameworks arc's 360° end to 0°, so compare on the open arc.
            let within = if a1 >= 360.0 {
                angle >= a0 || angle < a1 - 360.0
            } else {
                (a0..a1).contains(&angle)
            };
            assert!(within, "{quadrant:?} dot at {angle}° is outside {a0}..{a1}");
            assert!((x - CX).hypot(y - CY) <= MAX_R * 0.97 + 0.01);
        }
    }

    #[test]
    fn lays_the_dots_out_identically_on_every_build() {
        let (first_named, first_filler) = build_dots();
        let (second_named, second_filler) = build_dots();
        let positions = |named: &[NamedDot], filler: &[FillerDot]| -> Vec<(f32, f32)> {
            named
                .iter()
                .map(|d| (d.x, d.y))
                .chain(filler.iter().map(|d| (d.x, d.y)))
                .collect()
        };
        assert_eq!(
            positions(&first_named, &first_filler),
            positions(&second_named, &second_filler)
        );
    }

    #[test]
    fn clamps_the_radius_to_the_ring_band() {
        assert!((radius_for(0.0, 0.0) - 0.05).abs() < f32::EPSILON);
        assert!((radius_for(1.0, 1.0) - 0.97).abs() < f32::EPSILON);
        assert!((radius_for(0.5, 0.5) - 0.5).abs() < f32::EPSILON);
    }

    #[cfg(feature = "server")]
    mod rendered {
        use super::*;
        use crate::ui::test_render::render;

        #[test]
        fn labels_one_focusable_dot_per_matrix_skill() {
            let html = render("en", || rsx! { Radar { active: None, on_hover: |_| {} } });
            assert_eq!(html.matches(r#"tabindex="0""#).count(), DOTS.0.len());
            for dot in &DOTS.0 {
                assert!(
                    html.contains(dot.skill.name),
                    "{} is missing",
                    dot.skill.name
                );
            }
            assert_eq!(html.matches(r#"aria-hidden="true""#).count(), DOTS.1.len());
        }

        #[test]
        fn dims_every_quadrant_but_the_active_one() {
            let all = render("en", || rsx! { Radar { active: None, on_hover: |_| {} } });
            let one = render("en", || {
                rsx! { Radar { active: Some(Quadrant::Languages), on_hover: |_| {} } }
            });
            // The halo ring marks every named dot only while no quadrant is active.
            assert_eq!(all.matches(r#"opacity="0.35""#).count(), DOTS.0.len());
            assert_eq!(one.matches(r#"opacity="0.35""#).count(), 0);
            // Three of the four corner labels fade while one quadrant is active.
            assert_eq!(
                all.matches(r#"letter-spacing="0.14em" opacity="0.3""#)
                    .count(),
                0
            );
            assert_eq!(
                one.matches(r#"letter-spacing="0.14em" opacity="0.3""#)
                    .count(),
                3
            );
            let others = DOTS
                .0
                .iter()
                .filter(|d| d.skill.quadrant != Quadrant::Languages)
                .count();
            assert_eq!(
                one.matches(r#"opacity="0.15""#).count(),
                others
                    + DOTS
                        .1
                        .iter()
                        .filter(|d| d.skill.quadrant != Quadrant::Languages)
                        .count()
            );
        }
    }
}
