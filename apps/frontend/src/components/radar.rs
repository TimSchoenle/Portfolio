//! Skill radar from the v4 design (radar-v3): four quadrants with one named,
//! hoverable dot per skill plus seeded background filler dots, concentric
//! rings and a rotating sweep wedge.

use i18nrs::yew::use_translation;
use portfolio_data::{Quadrant, SKILLS, Skill, matrix_skills};
use yew::prelude::*;

const SIZE: f32 = 480.0;
const CX: f32 = SIZE / 2.0;
const CY: f32 = SIZE / 2.0;
const MAX_R: f32 = 200.0;

/// Start angle (degrees) of each quadrant arc, matching the design:
/// languages top-left, frameworks top-right, build bottom-right, infra bottom-left.
fn quadrant_arc(q: Quadrant) -> (f32, f32) {
    match q {
        Quadrant::Languages => (180.0, 270.0),
        Quadrant::Frameworks => (270.0, 360.0),
        Quadrant::Build => (0.0, 90.0),
        Quadrant::Infra => (90.0, 180.0),
    }
}

/// The design seeds its PRNG from the category's JS key.
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
    /// True when a named (highlighted) dot sits close enough that it should win
    /// the hover; such filler dots ignore pointer events so the named dot wins.
    near_named: bool,
}

fn polar(angle_deg: f32, r: f32) -> (f32, f32) {
    let rad = angle_deg.to_radians();
    (CX + r * rad.cos(), CY + r * rad.sin())
}

/// Maps a skill's confidence (0..=1) onto a normalized radar radius so the dot
/// sits at the matching percentage mark: 0.95 confidence lands near the rim,
/// low confidence stays close to the centre. A touch of seeded jitter keeps
/// dots from stacking on a perfect ring.
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

        // Radar-only skills scatter as the secondary dots: the same
        // confidence-based radius puts them at their true mark, but they are
        // smaller, dimmer and hoverable in their own right.
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

    // A highlighted dot always wins the hover: flag any filler dot that overlaps
    // a named one so it lets pointer events fall through to the named dot.
    for f in filler.iter_mut() {
        f.near_named = named.iter().any(|n| {
            n.skill.quadrant == f.skill.quadrant
                && (n.x - f.x).hypot(n.y - f.y) <= n.size + f.size + 2.0
        });
    }

    (named, filler)
}

#[derive(Properties, PartialEq)]
pub struct RadarProps {
    pub active: Option<Quadrant>,
    pub on_hover: Callback<Option<Skill>>,
}

#[function_component(Radar)]
pub fn radar(p: &RadarProps) -> Html {
    let (i18n, _) = use_translation();
    let hovered = use_state(|| None::<usize>);
    let hovered_filler = use_state(|| None::<usize>);
    let (named, filler) = build_dots();

    let label = |q: Quadrant| i18n.t(q.i18n_key()).to_uppercase();
    let label_opacity = |q: Quadrant| {
        if p.active.is_none() || p.active == Some(q) {
            "1"
        } else {
            "0.3"
        }
    };

    // Sweep wedge: 30° arc trailing the rotating line.
    let wedge_end = polar(-30.0, MAX_R);

    html! {
        <div class="radar-wrap">
            <svg viewBox={format!("0 0 {SIZE} {SIZE}")} class="radar-svg">
                <defs>
                    <radialGradient id="radarGlow" cx="50%" cy="50%" r="50%">
                        <stop offset="0%" stop-color="rgba(96,165,250,0.06)" />
                        <stop offset="100%" stop-color="rgba(96,165,250,0)" />
                    </radialGradient>
                    <linearGradient id="sweepGrad" x1="0%" y1="50%" x2="100%" y2="50%">
                        <stop offset="0%" stop-color="var(--accent)" stop-opacity="0" />
                        <stop offset="100%" stop-color="var(--accent)" stop-opacity="0.55" />
                    </linearGradient>
                </defs>

                // glow
                <circle cx={CX.to_string()} cy={CY.to_string()} r={MAX_R.to_string()} fill="url(#radarGlow)" />

                // concentric rings
                { for [0.3_f32, 0.5, 0.7, 0.9].iter().enumerate().map(|(i, f)| html!{
                    <circle cx={CX.to_string()} cy={CY.to_string()} r={(MAX_R * f).to_string()}
                            fill="none" stroke="rgba(200,220,255,0.08)" stroke-width="0.6"
                            stroke-dasharray={ if i == 3 { "" } else { "2 3" } } />
                })}
                <circle cx={CX.to_string()} cy={CY.to_string()} r={MAX_R.to_string()}
                        fill="none" stroke="rgba(200,220,255,0.14)" stroke-width="0.8" />

                // axes
                <line x1={CX.to_string()} y1={(CY - MAX_R).to_string()}
                      x2={CX.to_string()} y2={(CY + MAX_R).to_string()}
                      stroke="rgba(200,220,255,0.1)" stroke-width="0.6" />
                <line x1={(CX - MAX_R).to_string()} y1={CY.to_string()}
                      x2={(CX + MAX_R).to_string()} y2={CY.to_string()}
                      stroke="rgba(200,220,255,0.1)" stroke-width="0.6" />

                // rotating sweep
                <g class="radar-sweep-g" style={format!("transform-origin: {CX}px {CY}px")}>
                    <path d={format!("M {CX} {CY} L {} {CY} A {MAX_R} {MAX_R} 0 0 0 {} {} Z",
                                     CX + MAX_R, wedge_end.0, wedge_end.1)}
                          fill="url(#sweepGrad)" opacity="0.7" />
                    <line x1={CX.to_string()} y1={CY.to_string()}
                          x2={(CX + MAX_R).to_string()} y2={CY.to_string()}
                          stroke="var(--accent)" stroke-width="1.3" opacity="0.55" />
                </g>

                // quadrant labels
                <text x={(CX - MAX_R + 8.0).to_string()} y={(CY - MAX_R + 16.0).to_string()}
                      fill={Quadrant::Languages.color()} font-size="10"
                      font-family="JetBrains Mono, monospace" letter-spacing="0.14em"
                      opacity={label_opacity(Quadrant::Languages)}>
                    { label(Quadrant::Languages) }
                </text>
                <text x={(CX + MAX_R - 90.0).to_string()} y={(CY - MAX_R + 16.0).to_string()}
                      fill={Quadrant::Frameworks.color()} font-size="10"
                      font-family="JetBrains Mono, monospace" letter-spacing="0.14em"
                      opacity={label_opacity(Quadrant::Frameworks)}>
                    { label(Quadrant::Frameworks) }
                </text>
                <text x={(CX + MAX_R - 100.0).to_string()} y={(CY + MAX_R - 6.0).to_string()}
                      fill={Quadrant::Build.color()} font-size="10"
                      font-family="JetBrains Mono, monospace" letter-spacing="0.14em"
                      opacity={label_opacity(Quadrant::Build)}>
                    { label(Quadrant::Build) }
                </text>
                <text x={(CX - MAX_R + 8.0).to_string()} y={(CY + MAX_R - 6.0).to_string()}
                      fill={Quadrant::Infra.color()} font-size="10"
                      font-family="JetBrains Mono, monospace" letter-spacing="0.14em"
                      opacity={label_opacity(Quadrant::Infra)}>
                    { label(Quadrant::Infra) }
                </text>

                // radar-only scatter dots: hoverable, but they defer to a nearby
                // highlighted dot so the named skill always wins the hover.
                { for filler.iter().enumerate().map(|(i, d)| {
                    let dim = p.active.is_some() && p.active != Some(d.skill.quadrant);
                    let is_hover = *hovered_filler == Some(i);
                    let enter = {
                        let hovered_filler = hovered_filler.clone();
                        let on_hover = p.on_hover.clone();
                        let skill = d.skill;
                        Callback::from(move |_: MouseEvent| {
                            hovered_filler.set(Some(i));
                            on_hover.emit(Some(skill));
                        })
                    };
                    let leave = {
                        let hovered_filler = hovered_filler.clone();
                        let on_hover = p.on_hover.clone();
                        Callback::from(move |_: MouseEvent| {
                            hovered_filler.set(None);
                            on_hover.emit(None);
                        })
                    };
                    let opacity = if dim {
                        0.15
                    } else if is_hover {
                        0.95
                    } else {
                        d.opacity
                    };
                    html!{
                        <circle cx={d.x.to_string()} cy={d.y.to_string()}
                                r={ if is_hover { (d.size + 1.5).to_string() } else { d.size.to_string() } }
                                fill={d.skill.quadrant.color()}
                                opacity={format!("{opacity:.2}")}
                                style={ if d.near_named { "pointer-events: none" } else { "cursor: pointer" } }
                                onmouseenter={enter} onmouseleave={leave} />
                    }
                })}

                // named, hoverable dots
                { for named.iter().enumerate().map(|(i, d)| {
                    let dim = p.active.is_some() && p.active != Some(d.skill.quadrant);
                    let is_hover = *hovered == Some(i);
                    let enter = {
                        let hovered = hovered.clone();
                        let on_hover = p.on_hover.clone();
                        let skill = d.skill;
                        Callback::from(move |_: MouseEvent| {
                            hovered.set(Some(i));
                            on_hover.emit(Some(skill));
                        })
                    };
                    let leave = {
                        let hovered = hovered.clone();
                        let on_hover = p.on_hover.clone();
                        Callback::from(move |_: MouseEvent| {
                            hovered.set(None);
                            on_hover.emit(None);
                        })
                    };
                    html!{
                        <g onmouseenter={enter} onmouseleave={leave}
                           style="cursor: pointer"
                           opacity={ if dim { "0.15" } else { "1" } }>
                            <circle cx={d.x.to_string()} cy={d.y.to_string()}
                                    r={ if is_hover { (d.size + 3.0).to_string() } else { d.size.to_string() } }
                                    fill={d.skill.quadrant.color()}
                                    stroke={ if is_hover { "var(--fg)" } else { "none" } }
                                    stroke-width="1.3" />
                            if p.active.is_none() {
                                <circle cx={d.x.to_string()} cy={d.y.to_string()}
                                        r={(d.size + 3.0).to_string()} fill="none"
                                        stroke={d.skill.quadrant.color()} stroke-width="0.6" opacity="0.35" />
                            }
                        </g>
                    }
                })}

                <circle cx={CX.to_string()} cy={CY.to_string()} r="2" fill="var(--accent)" />
            </svg>
        </div>
    }
}
