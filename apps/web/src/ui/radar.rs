//! Skill radar (radar-v3): four quadrants with named, hoverable dots plus
//! seeded filler dots, concentric rings and a rotating sweep wedge.

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
    (key.as_bytes()[0] as f64) * 131.0 + (key.len() as f64) * 17.0
}

/// Deterministic PRNG from the design: s = (s * 9301 + 49297) % 233280.
struct Seeded(f64);

impl Seeded {
    fn next(&mut self) -> f32 {
        self.0 = (self.0 * 9301.0 + 49297.0) % 233280.0;
        (self.0 / 233280.0) as f32
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
                size: 5.0 + skill.level() as f32 * 0.4,
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
                size: 2.4 + skill.level() as f32 * 0.3,
                opacity: 0.35 + skill.confidence * 0.4,
                skill,
                near_named: false,
            });
        }
    }

    for f in filler.iter_mut() {
        f.near_named = named.iter().any(|n| {
            n.skill.quadrant == f.skill.quadrant
                && (n.x - f.x).hypot(n.y - f.y) <= n.size + f.size + 2.0
        });
    }

    (named, filler)
}

#[component]
pub fn Radar(active: Option<Quadrant>, on_hover: EventHandler<Option<Skill>>) -> Element {
    let i18n = use_i18n().i18n;
    let mut hovered = use_signal(|| None::<usize>);
    let mut hovered_filler = use_signal(|| None::<usize>);
    let (named, filler) = build_dots();

    let label = move |q: Quadrant| i18n.read().t(q.i18n_key()).to_uppercase();
    let label_opacity = move |q: Quadrant| {
        if active.is_none() || active == Some(q) {
            "1"
        } else {
            "0.3"
        }
    };

    let wedge_end = polar(-30.0, MAX_R);
    let view_box = format!("0 0 {SIZE} {SIZE}");
    let sweep_path = format!(
        "M {CX} {CY} L {} {CY} A {MAX_R} {MAX_R} 0 0 0 {} {} Z",
        CX + MAX_R,
        wedge_end.0,
        wedge_end.1
    );

    rsx! {
        div { class: "radar-wrap",
            svg { "viewBox": "{view_box}", class: "radar-svg",
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

                {filler.iter().enumerate().map(|(i, d)| {
                    let dim = active.is_some() && active != Some(d.skill.quadrant);
                    let is_hover = hovered_filler() == Some(i);
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
                            onmouseenter: move |_| { hovered_filler.set(Some(i)); on_hover.call(Some(skill)); },
                            onmouseleave: move |_| { hovered_filler.set(None); on_hover.call(None); },
                        }
                    }
                })}

                {named.iter().enumerate().map(|(i, d)| {
                    let dim = active.is_some() && active != Some(d.skill.quadrant);
                    let is_hover = hovered() == Some(i);
                    let skill = d.skill;
                    let color = d.skill.quadrant.color();
                    let r = if is_hover { d.size + 3.0 } else { d.size };
                    let stroke = if is_hover { "var(--fg)" } else { "none" };
                    let g_opacity = if dim { "0.15" } else { "1" };
                    let ring = MAX_R; // unused placeholder to keep layout parity
                    let _ = ring;
                    rsx! {
                        g {
                            key: "n{i}",
                            style: "cursor: pointer", opacity: "{g_opacity}",
                            onmouseenter: move |_| { hovered.set(Some(i)); on_hover.call(Some(skill)); },
                            onmouseleave: move |_| { hovered.set(None); on_hover.call(None); },
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

                circle { cx: "{CX}", cy: "{CY}", r: "2", fill: "var(--accent)" }
            }
        }
    }
}
