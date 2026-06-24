//! Assembles the resume `Document` from the shared data and translations:
//! identity header, full-width summary, and the two-column sidebar/experience
//! body. Also loads the embedded fonts.

use std::error::Error;

use genpdf::elements::{LinearLayout, Paragraph, TableLayout};
use genpdf::fonts::{FontData, FontFamily};
use genpdf::style::Style;
use genpdf::{Alignment, Document, Element, Margins, SimplePageDecorator};
use portfolio_data::{CONFIG, EDUCATION, EXPERIENCE, Quadrant, matrix_skills};

use crate::elements::{BulletItem, GradientRule, RuledColumn};
use crate::fit::Detail;
use crate::style::{Layout, NAVY, SOFT};
use crate::translations::Translations;

pub(crate) fn build_resume(
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
pub(crate) fn load_fonts() -> Result<FontFamily<FontData>, Box<dyn Error>> {
    let load = |bytes: &'static [u8]| FontData::new(bytes.to_vec(), None);
    Ok(FontFamily {
        regular: load(include_bytes!("../fonts/LiberationSans-Regular.ttf"))?,
        bold: load(include_bytes!("../fonts/LiberationSans-Bold.ttf"))?,
        italic: load(include_bytes!("../fonts/LiberationSans-Italic.ttf"))?,
        bold_italic: load(include_bytes!("../fonts/LiberationSans-BoldItalic.ttf"))?,
    })
}

fn strip_scheme(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_string()
}
