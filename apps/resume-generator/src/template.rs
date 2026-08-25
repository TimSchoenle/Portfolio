//! Generates the two-column resume as Typst markup from the shared portfolio
//! data and the localized translations.
//!
//! The main column is written *first* even though the sidebar prints to the
//! left of it, so the reading order a parser sees is identity → summary →
//! experience → contact → skills. [`two_column`] is where that is arranged, and
//! its doc comment explains why the two columns are `#place`d rather than put
//! in a `#grid`.
//!
//! Every length here comes from [`style`] in the design's own unit — see that
//! module for how a CSS pixel, a CSS margin and a CSS `line-height` each map
//! onto Typst.

use portfolio_data::{CONFIG, EDUCATION, Quadrant, experiences_sorted, matrix_skills};

use crate::fit::Detail;
use crate::qr;
use crate::style::{self, Layout, Px};
use crate::translations::Translations;

/// The German sheet's optional application photo, already resolved to the
/// virtual file the [`world`](crate::world) serves it under.
#[derive(Clone, Copy)]
pub(crate) struct Photo<'a> {
    /// File name the document references, extension included so Typst can pick
    /// the decoder.
    pub(crate) file_name: &'a str,
}

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

/// Vertical space, in design pixels.
fn v(l: &Layout, px: Px) -> String {
    format!("#v({})", l.pt(px))
}

/// A run of text. `body` is already escaped.
fn run(l: &Layout, size: Px, weight: u16, fill: &str, body: &str) -> String {
    format!(
        "#text(size: {size}, weight: {weight}, fill: rgb(\"{fill}\"))[{body}]",
        size = l.pt(size),
    )
}

/// A run of text with letter spacing — the uppercase labels.
fn tracked(l: &Layout, size: Px, weight: u16, fill: &str, tracking: f64, body: &str) -> String {
    format!(
        "#text(size: {size}, weight: {weight}, fill: rgb(\"{fill}\"), tracking: {tracking}em)[{body}]",
        size = l.pt(size),
    )
}

/// A hairline that runs solid to `solid_until` percent of its width and fades
/// to nothing by its right edge.
///
/// Typst cannot gradient-stroke a `#line`, so the rules are zero-content blocks
/// filled with the gradient instead. sRGB interpolation, because that is what
/// the CSS the design is written in does.
fn fading_rule(l: &Layout, height: Px, color: &str, solid_until: u32) -> String {
    format!(
        "#block(width: 100%, height: {h}, fill: gradient.linear(space: rgb, \
         (rgb(\"{color}\"), 0%), (rgb(\"{color}\"), {solid_until}%), \
         (rgb(\"{color}\").transparentize(100%), 100%)))",
        h = l.pt(height),
    )
}

/// Builds the complete `.typ` source for one localized, single-page resume.
///
/// `photo` is honored only on the German sheet; the English one never renders
/// it, and neither sheet renders anything in its place when it is absent.
pub(crate) fn build_typ(
    t: &Translations,
    l: Layout,
    detail: Detail,
    lang: &str,
    photo: Option<Photo<'_>>,
) -> String {
    let region = if lang == "de" { "de" } else { "us" };
    let title = format!("{} — {}", CONFIG.full_name, t.get("hero.jobTitle"));
    let (top_edge, bottom_edge) = Layout::edges(style::LH_NORMAL);
    let photo = if lang == "de" { photo } else { None };

    format!(
        "#set document(title: {title:?}, author: {author:?}, date: none)\n\
         #set page(width: {pw}, height: {ph}, \
         margin: (x: {mx}, top: {mt}, bottom: {mb}), fill: rgb(\"{paper}\"))\n\
         #set text(font: (\"Inter\", \"Liberation Sans\"), lang: \"{lang}\", region: \"{region}\", \
         hyphenate: false, size: {body}, fill: rgb(\"{ink}\"), weight: {regular}, \
         top-edge: {top_edge}, bottom-edge: {bottom_edge}, variations: (opsz: {opsz}))\n\
         #set par(leading: 0em, spacing: 0pt, justify: false)\n\
         #set block(spacing: 0pt)\n\
         #show link: set text(fill: rgb(\"{link}\"))\n\
         \n{header}\n\n{gap}\n\n{grid}\n",
        title = title,
        author = CONFIG.full_name,
        pw = Layout::page_pt(style::PAGE_W),
        ph = Layout::page_pt(style::PAGE_H),
        mx = Layout::page_pt(style::PAGE_PAD_X),
        mt = Layout::page_pt(style::PAGE_PAD_TOP),
        mb = Layout::page_pt(style::PAGE_PAD_BOTTOM),
        paper = style::PAPER,
        lang = lang,
        region = region,
        body = l.pt(style::FS_BODY),
        ink = style::INK_BODY,
        regular = style::W_REGULAR,
        opsz = style::OPTICAL_SIZE,
        link = style::LINK,
        header = header_band(t, &l),
        gap = v(&l, l.sp().header_rule_below),
        grid = two_column(t, &l, detail, photo),
    )
}

/// Full-width header band: the QR frame, the name over its accent job title,
/// and the availability pill pushed to the right margin — one row, centered on
/// each other — over the accent rule that fades out to the right.
///
/// A plain `#grid` is enough here even though the QR sits leftmost: it is a
/// pure vector graphic carrying no text, so the identity is still the first
/// thing the text layer holds.
fn header_band(t: &Translations, l: &Layout) -> String {
    let (name_top, name_bottom) = Layout::edges(style::LH_NAME);
    let name_stack = format!(
        "#text(size: {fs_name}, weight: {w_name}, fill: rgb(\"{ink}\"), \
         tracking: {track}em, top-edge: {name_top}, bottom-edge: {name_bottom})[{name}]\n\n\
         {gap}\n\n{role}",
        fs_name = l.pt(style::FS_NAME),
        w_name = style::W_TITLE,
        ink = style::INK,
        track = style::TRACK_NAME,
        name = esc(CONFIG.full_name),
        gap = v(l, style::NAME_TO_ROLE),
        role = run(
            l,
            style::FS_HEADLINE,
            style::W_MEDIUM,
            style::ACCENT,
            &esc(&t.get("hero.jobTitle")),
        ),
    );

    // A frame the QR sits in, white so the code keeps its quiet zone whatever
    // the paper does. A `#block` rather than a `#box`, so the cell is exactly
    // the frame's height and the row centers on it correctly.
    let qr_frame = match qr::field(l, CONFIG.url) {
        Ok(field) => format!(
            "#block(fill: rgb(\"{bg}\"), stroke: {w} + rgb(\"{border}\"), \
             radius: {radius}, inset: {inset})[{field}]",
            bg = style::TAG_BG,
            w = l.pt(style::HAIRLINE),
            border = style::TAG_BORDER,
            radius = l.pt(style::RADIUS_TAG),
            inset = l.pt(style::QR_PAD + style::HAIRLINE / 2.0),
        ),
        // The site URL is a compile-time constant that encodes; a failure here
        // means someone changed it to something no QR version holds, and the
        // header simply loses the frame rather than the build losing the sheet.
        Err(_) => String::new(),
    };

    let pill = format!(
        "#block(fill: rgb(\"{tint}\"), stroke: {w} + rgb(\"{border}\"), \
         radius: {radius}, inset: (x: {px}, y: {py}))[{text}]",
        tint = style::SIDEBAR_BG,
        w = l.pt(style::HAIRLINE),
        border = style::PILL_BORDER,
        radius = l.pt(999.0),
        px = l.pt(style::PILL_PAD_X + style::HAIRLINE / 2.0),
        py = l.pt(style::PILL_PAD_Y + style::HAIRLINE / 2.0),
        text = run(
            l,
            style::FS_AVAIL,
            style::W_REGULAR,
            style::INK_MUTED,
            &esc(&t.get("common.openToRemote")),
        ),
    );

    format!(
        "#grid(columns: (auto, 1fr, auto), column-gutter: ({gap_qr}, {gap_pill}), \
         align: (left + horizon, left + horizon, right + horizon),\n\
         [{qr_frame}],\n[{name_stack}],\n[{pill}],\n)\n\n\
         {above}\n\n{rule}",
        gap_qr = l.pt(style::HEADER_GAP_QR),
        gap_pill = l.pt(style::HEADER_GAP_PILL),
        above = v(l, l.sp().header_rule_above),
        rule = format_args!(
            "#block(width: 100%, height: {h}, fill: gradient.linear(space: rgb, \
             (rgb(\"{accent}\"), 0%), (rgb(\"{soft}\"), 55%), \
             (rgb(\"{soft}\").transparentize(100%), 100%)))",
            h = l.pt(style::HEADER_RULE_H),
            accent = style::ACCENT,
            soft = style::ACCENT_SOFT,
        ),
    )
}

/// The two columns: the wide main column on the right, the tinted sidebar on
/// the left — with the main column *written first*.
///
/// That ordering is the point of this function. A resume parser reads the text
/// layer in the order the page paints it, and the order this document wants is
/// identity → summary → experience → contact → skills, which is not the order
/// the columns sit in. A `#grid` cannot express it: cells are painted in grid
/// position order however they are written, so the sidebar would go down first.
/// Two `#place`d columns inside one sized block can, because `#place` paints in
/// source order — and measuring both columns first is what gives that block the
/// height of the taller one, so an overflowing sheet still spills onto a second
/// page for [`fit`](crate::fit) to catch.
fn two_column(t: &Translations, l: &Layout, detail: Detail, photo: Option<Photo<'_>>) -> String {
    let side_w = style::SIDEBAR_W * l.scale;
    let gap = style::COL_GAP * l.scale;
    // The trim is fixed, so the main column is simply what the sidebar and the
    // gutter leave of it.
    let main_w = style::PAGE_W - 2.0 * style::PAGE_PAD_X - side_w - gap;
    let unscaled = Layout {
        scale: 1.0,
        dense: l.dense,
    };

    format!(
        "#context {{\n\
         let main = [{main}]\n\
         let side = [#block(width: 100%, fill: rgb(\"{tint}\"), \
         inset: (x: {pad_x}, y: {pad_y}), radius: {radius})[{side_c}]]\n\
         let height = calc.max(\n\
         measure(block(width: {main_w}, main)).height,\n\
         measure(block(width: {side_w}, side)).height,\n\
         )\n\
         block(width: 100%, height: height, {{\n\
         place(top + left, dx: {main_x}, block(width: {main_w}, main))\n\
         place(top + left, block(width: {side_w}, side))\n\
         }})\n\
         }}",
        main = main_column(t, l, detail),
        tint = style::SIDEBAR_BG,
        pad_x = l.pt(l.sp().sidebar_pad_x),
        pad_y = l.pt(l.sp().sidebar_pad_y),
        radius = l.pt(style::RADIUS_SIDEBAR),
        side_c = sidebar(t, l, photo),
        main_w = unscaled.pt(main_w),
        side_w = unscaled.pt(side_w),
        main_x = unscaled.pt(side_w + gap),
    )
}

/// Main-column section header: uppercase accent title over the fading hairline.
fn main_section(l: &Layout, title: &str) -> String {
    format!(
        "{head}\n\n{above}\n\n{rule}\n\n{below}",
        head = tracked(
            l,
            style::FS_SECTION,
            style::W_BOLD,
            style::ACCENT,
            style::TRACK_SECTION,
            &esc(&title.to_uppercase()),
        ),
        above = v(l, l.sp().main_rule_above),
        rule = fading_rule(l, style::HAIRLINE, style::RULE, 70),
        below = v(l, l.sp().main_rule_below),
    )
}

/// Sidebar section header: uppercase accent title, no rule (the tint separates).
fn side_section(l: &Layout, title: &str, gap: Px) -> String {
    format!(
        "{head}\n\n{gap}",
        head = tracked(
            l,
            style::FS_SECTION_SM,
            style::W_BOLD,
            style::ACCENT,
            style::TRACK_SECTION,
            &esc(&title.to_uppercase()),
        ),
        gap = v(l, gap),
    )
}

/// Wide main column: Summary then reverse-chronological Experience.
fn main_column(t: &Translations, l: &Layout, detail: Detail) -> String {
    let (top_edge, bottom_edge) = Layout::edges(style::LH_BODY);
    let mut blocks: Vec<String> = Vec::new();

    // -- Summary --
    blocks.push(main_section(l, &t.get("resume.summaryTitle")));
    blocks.push(format!(
        "#text(size: {fs}, fill: rgb(\"{ink}\"), top-edge: {top_edge}, bottom-edge: {bottom_edge})\
         [{summary}]",
        fs = l.pt(style::FS_BODY),
        ink = style::INK_BODY,
        summary = esc(&t.get("resume.summary")),
    ));

    // -- Experience --
    blocks.push(v(l, style::MAIN_SECTION_TOP));
    blocks.push(main_section(l, &t.get("resume.experienceTitle")));
    for (i, e) in experiences_sorted().into_iter().enumerate() {
        if i > 0 {
            blocks.push(entry_divider(l));
        }
        blocks.push(experience_entry(t, l, detail, i, e));
    }

    blocks.join("\n\n")
}

/// The faint hairline in the gap between experience entries (never after the
/// last one), with an equal band of air above and below it.
fn entry_divider(l: &Layout) -> String {
    let band = v(l, l.sp().divider_band);
    format!(
        "{band}\n\n{rule}\n\n{band}",
        rule = fading_rule(l, style::HAIRLINE, style::RULE_ENTRY, 70),
    )
}

/// One Experience entry: the role owns line 1 with the date range right-aligned
/// beside it; the company and location share line 2 (company in soft ink,
/// non-breaking; location muted); then the accent `–` bullets and a `·` tech
/// run.
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
        "#grid(columns: (1fr, auto), column-gutter: {gutter}, \
         align: (left + bottom, right + bottom),\n[{role}],\n[{dates}],\n)",
        gutter = l.pt(style::ENTRY_DATE_GAP),
        role = run(
            l,
            style::FS_TITLE,
            style::W_TITLE,
            style::INK,
            &esc(&t.get(&key("role"))),
        ),
        dates = run(
            l,
            style::FS_META,
            style::W_REGULAR,
            style::INK_MUTED,
            &esc(&t.period(e.start, e.end)),
        ),
    ));

    // Line 2: company (kept whole via `#box`) · location (muted).
    s.push_str(&format!(
        "\n\n{gap}\n\n#box[{org}]{loc}",
        gap = v(l, style::TITLE_TO_ORG),
        org = run(
            l,
            style::FS_ORG,
            style::W_REGULAR,
            style::INK_SOFT,
            &esc(&t.get(&key("org"))),
        ),
        loc = run(
            l,
            style::FS_ORG,
            style::W_REGULAR,
            style::INK_MUTED,
            &format!(" · {}", esc(e.location)),
        ),
    ));

    // Bullets (native list, so the marker precedes the text).
    let n = detail.bullet_count(index, e);
    if n > 0 {
        let (top_edge, bottom_edge) = Layout::edges(l.sp().lh_bullet);
        let items: String = (1..=n)
            .map(|b| format!("[{}],\n", esc(&t.get(&key(&format!("bullets.b{b}"))))))
            .collect();
        s.push_str(&format!(
            "\n\n{gap}\n\n#[\n\
             #set text(size: {fs}, fill: rgb(\"{ink}\"), \
             top-edge: {top_edge}, bottom-edge: {bottom_edge})\n\
             #set list(marker: text(fill: rgb(\"{marker}\"))[–], spacing: {bgap}, \
             body-indent: {indent}, indent: 0pt)\n\
             #list(\n{items})\n]",
            gap = v(l, l.sp().bullets_top),
            fs = l.pt(style::FS_BULLET),
            ink = style::INK_BODY,
            marker = style::ACCENT_SOFT,
            bgap = l.pt(l.sp().bullet_gap),
            // The design sets the marker/text gutter in pixels; as a fraction of
            // the bullet size it survives the fit ladder unchanged.
            indent = format_args!("{:.4}em", style::BULLET_INDENT / style::FS_BULLET),
            items = items,
        ));
    }

    // Explicit technology keywords (`·` run) so resume parsers pick them up.
    // The bold `Stack:` label stays in ink; the tech run is soft ink, not
    // accent, so the accent color reads as structure only.
    s.push_str(&format!(
        "\n\n{gap}\n\n{label}{tech}",
        gap = v(l, l.sp().stack_top),
        label = run(
            l,
            style::FS_STACK,
            style::W_BOLD,
            style::INK,
            &format!("{}: ", esc(&t.get("resume.stackLabel"))),
        ),
        tech = run(
            l,
            style::FS_STACK,
            style::W_REGULAR,
            style::INK_SOFT,
            &esc(&e.tech.join(" · ")),
        ),
    ));

    s
}

/// Tinted reference sidebar: Contact, Skills, Education, Languages — with the
/// German sheet's optional application photo above all of them.
fn sidebar(t: &Translations, l: &Layout, photo: Option<Photo<'_>>) -> String {
    let mut blocks: Vec<String> = Vec::new();

    if let Some(photo) = photo {
        blocks.push(photo_block(l, photo));
        blocks.push(v(l, style::PHOTO_GAP));
    }

    blocks.push(side_section(
        l,
        &t.get("resume.contactTitle"),
        l.sp().contact_head_gap,
    ));
    blocks.push(contact_block(t, l));

    blocks.push(v(l, l.sp().side_section_top));
    blocks.push(side_section(
        l,
        &t.get("resume.skillsTitle"),
        l.sp().skills_head_gap,
    ));
    blocks.push(skills_block(t, l));

    blocks.push(v(l, l.sp().side_section_top));
    blocks.push(side_section(
        l,
        &t.get("resume.educationTitle"),
        l.sp().edu_head_gap,
    ));
    blocks.push(education_block(t, l));

    blocks.push(v(l, l.sp().side_section_top));
    blocks.push(side_section(
        l,
        &t.get("resume.languagesTitle"),
        l.sp().lang_head_gap,
    ));
    blocks.push(languages_block(t, l));

    blocks.join("\n\n")
}

/// The German application photo, framed like a skill chip and centered over the
/// sidebar. The frame clips the image, so a source of any aspect ratio fills the
/// 3:4 portrait a German application expects instead of distorting into it.
fn photo_block(l: &Layout, photo: Photo<'_>) -> String {
    format!(
        "#align(center)[#box(stroke: {w} + rgb(\"{border}\"), radius: {radius}, \
         inset: {inset}, clip: true)[\
         #image({file:?}, width: {iw}, height: {ih}, fit: \"cover\", alt: {alt:?})]]",
        w = l.pt(style::HAIRLINE),
        border = style::TAG_BORDER,
        radius = l.pt(style::RADIUS_TAG),
        inset = l.pt(style::HAIRLINE / 2.0),
        file = photo.file_name,
        iw = l.pt(style::PHOTO_W - style::HAIRLINE),
        ih = l.pt(style::PHOTO_H - style::HAIRLINE),
        alt = CONFIG.full_name,
    )
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
        tracked(
            l,
            style::FS_LABEL,
            style::W_BOLD,
            style::INK_MUTED,
            style::TRACK_LABEL,
            &esc(&text.to_uppercase()),
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
            "#link({uri:?})[#box[{d}]]",
            uri = uri,
            d = run(
                l,
                style::FS_SIDEBAR,
                style::W_REGULAR,
                style::LINK,
                &esc(display)
            ),
        )
    };
    // The label is its own line; the value starts fresh beneath it, so every
    // row has the same label-above-value shape.
    let row = |lab: String, val: String| {
        format!(
            "[{lab}\n\n{gap}\n\n{val}]",
            gap = v(l, style::LABEL_TO_VALUE)
        )
    };

    // The region name is wrapped in a `#box` so it stays intact on one line
    // (never splitting at its hyphen); only the comma before the country may
    // break.
    let region = run(
        l,
        style::FS_SIDEBAR,
        style::W_REGULAR,
        style::INK_SOFT,
        &format!(
            "#box[{region}], {country}",
            region = esc(&t.get("common.region")),
            country = esc(&t.get("common.country")),
        ),
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

    format!(
        "#stack(dir: ttb, spacing: {gap},\n{rows},\n)",
        gap = l.pt(l.sp().contact_row_gap),
    )
}

/// Skills: per-group label on its own line, items as wrapped white chips. Each
/// chip is an atomic `#box` separated by a real space, so the text layer stays
/// parseable; the space is widened to the design's chip gutter and reset inside
/// each chip, which is why a two-word skill still reads with a normal space.
fn skills_block(t: &Translations, l: &Layout) -> String {
    let skills = matrix_skills();
    let mut blocks: Vec<String> = Vec::new();
    for (i, q) in Quadrant::all().into_iter().enumerate() {
        if i > 0 {
            blocks.push(v(l, l.sp().skill_group_gap));
        }
        let chips: String = skills
            .iter()
            .filter(|s| s.quadrant == q)
            .map(|s| {
                format!(
                    "#box(fill: rgb(\"{bg}\"), stroke: {w} + rgb(\"{border}\"), \
                     inset: (x: {px}, y: {py}), radius: {radius})[\
                     #text(size: {fs}, fill: rgb(\"{ink}\"), spacing: 100%)[{name}]] ",
                    bg = style::TAG_BG,
                    w = l.pt(style::HAIRLINE),
                    border = style::TAG_BORDER,
                    px = l.pt(l.sp().chip_pad_x + style::HAIRLINE / 2.0),
                    py = l.pt(l.sp().chip_pad_y + style::HAIRLINE / 2.0),
                    radius = l.pt(style::RADIUS_TAG),
                    fs = l.pt(style::FS_TAG),
                    ink = style::TAG_INK,
                    name = esc(s.name),
                )
            })
            .collect();
        blocks.push(format!(
            "{label}\n\n{gap}\n\n#[\n\
             #set text(size: {fs}, spacing: 0% + {gutter})\n\
             #set par(leading: {gutter})\n\
             {chips}\n]",
            label = run(
                l,
                style::FS_SKILL_LABEL,
                style::W_TITLE,
                style::INK,
                &esc(&t.get(q.i18n_key())),
            ),
            gap = v(l, style::SKILL_LABEL_GAP),
            fs = l.pt(style::FS_TAG),
            gutter = l.chip_gutter(l.sp().chip_gap),
            chips = chips,
        ));
    }
    blocks.join("\n\n")
}

/// Education: `Degree` (semibold) over `Institution · Years` (muted).
fn education_block(t: &Translations, l: &Layout) -> String {
    let mut blocks: Vec<String> = Vec::new();
    for (i, e) in EDUCATION.iter().enumerate() {
        if i > 0 {
            blocks.push(v(l, l.sp().edu_gap));
        }
        let key = |field: &str| format!("resume.education.{}.{field}", e.id);
        blocks.push(format!(
            "{degree}\n\n{gap}\n\n{inst}",
            degree = run(
                l,
                style::FS_SIDEBAR,
                style::W_TITLE,
                style::INK,
                &esc(&t.get(&key("degree"))),
            ),
            gap = v(l, style::LABEL_TO_VALUE),
            inst = run(
                l,
                style::FS_META,
                style::W_REGULAR,
                style::INK_MUTED,
                &format!(
                    "{inst} · {years}",
                    inst = esc(&t.get(&key("institution"))),
                    years = year_range(e.start, e.end),
                ),
            ),
        ));
    }
    blocks.join("\n\n")
}

/// Languages: `Language — Level`.
fn languages_block(t: &Translations, l: &Layout) -> String {
    let rows: Vec<String> = [("german", "germanLevel"), ("english", "englishLevel")]
        .into_iter()
        .map(|(name_key, level_key)| {
            format!(
                "[{name}{level}]",
                name = run(
                    l,
                    style::FS_SIDEBAR,
                    style::W_REGULAR,
                    style::INK,
                    &esc(&t.get(&format!("resume.languages.{name_key}"))),
                ),
                level = run(
                    l,
                    style::FS_SIDEBAR,
                    style::W_REGULAR,
                    style::INK_MUTED,
                    &format!(
                        " — {}",
                        esc(&t.get(&format!("resume.languages.{level_key}")))
                    ),
                ),
            )
        })
        .collect();
    format!(
        "#stack(dir: ttb, spacing: {gap},\n{rows},\n)",
        gap = l.pt(style::LANG_GAP),
        rows = rows.join(",\n"),
    )
}

/// `YYYY–YYYY` (single year if start == end year) for the education lines.
fn year_range(start: portfolio_data::YearMonth, end: Option<portfolio_data::YearMonth>) -> String {
    match end {
        Some(end) if end.year == start.year => format!("{}", start.year),
        Some(end) => format!("{}–{}", start.year, end.year),
        None => format!("{}–", start.year),
    }
}
