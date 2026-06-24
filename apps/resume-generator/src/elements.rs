//! Custom genpdf elements: gradient rules, compact bullets and the ruled
//! sidebar divider. genpdf exposes no PDF shadings, so every gradient is drawn
//! as many short interpolated stroke segments.

use genpdf::error::Error as PdfError;
use genpdf::style::{Color, Style};
use genpdf::{Context, Element, Mm, Position, RenderResult, Size, render};

use crate::style::{Layout, NAVY, SOFT, lerp_color};

/// Segment count per gradient line: high enough that adjacent color steps
/// are invisible. Segments meet at exact butt joints (plus a hairline
/// epsilon against anti-aliasing seams); larger overlaps double-paint and
/// show as periodic darker patches.
const GRADIENT_SEGMENTS: usize = 96;
const GRADIENT_JOINT_EPSILON: f64 = 0.03;

/// A horizontal rule fading from `from` to `to` across its width. genpdf
/// exposes no PDF shadings, so the gradient is many short solid segments
/// with interpolated colors; segments overlap slightly so no seams show.
pub(crate) struct GradientRule {
    pub(crate) from: Color,
    pub(crate) to: Color,
}

impl Element for GradientRule {
    fn render(
        &mut self,
        _context: &Context,
        area: render::Area<'_>,
        _style: Style,
    ) -> Result<RenderResult, PdfError> {
        let width = area.size().width;
        let seg = width / GRADIENT_SEGMENTS as f64;
        for i in 0..GRADIENT_SEGMENTS {
            let t = i as f64 / (GRADIENT_SEGMENTS - 1) as f64;
            let x0 = seg * i as f64;
            let mut x1 = seg * (i as f64 + 1.0) + Mm::from(GRADIENT_JOINT_EPSILON);
            if x1 > width {
                x1 = width;
            }
            area.draw_line(
                vec![Position::new(x0, 0.4), Position::new(x1, 0.4)],
                Style::new().with_color(lerp_color(self.from, self.to, t)),
            );
        }
        Ok(RenderResult {
            size: Size::new(width, Mm::from(0.8)),
            has_more: false,
        })
    }
}

/// A bullet item with a compact indent. genpdf's own `BulletPoint` hardcodes
/// a 10 mm indent, which wastes precious width on every wrapped line inside
/// the experience column.
pub(crate) struct BulletItem<E: Element> {
    child: E,
    style: Style,
    indent: f64,
    bullet_rendered: bool,
}

impl<E: Element> BulletItem<E> {
    pub(crate) fn new(child: E, l: &Layout) -> Self {
        BulletItem {
            child,
            style: l.body(),
            indent: 3.6 * l.scale,
            bullet_rendered: false,
        }
    }
}

impl<E: Element> Element for BulletItem<E> {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, PdfError> {
        let mut inner = area.clone();
        inner.add_offset(Position::new(self.indent, 0));
        let result = self.child.render(context, inner, style)?;
        if !self.bullet_rendered {
            area.print_str(&context.font_cache, Position::new(0.4, 0), self.style, "•")?;
            self.bullet_rendered = true;
        }
        Ok(RenderResult {
            size: Size::new(area.size().width, result.size.height),
            has_more: result.has_more,
        })
    }
}

/// Renders `child` indented by `indent` and draws a vertical gradient line
/// along its left edge, as tall as the rendered content — the divider
/// between the sidebar and the main column.
pub(crate) struct RuledColumn<E: Element> {
    pub(crate) child: E,
    pub(crate) indent: f64,
}

impl<E: Element> Element for RuledColumn<E> {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, PdfError> {
        let mut inner = area.clone();
        inner.add_offset(Position::new(self.indent, 0));
        let result = self.child.render(context, inner, style)?;
        let height = result.size.height;
        let seg = height / GRADIENT_SEGMENTS as f64;
        for i in 0..GRADIENT_SEGMENTS {
            let t = i as f64 / (GRADIENT_SEGMENTS - 1) as f64;
            let y0 = seg * i as f64;
            let mut y1 = seg * (i as f64 + 1.0) + Mm::from(GRADIENT_JOINT_EPSILON);
            if y1 > height {
                y1 = height;
            }
            area.draw_line(
                vec![Position::new(0.4, y0), Position::new(0.4, y1)],
                Style::new().with_color(lerp_color(NAVY, SOFT, t)),
            );
        }
        Ok(RenderResult {
            size: Size::new(area.size().width, height),
            has_more: result.has_more,
        })
    }
}
