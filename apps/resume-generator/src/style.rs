//! Colors and the scaled typography used throughout the resume.

use genpdf::elements::Break;
use genpdf::style::{Color, Style};

/// Primary accent: conservative navy for the name, headings and the dark end
/// of every gradient — print-safe, one hue family total (recruiter guidance).
pub(crate) const NAVY: Color = Color::Rgb(23, 54, 93);
/// Secondary accent: a lighter steel blue for the job title, category labels
/// and "Stack:" prefixes.
pub(crate) const STEEL: Color = Color::Rgb(64, 102, 154);
/// Dark gray for dates / companies — readable on paper, clearly secondary.
pub(crate) const GRAY: Color = Color::Rgb(90, 98, 110);
/// Soft blue-gray every gradient fades into (close to the paper white).
pub(crate) const SOFT: Color = Color::Rgb(228, 233, 240);

/// Linear interpolation between two RGB colors (`t` in 0.0..=1.0).
pub(crate) fn lerp_color(from: Color, to: Color, t: f64) -> Color {
    match (from, to) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let mix = |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * t) as u8;
            Color::Rgb(mix(r1, r2), mix(g1, g2), mix(b1, b2))
        }
        _ => from,
    }
}

/// Scaled typography (bases follow resume-typography guidance: name 20 pt,
/// headings 11.5 pt, body 10 pt, secondary 9.5 pt, sidebar ~9 pt).
/// Everything shrinks together so the hierarchy survives the
/// fit-to-one-page reduction.
#[derive(Clone, Copy)]
pub(crate) struct Layout {
    pub(crate) scale: f64,
}

impl Layout {
    pub(crate) fn pt(&self, base: f64) -> u8 {
        ((base * self.scale).round() as u8).max(7)
    }
    pub(crate) fn name(&self) -> Style {
        Style::new()
            .bold()
            .with_font_size(self.pt(20.0))
            .with_color(NAVY)
    }
    pub(crate) fn title(&self) -> Style {
        Style::new().with_font_size(self.pt(11.0)).with_color(STEEL)
    }
    pub(crate) fn contact(&self) -> Style {
        Style::new().with_font_size(self.pt(8.8)).with_color(GRAY)
    }
    pub(crate) fn heading(&self) -> Style {
        Style::new()
            .bold()
            .with_font_size(self.pt(11.5))
            .with_color(NAVY)
    }
    pub(crate) fn body(&self) -> Style {
        Style::new().with_font_size(self.pt(10.0))
    }
    pub(crate) fn role(&self) -> Style {
        Style::new().bold().with_font_size(self.pt(10.5))
    }
    pub(crate) fn meta(&self) -> Style {
        Style::new()
            .italic()
            .with_font_size(self.pt(9.5))
            .with_color(GRAY)
    }
    pub(crate) fn dates(&self) -> Style {
        Style::new().with_font_size(self.pt(9.5)).with_color(GRAY)
    }
    pub(crate) fn stack_label(&self) -> Style {
        Style::new()
            .bold()
            .with_font_size(self.pt(8.8))
            .with_color(STEEL)
    }
    pub(crate) fn stack(&self) -> Style {
        Style::new().with_font_size(self.pt(8.8)).with_color(GRAY)
    }
    pub(crate) fn side_heading(&self) -> Style {
        Style::new()
            .bold()
            .with_font_size(self.pt(10.0))
            .with_color(NAVY)
    }
    pub(crate) fn side_label(&self) -> Style {
        Style::new()
            .bold()
            .with_font_size(self.pt(8.0))
            .with_color(STEEL)
    }
    pub(crate) fn side_body(&self) -> Style {
        Style::new().with_font_size(self.pt(8.8))
    }
    pub(crate) fn side_role(&self) -> Style {
        Style::new().bold().with_font_size(self.pt(9.2))
    }
    pub(crate) fn side_meta(&self) -> Style {
        Style::new().with_font_size(self.pt(8.5)).with_color(GRAY)
    }
    pub(crate) fn gap(&self, lines: f64) -> Break {
        Break::new(lines * self.scale)
    }
}
