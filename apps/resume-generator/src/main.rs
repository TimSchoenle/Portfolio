//! Generates localized, single-page resume PDFs from the shared portfolio data.
//!
//! Output (into `<out-dir>`, default `dist`):
//!   resume/Tim-Schönle-Resume.pdf      (en)
//!   resume/Tim-Schönle-Lebenslauf.pdf  (de)
//!   resume-fingerprint.json            (SHA-256 per file, shown on the contact card)
//!
//! Layout: an open header (name and title left, contact right) over a
//! gradient rule, a full-width professional summary, then a two-column body —
//! skills (sorted strongest first, grouped by category), education and
//! languages in a narrow left sidebar, reverse-chronological experience (each
//! role with a plain-text "Stack:" keyword line) in the main column. All
//! structural lines share one visual language: navy fading to a soft
//! blue-gray, drawn as interpolated stroke segments since genpdf exposes no
//! PDF shadings. Text is emitted in the order ATS parsers expect (identity →
//! summary → skills → experience → education), with standard section names
//! and consistent "Mon YYYY – Mon YYYY" ranges. Fonts are embedded
//! (Liberation Sans, SIL OFL, pre-subset to Latin-1) so text stays selectable
//! and machine-readable, including umlauts; the gradient lines are vector
//! strokes, invisible to text extraction.
//!
//! The single-page guarantee is dynamic and prefers readability over
//! shrinking: a binary search finds the largest scale ≥ 0.9 that fits; if
//! the content has grown too much, the two oldest roles are condensed to
//! their two strongest bullets before smaller scales are even considered.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use genpdf::elements::{Break, LinearLayout, Paragraph, TableLayout};
use genpdf::error::Error as PdfError;
use genpdf::fonts::{FontData, FontFamily};
use genpdf::style::{Color, Style};
use genpdf::{
    Alignment, Context, Document, Element, Margins, Mm, Position, RenderResult,
    SimplePageDecorator, Size, render,
};
use portfolio_data::{
    CONFIG, EDUCATION, EXPERIENCE, I18N_DE, I18N_EN, Quadrant, RESUME_FILES, matrix_skills,
};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Primary accent: conservative navy for the name, headings and the dark end
/// of every gradient — print-safe, one hue family total (recruiter guidance).
const NAVY: Color = Color::Rgb(23, 54, 93);
/// Secondary accent: a lighter steel blue for the job title, category labels
/// and "Stack:" prefixes.
const STEEL: Color = Color::Rgb(64, 102, 154);
/// Dark gray for dates / companies — readable on paper, clearly secondary.
const GRAY: Color = Color::Rgb(90, 98, 110);
/// Soft blue-gray every gradient fades into (close to the paper white).
const SOFT: Color = Color::Rgb(228, 233, 240);

/// Scales ≥ this keep the body font at ~9 pt — the readability floor
/// recommended for human review. Content is condensed before going lower.
const PREFERRED_MIN_SCALE: f64 = 0.90;
/// Hard floor; below this the generator refuses and fails the build.
const ABSOLUTE_MIN_SCALE: f64 = 0.82;

fn main() {
    if let Err(err) = run() {
        eprintln!("resume-generator: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let out_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "dist".to_string());
    let resume_dir = Path::new(&out_dir).join("resume");
    fs::create_dir_all(&resume_dir)?;

    let fonts = load_fonts()?;
    let mut fingerprints: BTreeMap<String, String> = BTreeMap::new();

    for (lang, file_name) in RESUME_FILES {
        let json = match lang {
            "de" => I18N_DE,
            _ => I18N_EN,
        };
        let t = Translations::parse(json)?;
        let fitted = fit_single_page(&fonts, &t)
            .ok_or_else(|| format!("{file_name}: does not fit one page even condensed"))?;

        let title = format!("{} — {}", CONFIG.full_name, t.get("hero.jobTitle"));
        let bytes = fix_metadata(&fitted.bytes, &title)?;

        let path = resume_dir.join(file_name);
        fs::write(&path, &bytes)?;
        fingerprints.insert(file_name.to_string(), hex(&Sha256::digest(&bytes)));
        println!(
            "wrote {} ({} bytes, scale {:.2}, {})",
            path.display(),
            bytes.len(),
            fitted.scale,
            fitted.detail.describe()
        );
    }

    let manifest = serde_json::json!({
        "algorithm": "SHA-256",
        "generated_at": OffsetDateTime::now_utc().format(&Rfc3339)?,
        "files": fingerprints,
    });
    let manifest_path = Path::new(&out_dir).join("resume-fingerprint.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    println!("wrote {}", manifest_path.display());

    Ok(())
}

// ---------- single-page fitting ----------

/// How much detail the experience section carries. Standard one-page
/// practice: recent roles get full bullets, older roles get fewer — content
/// is condensed before type is shrunk below readable sizes.
#[derive(Clone, Copy, PartialEq)]
enum Detail {
    /// Every bullet of every role.
    Full,
    /// Ended roles beyond the two most recent keep their two strongest
    /// bullets; ongoing engagements stay full.
    Condensed,
    /// Every role beyond the two most recent keeps two bullets.
    Compact,
}

impl Detail {
    fn bullet_count(self, entry_index: usize, e: &portfolio_data::Experience) -> u8 {
        let recent = entry_index < 2;
        let ongoing = e.end.is_none();
        match self {
            Detail::Full => e.bullet_count,
            Detail::Condensed if recent || ongoing => e.bullet_count,
            Detail::Compact if recent => e.bullet_count,
            _ => e.bullet_count.min(2),
        }
    }

    fn describe(self) -> &'static str {
        match self {
            Detail::Full => "full detail",
            Detail::Condensed => "older ended roles condensed",
            Detail::Compact => "older roles condensed",
        }
    }
}

struct Fitted {
    bytes: Vec<u8>,
    scale: f64,
    detail: Detail,
}

/// Degradation ladder: full detail at readable scales, then condensed at
/// readable scales, then condensed down to the hard floor.
fn fit_single_page(fonts: &FontFamily<FontData>, t: &Translations) -> Option<Fitted> {
    let attempts = [
        (Detail::Full, PREFERRED_MIN_SCALE, 1.0),
        (Detail::Condensed, PREFERRED_MIN_SCALE, 1.0),
        (Detail::Compact, PREFERRED_MIN_SCALE, 1.0),
        (Detail::Compact, ABSOLUTE_MIN_SCALE, PREFERRED_MIN_SCALE),
    ];
    attempts.into_iter().find_map(|(detail, lo, hi)| {
        largest_fitting_scale(fonts, t, detail, lo, hi).map(|(bytes, scale)| Fitted {
            bytes,
            scale,
            detail,
        })
    })
}

/// Binary-searches the largest scale within `[lo, hi]` that renders as a
/// single page. Returns `None` if even `lo` overflows.
fn largest_fitting_scale(
    fonts: &FontFamily<FontData>,
    t: &Translations,
    detail: Detail,
    lo: f64,
    hi: f64,
) -> Option<(Vec<u8>, f64)> {
    let render_at = |scale: f64| -> Option<(Vec<u8>, bool)> {
        let doc = build_resume(fonts.clone(), t, Layout { scale }, detail);
        let mut bytes = Vec::new();
        doc.render(&mut bytes).ok()?;
        let fits = page_count(&bytes) <= 1;
        Some((bytes, fits))
    };

    let (lo_bytes, lo_fits) = render_at(lo)?;
    if !lo_fits {
        return None;
    }
    if let Some((hi_bytes, true)) = render_at(hi) {
        return Some((hi_bytes, hi));
    }

    // Invariant: `lo` fits, `hi` does not.
    let (mut lo, mut hi) = (lo, hi);
    let mut best = (lo_bytes, lo);
    while hi - lo > 0.01 {
        let mid = (lo + hi) / 2.0;
        match render_at(mid) {
            Some((bytes, true)) => {
                best = (bytes, mid);
                lo = mid;
            }
            _ => hi = mid,
        }
    }
    Some(best)
}

/// Number of pages, read from the `/Count N` entry of the PDF page tree.
fn page_count(bytes: &[u8]) -> usize {
    let needle = b"/Count ";
    bytes
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|i| {
            bytes[i + needle.len()..]
                .iter()
                .take_while(|b| b.is_ascii_digit())
                .fold(0usize, |acc, b| acc * 10 + (b - b'0') as usize)
        })
        .unwrap_or(usize::MAX)
}

// ---------- typography ----------

/// Scaled typography (bases follow resume-typography guidance: name 20 pt,
/// headings 11.5 pt, body 10 pt, secondary 9.5 pt, sidebar ~9 pt).
/// Everything shrinks together so the hierarchy survives the
/// fit-to-one-page reduction.
#[derive(Clone, Copy)]
struct Layout {
    scale: f64,
}

impl Layout {
    fn pt(&self, base: f64) -> u8 {
        ((base * self.scale).round() as u8).max(7)
    }
    fn name(&self) -> Style {
        Style::new()
            .bold()
            .with_font_size(self.pt(20.0))
            .with_color(NAVY)
    }
    fn title(&self) -> Style {
        Style::new().with_font_size(self.pt(11.0)).with_color(STEEL)
    }
    fn contact(&self) -> Style {
        Style::new().with_font_size(self.pt(8.8)).with_color(GRAY)
    }
    fn heading(&self) -> Style {
        Style::new()
            .bold()
            .with_font_size(self.pt(11.5))
            .with_color(NAVY)
    }
    fn body(&self) -> Style {
        Style::new().with_font_size(self.pt(10.0))
    }
    fn role(&self) -> Style {
        Style::new().bold().with_font_size(self.pt(10.5))
    }
    fn meta(&self) -> Style {
        Style::new()
            .italic()
            .with_font_size(self.pt(9.5))
            .with_color(GRAY)
    }
    fn dates(&self) -> Style {
        Style::new().with_font_size(self.pt(9.5)).with_color(GRAY)
    }
    fn stack_label(&self) -> Style {
        Style::new()
            .bold()
            .with_font_size(self.pt(8.8))
            .with_color(STEEL)
    }
    fn stack(&self) -> Style {
        Style::new().with_font_size(self.pt(8.8)).with_color(GRAY)
    }
    fn side_heading(&self) -> Style {
        Style::new()
            .bold()
            .with_font_size(self.pt(10.0))
            .with_color(NAVY)
    }
    fn side_label(&self) -> Style {
        Style::new()
            .bold()
            .with_font_size(self.pt(8.0))
            .with_color(STEEL)
    }
    fn side_body(&self) -> Style {
        Style::new().with_font_size(self.pt(8.8))
    }
    fn side_role(&self) -> Style {
        Style::new().bold().with_font_size(self.pt(9.2))
    }
    fn side_meta(&self) -> Style {
        Style::new().with_font_size(self.pt(8.5)).with_color(GRAY)
    }
    fn gap(&self, lines: f64) -> Break {
        Break::new(lines * self.scale)
    }
}

// ---------- custom elements ----------

/// Linear interpolation between two RGB colors (`t` in 0.0..=1.0).
fn lerp_color(from: Color, to: Color, t: f64) -> Color {
    match (from, to) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let mix = |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * t) as u8;
            Color::Rgb(mix(r1, r2), mix(g1, g2), mix(b1, b2))
        }
        _ => from,
    }
}

/// A horizontal rule fading from `from` to `to` across its width. genpdf
/// exposes no PDF shadings, so the gradient is many short solid segments
/// with interpolated colors; segments overlap slightly so no seams show.
struct GradientRule {
    from: Color,
    to: Color,
}

/// Segment count per gradient line: high enough that adjacent color steps
/// are invisible. Segments meet at exact butt joints (plus a hairline
/// epsilon against anti-aliasing seams); larger overlaps double-paint and
/// show as periodic darker patches.
const GRADIENT_SEGMENTS: usize = 96;
const GRADIENT_JOINT_EPSILON: f64 = 0.03;

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
struct BulletItem<E: Element> {
    child: E,
    style: Style,
    indent: f64,
    bullet_rendered: bool,
}

impl<E: Element> BulletItem<E> {
    fn new(child: E, l: &Layout) -> Self {
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
struct RuledColumn<E: Element> {
    child: E,
    indent: f64,
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

// ---------- document ----------

fn build_resume(
    fonts: FontFamily<FontData>,
    t: &Translations,
    l: Layout,
    detail: Detail,
) -> Document {
    let mut doc = Document::new(fonts);
    doc.set_title(format!("{} — {}", CONFIG.full_name, t.get("hero.jobTitle")));
    // Drops the default ICC profile / XMP metadata (~650 KB per file).
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.05);
    // Gaps derive from the default style's line height; without this they
    // would be based on genpdf's 12 pt default and come out ~20% too tall.
    doc.set_font_size(10);

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(Margins::trbl(10, 13, 10, 13));
    doc.set_page_decorator(decorator);

    // ---- identity header: name/title left, contact right ----
    let mut left = LinearLayout::vertical();
    left.push(Paragraph::new(CONFIG.full_name).styled(l.name()));
    left.push(l.gap(0.1));
    left.push(Paragraph::new(t.get("hero.jobTitle")).styled(l.title()));

    let mut right = LinearLayout::vertical();
    // Optically aligns the first contact line with the large name's cap height.
    right.push(l.gap(0.25));
    for line in [
        format!("{} · {}", t.get("common.country"), CONFIG.email),
        format!(
            "{} · {}",
            strip_scheme(CONFIG.url),
            strip_scheme(CONFIG.github)
        ),
        strip_scheme(CONFIG.linkedin),
    ] {
        right.push(
            Paragraph::new(line)
                .aligned(Alignment::Right)
                .styled(l.contact()),
        );
    }

    let mut header = TableLayout::new(vec![3, 2]);
    header
        .row()
        .element(left)
        .element(right)
        .push()
        .expect("header row matches column count");
    doc.push(header);
    doc.push(l.gap(0.4));
    doc.push(GradientRule {
        from: NAVY,
        to: SOFT,
    });
    doc.push(l.gap(0.5));

    // ---- professional summary (full width) ----
    doc.push(heading_block(
        &l,
        &t.get("resume.summaryTitle"),
        l.heading(),
    ));
    doc.push(Paragraph::new(t.get("resume.summary")).styled(l.body()));
    doc.push(l.gap(0.9));

    // ---- two-column body: sidebar | experience ----
    let mut columns = TableLayout::new(vec![46, 134]);
    columns
        .row()
        .element(build_sidebar(t, &l).padded(Margins::trbl(0, 3, 0, 0)))
        .element(RuledColumn {
            child: build_experience(t, &l, detail),
            indent: 4.5 * l.scale,
        })
        .push()
        .expect("column row matches column count");
    doc.push(columns);

    doc
}

/// Sidebar: skills sorted strongest first per category, education and
/// spoken languages. Skills come before experience in the extraction order,
/// which is where ATS parsers map them best.
fn build_sidebar(t: &Translations, l: &Layout) -> LinearLayout {
    let mut col = LinearLayout::vertical();

    col.push(heading_block(
        l,
        &t.get("resume.skillsTitle"),
        l.side_heading(),
    ));
    let skills = matrix_skills();
    for q in Quadrant::all() {
        col.push(l.gap(0.4));
        col.push(Paragraph::new(t.get(q.i18n_key()).to_uppercase()).styled(l.side_label()));
        col.push(l.gap(0.08));
        let names: Vec<&str> = skills
            .iter()
            .filter(|s| s.quadrant == q)
            .map(|s| s.name)
            .collect();
        col.push(Paragraph::new(names.join(" · ")).styled(l.side_body()));
    }

    col.push(l.gap(0.9));
    col.push(heading_block(
        l,
        &t.get("resume.educationTitle"),
        l.side_heading(),
    ));
    for (i, e) in EDUCATION.iter().enumerate() {
        let key = |field: &str| format!("resume.education.{}.{field}", e.id);
        if i > 0 {
            col.push(l.gap(0.45));
        }
        col.push(Paragraph::new(t.get(&key("degree"))).styled(l.side_role()));
        col.push(Paragraph::new(t.get(&key("institution"))).styled(l.side_meta()));
        col.push(Paragraph::new(t.period(e.start, e.end)).styled(l.side_meta()));
    }

    col.push(l.gap(0.9));
    col.push(heading_block(
        l,
        &t.get("resume.languagesTitle"),
        l.side_heading(),
    ));
    for (name_key, level_key) in [("german", "germanLevel"), ("english", "englishLevel")] {
        col.push(
            Paragraph::default()
                .styled_string(
                    t.get(&format!("resume.languages.{name_key}")),
                    l.side_role(),
                )
                .styled_string(" – ", l.side_meta())
                .styled_string(
                    t.get(&format!("resume.languages.{level_key}")),
                    l.side_meta(),
                ),
        );
    }

    col
}

/// Main column: reverse-chronological experience. Each role carries a
/// plain-text "Stack:" line so the technologies are explicit keywords for
/// ATS and AI reviewers, not just implied by the bullets.
fn build_experience(t: &Translations, l: &Layout, detail: Detail) -> LinearLayout {
    let mut col = LinearLayout::vertical();
    col.push(heading_block(
        l,
        &t.get("resume.experienceTitle"),
        l.heading(),
    ));
    for (i, e) in EXPERIENCE.iter().enumerate() {
        let key = |field: &str| format!("experience.entries.{}.{field}", e.id);
        if i > 0 {
            col.push(l.gap(0.6));
        }
        col.push(split_row(
            l,
            Paragraph::new(t.get(&key("role"))).styled(l.role()),
            &t.period(e.start, e.end),
        ));
        col.push(
            Paragraph::new(format!("{} — {}", t.get(&key("org")), e.location)).styled(l.meta()),
        );
        col.push(l.gap(0.12));
        for n in 1..=detail.bullet_count(i, e) {
            col.push(BulletItem::new(
                Paragraph::new(t.get(&key(&format!("bullets.b{n}")))).styled(l.body()),
                l,
            ));
        }
        col.push(l.gap(0.08));
        col.push(
            Paragraph::default()
                .styled_string(
                    format!("{}:  ", t.get("resume.stackLabel")),
                    l.stack_label(),
                )
                .styled_string(e.tech.join(" · "), l.stack()),
        );
    }
    col
}

/// Section heading: uppercase title over a gradient rule, with a small gap
/// below (the gap above is the caller's, so document- and column-level
/// spacing can differ).
fn heading_block(l: &Layout, title: &str, style: Style) -> LinearLayout {
    let mut block = LinearLayout::vertical();
    block.push(Paragraph::new(title.to_uppercase()).styled(style));
    block.push(GradientRule {
        from: NAVY,
        to: SOFT,
    });
    block.push(l.gap(0.12));
    block
}

/// Left content with a right-aligned date column. Renders as positioned
/// text (not a structural table), so extraction order stays "content, then
/// dates" — the layout Word templates produce with tab stops.
fn split_row(l: &Layout, left: impl Element + 'static, dates: &str) -> TableLayout {
    let mut row = TableLayout::new(vec![7, 3]);
    row.row()
        .element(left)
        .element(
            Paragraph::new(dates)
                .aligned(Alignment::Right)
                .styled(l.dates()),
        )
        .push()
        .expect("split row matches column count");
    row
}

/// Liberation Sans (SIL OFL), pre-subset to Latin-1 + typographic
/// punctuation so the embedded fonts stay small while text remains
/// selectable and machine-readable (ATS-friendly).
fn load_fonts() -> Result<FontFamily<FontData>, Box<dyn Error>> {
    let load = |bytes: &'static [u8]| FontData::new(bytes.to_vec(), None);
    Ok(FontFamily {
        regular: load(include_bytes!("../fonts/LiberationSans-Regular.ttf"))?,
        bold: load(include_bytes!("../fonts/LiberationSans-Bold.ttf"))?,
        italic: load(include_bytes!("../fonts/LiberationSans-Italic.ttf"))?,
        bold_italic: load(include_bytes!("../fonts/LiberationSans-BoldItalic.ttf"))?,
    })
}

/// Rewrites the document-info strings as UTF-16BE (with BOM).
///
/// printpdf 0.3 stores the title as raw UTF-8 inside a PDFDocEncoding string,
/// which viewers display as mojibake for non-ASCII ("SchÃ¶nle"). Re-encoding
/// through lopdf also rebuilds the xref correctly.
fn fix_metadata(bytes: &[u8], title: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    fn utf16be(text: &str) -> lopdf::Object {
        let mut bytes: Vec<u8> = vec![0xFE, 0xFF];
        for unit in text.encode_utf16() {
            bytes.extend_from_slice(&unit.to_be_bytes());
        }
        lopdf::Object::String(bytes, lopdf::StringFormat::Hexadecimal)
    }

    let mut doc = lopdf::Document::load_mem(bytes)?;
    let info_id = doc
        .trailer
        .get(b"Info")
        .and_then(lopdf::Object::as_reference)?;
    let info = doc.get_object_mut(info_id)?.as_dict_mut()?;
    info.set("Title", utf16be(title));
    info.set("Author", utf16be(CONFIG.full_name));

    let mut out = Vec::new();
    doc.save_to(&mut out)?;
    Ok(out)
}

fn strip_scheme(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_string()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Read-only view over one embedded translation file.
struct Translations {
    root: serde_json::Value,
    months: Vec<String>,
    present: String,
}

impl Translations {
    fn parse(json: &str) -> Result<Self, Box<dyn Error>> {
        let root: serde_json::Value = serde_json::from_str(json)?;
        let mut t = Translations {
            root,
            months: Vec::new(),
            present: String::new(),
        };
        t.months = (1..=12)
            .map(|n| t.get(&format!("common.months.m{n}")))
            .collect();
        t.present = t.get("common.present");
        Ok(t)
    }

    /// Looks up a dotted key; the data crate's tests guarantee presence.
    fn get(&self, key: &str) -> String {
        let mut value = &self.root;
        for part in key.split('.') {
            value = &value[part];
        }
        value
            .as_str()
            .unwrap_or_else(|| panic!("missing translation key '{key}'"))
            .to_string()
    }

    fn period(
        &self,
        start: portfolio_data::YearMonth,
        end: Option<portfolio_data::YearMonth>,
    ) -> String {
        portfolio_data::format_period(start, end, &self.months, &self.present)
    }
}
