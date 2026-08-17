//! Generates the two-column resume as Typst markup from the shared portfolio
//! data and the localized translations.
//!
//! The main column is emitted *first* in the document (placed into the right
//! grid cell via `grid.cell(x: 1)`) so the tagged-PDF reading order is
//! identity → summary → experience → contact → skills even though the sidebar
//! renders on the left.

use portfolio_data::{CONFIG, EDUCATION, Quadrant, experiences_sorted, matrix_skills};

use crate::fit::Detail;
use crate::style::{self, Layout};
use crate::translations::Translations;

/// Escapes a value so it is rendered literally inside a Typst content block
/// (`[...]`), neutralizing markup-significant characters.
pub(crate) fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' | '#' | '$' | '*' | '_' | '`' | '<' | '>' | '@' | '[' | ']' | '"' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// Builds the complete `.typ` source for one localized, single-page resume.
pub(crate) fn build_typ(t: &Translations, l: Layout, detail: Detail, lang: &str) -> String {
    let region = if lang == "de" { "de" } else { "us" };
    let title = format!("{} — {}", CONFIG.full_name, t.get("hero.jobTitle"));

    format!(
        "#set document(title: {title:?}, author: {author:?}, date: none)\n\
         #set page(width: 210mm, height: 297mm, margin: (x: {mx}mm, y: {my}mm))\n\
         #set text(font: (\"Inter\", \"Liberation Sans\"), lang: \"{lang}\", region: \"{region}\", \
         hyphenate: false, size: {body}, fill: rgb(\"{ink}\"))\n\
         #set par(leading: {leading}em, spacing: 0pt, justify: false)\n\
         #set list(marker: text(fill: rgb(\"{muted}\"))[–], spacing: {bgap}, body-indent: 0.45em, indent: 0pt)\n\
         #show link: set text(fill: rgb(\"{link}\"))\n\
         \n{header}\n\n#v({sp6})\n\n{grid}\n",
        title = title,
        author = CONFIG.full_name,
        mx = l.margin_x_mm(),
        my = l.margin_y_mm(),
        lang = lang,
        region = region,
        body = l.fs_body(),
        ink = style::INK,
        leading = l.leading_em(),
        muted = style::INK_MUTED,
        bgap = l.sp(style::BULLET_GAP),
        link = style::LINK,
        header = header_band(t, &l),
        sp6 = l.sp(style::SP_6),
        grid = two_column(t, &l, detail),
    )
}

/// Full-width header band: accent name on the left with a right-aligned
/// availability tag opposite it, the soft headline beneath, then the accent
/// hairline rule.
fn header_band(t: &Translations, l: &Layout) -> String {
    format!(
        "#grid(columns: (1fr, auto), column-gutter: 8pt, align: (left + bottom, right + bottom),\n\
         [#text(size: {fs_name}, weight: 700, fill: rgb(\"{accent}\"))[{name}]],\n\
         [#box(inset: (x: 6pt, y: 3pt), radius: {rad}, fill: rgb(\"{tint}\"), \
         stroke: 0.5pt + rgb(\"{border}\"))[#text(size: {fs_avail}, fill: rgb(\"{muted}\"))[{avail}]]],\n)\n\
         #v({sp1})\n\
         #text(size: {fs_head}, fill: rgb(\"{soft}\"))[{headline}]\n\
         #v({sp2})\n\
         #line(length: 100%, stroke: 0.75pt + rgb(\"{accent}\"))",
        fs_name = l.fs_name(),
        accent = style::ACCENT,
        name = esc(CONFIG.full_name),
        rad = l.radius_tag(),
        tint = style::SIDEBAR_BG,
        border = style::TAG_BORDER,
        fs_avail = l.fs_contact(),
        muted = style::INK_MUTED,
        avail = esc(&t.get("common.openToRemote")),
        sp1 = l.sp(style::SP_1),
        fs_head = l.fs_headline(),
        soft = style::INK_SOFT,
        headline = esc(&t.get("hero.jobTitle")),
        sp2 = l.sp(style::SP_2),
    )
}

/// The two-column grid: main cell (right, emitted first) + tinted sidebar cell
/// (left, emitted second).
fn two_column(t: &Translations, l: &Layout, detail: Detail) -> String {
    format!(
        "#grid(\n\
         columns: ({side}%, 1fr),\n\
         column-gutter: {gap}mm,\n\
         grid.cell(x: 1, y: 0)[{main}],\n\
         grid.cell(x: 0, y: 0)[#block(width: 100%, fill: rgb(\"{tint}\"), inset: {pad}mm, radius: 6pt)[{side_c}]],\n\
         )",
        side = 34,
        gap = l.col_gap_mm(),
        main = main_column(t, l, detail),
        tint = style::SIDEBAR_BG,
        pad = l.sidebar_pad_mm(),
        side_c = sidebar(t, l),
    )
}

/// Main-column section header: UPPERCASE accent title over a hairline rule.
fn main_section(l: &Layout, title: &str) -> String {
    format!(
        "#text(size: {fs}, weight: 700, fill: rgb(\"{accent}\"), tracking: 0.08em)[{title}]\n\n\
         #v({sp2})\n\n\
         #line(length: 100%, stroke: 0.75pt + rgb(\"{rule}\"))\n\n\
         #v({sp4})",
        fs = l.fs_section(),
        accent = style::ACCENT,
        title = esc(&title.to_uppercase()),
        sp2 = l.sp(style::SP_2),
        rule = style::RULE,
        sp4 = l.sp(style::SP_4),
    )
}

/// Sidebar section header: UPPERCASE accent title, no rule (the tint separates).
fn side_section(l: &Layout, title: &str) -> String {
    format!(
        "#text(size: {fs}, weight: 700, fill: rgb(\"{accent}\"), tracking: 0.08em)[{title}]\n\n\
         #v({sp3})",
        fs = l.fs_section_sm(),
        accent = style::ACCENT,
        title = esc(&title.to_uppercase()),
        sp3 = l.sp(style::SP_3),
    )
}

/// Wide main column: Summary then reverse-chronological Experience.
fn main_column(t: &Translations, l: &Layout, detail: Detail) -> String {
    let mut blocks: Vec<String> = Vec::new();

    // -- Summary --
    blocks.push(main_section(l, &t.get("resume.summaryTitle")));
    blocks.push(format!(
        "#text(size: {fs}, fill: rgb(\"{ink}\"))[{summary}]",
        fs = l.fs_body(),
        ink = style::INK,
        summary = esc(&t.get("resume.summary")),
    ));

    // -- Experience --
    blocks.push(format!("#v({})", l.sp(style::SP_7)));
    blocks.push(main_section(l, &t.get("resume.experienceTitle")));
    for (i, e) in experiences_sorted().into_iter().enumerate() {
        if i > 0 {
            blocks.push(entry_divider(l));
        }
        blocks.push(experience_entry(t, l, detail, i, e));
    }

    blocks.join("\n\n")
}

/// Faint hairline in the gap between experience entries (never after the last
/// one). The `SP_5` entry gap is split around the rule so the rhythm is
/// preserved while the divider still separates entries clearly.
fn entry_divider(l: &Layout) -> String {
    format!(
        "#v({half})\n\n#line(length: 100%, stroke: {w}pt + rgb(\"{rule}\"))\n\n#v({half})",
        half = l.sp(style::SP_5 / 2.0),
        w = style::RULE_WEIGHT_PT,
        rule = style::RULE,
    )
}

/// One Experience entry: the role owns line 1 with the date range right-aligned
/// beside it; the company and location share line 2 (company in ink,
/// non-breaking; location muted); then the `–` bullets and a `·` tech run.
fn experience_entry(
    t: &Translations,
    l: &Layout,
    detail: Detail,
    index: usize,
    e: &portfolio_data::Experience,
) -> String {
    let key = |field: &str| format!("experience.entries.{}.{field}", e.id);
    let mut s = String::new();

    // Line 1: role (left) | date range (right). The date follows the title in
    // source order so extraction reads role-then-date.
    s.push_str(&format!(
        "#grid(columns: (1fr, auto), column-gutter: 6pt, align: (left + bottom, right + bottom),\n\
         [#text(size: {fst}, weight: 700, fill: rgb(\"{ink}\"))[{role}]],\n\
         [#text(size: {fsm}, fill: rgb(\"{muted}\"))[{dates}]],\n)",
        fst = l.fs_title(),
        ink = style::INK,
        role = esc(&t.get(&key("role"))),
        fsm = l.fs_meta(),
        muted = style::INK_MUTED,
        dates = esc(&t.period(e.start, e.end)),
    ));

    // Line 2: company (ink, kept whole via `#box`) · location (muted).
    s.push_str(&format!(
        "\n\n#v({sp1})\n\n\
         #box[#text(size: {fsm}, fill: rgb(\"{ink}\"))[{org}]]\
         #text(size: {fsm}, fill: rgb(\"{muted}\"))[ · {loc}]",
        sp1 = l.sp(style::SP_1),
        fsm = l.fs_meta(),
        ink = style::INK,
        muted = style::INK_MUTED,
        org = esc(&t.get(&key("org"))),
        loc = esc(e.location),
    ));

    // Bullets (native list, so the marker precedes the text).
    let n = detail.bullet_count(index, e);
    if n > 0 {
        let items: String = (1..=n)
            .map(|b| format!("[{}],\n", esc(&t.get(&key(&format!("bullets.b{b}"))))))
            .collect();
        // Space below the company line so the bullets start clearly under the
        // header rather than hugging it.
        s.push_str(&format!(
            "\n\n#v({sp3})\n\n#list(\n{items})",
            sp3 = l.sp(style::SP_3)
        ));
    }

    // Explicit technology keywords (`·` run) so resume parsers pick them up.
    // The bold `Stack:` label stays in ink; the tech run is soft ink, not
    // accent, so the accent color reads as structure only.
    s.push_str(&format!(
        "\n\n#v({sgap})\n\n#text(size: {fsm}, weight: 700, fill: rgb(\"{ink}\"))[{label}: ]\
         #text(size: {fsm}, fill: rgb(\"{soft}\"))[{tech}]",
        sgap = l.sp(style::STACK_GAP),
        fsm = l.fs_meta(),
        ink = style::INK,
        label = esc(&t.get("resume.stackLabel")),
        soft = style::INK_SOFT,
        tech = esc(&e.tech.join(" · ")),
    ));

    s
}

/// Tinted reference sidebar: Contact, Skills, Education, Languages.
fn sidebar(t: &Translations, l: &Layout) -> String {
    let mut blocks: Vec<String> = Vec::new();

    blocks.push(side_section(l, &t.get("resume.contactTitle")));
    blocks.push(contact_block(t, l));

    blocks.push(format!("#v({})", l.sp(style::SP_7)));
    blocks.push(side_section(l, &t.get("resume.skillsTitle")));
    blocks.push(skills_block(t, l));

    blocks.push(format!("#v({})", l.sp(style::SP_7)));
    blocks.push(side_section(l, &t.get("resume.educationTitle")));
    blocks.push(education_block(t, l));

    blocks.push(format!("#v({})", l.sp(style::SP_7)));
    blocks.push(side_section(l, &t.get("resume.languagesTitle")));
    blocks.push(languages_block(t, l));

    blocks.join("\n\n")
}

/// Strips the scheme (and `www.`) for a human-readable link label.
fn strip_scheme(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .to_string()
}

/// Contact block: region-level location and label-prefixed live accent links
/// (no icons; inline SVG is unavailable in the offline Typst build). Every row
/// is label-above-value, since the narrow sidebar can't fit a label and a long
/// URL on one line.
fn contact_block(t: &Translations, l: &Layout) -> String {
    let label = |text: &str| {
        format!(
            "#text(size: {fs}, weight: 700, fill: rgb(\"{muted}\"), tracking: 0.06em)[{t}]",
            fs = l.fs_label(),
            muted = style::INK_MUTED,
            t = esc(&text.to_uppercase()),
        )
    };
    // The display value is wrapped in a `#box` so a handle never breaks
    // mid-word across lines. The URI is emitted with `{:?}`, which quotes and
    // escapes it as a Rust string literal — the same syntax Typst uses — so a
    // quote or backslash in it cannot terminate the literal and have the rest
    // parsed as Typst code. `esc` is content-mode escaping and would be wrong
    // here; the same reasoning is why `build_typ` formats `title`/`author` this
    // way.
    let link = |uri: &str, display: &str| {
        format!(
            "#link({uri:?})[#box[#text(size: {fs})[{d}]]]",
            uri = uri,
            fs = l.fs_contact(),
            d = esc(display),
        )
    };
    // The label is its own line; the value starts fresh beneath it, so every
    // row has the same label-above-value shape.
    let row = |lab: String, val: String| format!("[{lab}#linebreak(){val}]");

    // The region name is wrapped in a `#box` so it stays intact on one line
    // (never splitting at its hyphen); only the comma before the country may
    // break.
    let region = format!(
        "#text(size: {fs}, fill: rgb(\"{muted}\"))[#box[{region}], {country}]",
        fs = l.fs_contact(),
        muted = style::INK_MUTED,
        region = esc(&t.get("common.region")),
        country = esc(&t.get("common.country")),
    );
    let lbl = |k: &str| t.get(&format!("resume.contactLabels.{k}"));

    let rows = [
        row(label(&lbl("location")), region),
        row(
            label(&lbl("email")),
            link(&format!("mailto:{}", CONFIG.email), CONFIG.email),
        ),
        row(
            label(&lbl("web")),
            link(CONFIG.url, &strip_scheme(CONFIG.url)),
        ),
        row(
            label(&lbl("github")),
            link(CONFIG.github, &strip_scheme(CONFIG.github)),
        ),
        row(
            label(&lbl("linkedin")),
            link(CONFIG.linkedin, &strip_scheme(CONFIG.linkedin)),
        ),
    ]
    .join(",\n");

    // A clear step between contact items: the inter-item gap is larger than the
    // in-row label/value break.
    format!(
        "#stack(dir: ttb, spacing: {sp4},\n{rows},\n)",
        sp4 = l.sp(style::SP_4)
    )
}

/// Skills: per-group label on its own line, items as wrapped white chips. Each
/// chip is an atomic `#box`, with real whitespace between chips so the text
/// layer stays parseable.
fn skills_block(t: &Translations, l: &Layout) -> String {
    let skills = matrix_skills();
    let mut blocks: Vec<String> = Vec::new();
    for (i, q) in Quadrant::all().into_iter().enumerate() {
        if i > 0 {
            // Between skill groups: a larger gap than between chip rows so the
            // groups read as separate units.
            blocks.push(format!("#v({})", l.sp(style::SP_4)));
        }
        let chips: String = skills
            .iter()
            .filter(|s| s.quadrant == q)
            .map(|s| {
                format!(
                    "#box(fill: rgb(\"{bg}\"), stroke: 0.5pt + rgb(\"{border}\"), \
                     inset: (x: 4pt, y: 1.5pt), radius: {rad})[\
                     #text(size: {fst}, fill: rgb(\"{tink}\"))[{name}]] ",
                    bg = style::TAG_BG,
                    border = style::TAG_BORDER,
                    rad = l.radius_tag(),
                    fst = l.fs_tag(),
                    tink = style::TAG_INK,
                    name = esc(s.name),
                )
            })
            .collect();
        blocks.push(format!(
            "#text(size: {fs}, weight: 700, fill: rgb(\"{ink}\"))[{label}]#linebreak()\
             #v({sp_h})\n\n{chips}",
            fs = l.fs_sidebar(),
            ink = style::INK,
            label = esc(&t.get(q.i18n_key())),
            sp_h = l.sp(style::SP_LABEL),
            chips = chips,
        ));
    }
    blocks.join("\n\n")
}

/// Education: `Degree` (semibold) / `Institution · Year` (muted).
fn education_block(t: &Translations, l: &Layout) -> String {
    let mut blocks: Vec<String> = Vec::new();
    for (i, e) in EDUCATION.iter().enumerate() {
        if i > 0 {
            blocks.push(format!("#v({})", l.sp(style::SP_3)));
        }
        let key = |field: &str| format!("resume.education.{}.{field}", e.id);
        // An explicit `#linebreak()` keeps the muted institution line beneath
        // the degree regardless of paragraph spacing.
        blocks.push(format!(
            "#text(size: {fs}, weight: 700, fill: rgb(\"{ink}\"))[{degree}]#linebreak()\
             #text(size: {fs}, fill: rgb(\"{muted}\"))[{inst} · {years}]",
            fs = l.fs_sidebar(),
            ink = style::INK,
            degree = esc(&t.get(&key("degree"))),
            muted = style::INK_MUTED,
            inst = esc(&t.get(&key("institution"))),
            years = year_range(e.start, e.end),
        ));
    }
    blocks.join("\n\n")
}

/// Languages: `Language — Level`.
fn languages_block(t: &Translations, l: &Layout) -> String {
    let mut blocks: Vec<String> = Vec::new();
    for (i, (name_key, level_key)) in [("german", "germanLevel"), ("english", "englishLevel")]
        .into_iter()
        .enumerate()
    {
        if i > 0 {
            blocks.push(format!("#v({})", l.sp(style::SP_1)));
        }
        blocks.push(format!(
            "#text(size: {fs}, fill: rgb(\"{ink}\"))[{name}]\
             #text(size: {fs}, fill: rgb(\"{muted}\"))[ — {level}]",
            fs = l.fs_sidebar(),
            ink = style::INK,
            name = esc(&t.get(&format!("resume.languages.{name_key}"))),
            muted = style::INK_MUTED,
            level = esc(&t.get(&format!("resume.languages.{level_key}"))),
        ));
    }
    blocks.join("\n\n")
}

/// `YYYY–YYYY` (single year if start == end year) for the education lines.
fn year_range(start: portfolio_data::YearMonth, end: Option<portfolio_data::YearMonth>) -> String {
    match end {
        Some(end) if end.year == start.year => format!("{}", start.year),
        Some(end) => format!("{}–{}", start.year, end.year),
        None => format!("{}–", start.year),
    }
}
