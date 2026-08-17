//! The Open Graph card — the image a link to this site unfurls to in Facebook,
//! LinkedIn, Slack, X and every other preview surface.
//!
//! # Why this exists at all
//!
//! `og:image` used to point at `/favicon.svg`, and every one of those consumers
//! refuses SVG: the tag was present, well-formed, and produced no image
//! anywhere. A social card has to be a raster, and the conventional size is
//! 1200 × 630 (the 1.91:1 box every major consumer crops to).
//!
//! # Why it is generated rather than drawn
//!
//! A committed PNG would be a second copy of facts that already live in
//! [`CONFIG`] — the name, the role, the site's own address — free to drift the
//! moment one of them changes, and silently, because nobody looks at their own
//! link previews. Generating it here keeps the card a *function* of the same
//! data the page is built from, and costs one more tiny document from a Typst
//! pipeline that is already set up with the brand font embedded.
//!
//! The palette is the site's, not the resume's: this card is seen on a screen
//! beside the page it links to, whereas [`crate::style`] is a light theme for
//! print.

use portfolio_data::{CONFIG, OG_IMAGE_SIZE};

use crate::template::esc;
use crate::world::render_png;

/// Page size in typographic points. Rendered at [`SCALE`] to reach the published
/// [`OG_IMAGE_SIZE`]; laying out at half of it keeps the type sizes below in
/// ordinary point values rather than doubled ones.
const WIDTH_PT: f64 = 600.0;
const HEIGHT_PT: f64 = 315.0;

/// Device pixels per typographic point.
const SCALE: f64 = 2.0;

/// Compile-time proof that this layout produces the size the web app declares.
/// The two live in different crates — the renderer here, the `og:image:width` /
/// `og:image:height` there — and a card whose bytes disagree with its declared
/// dimensions is letterboxed or dropped by the consumer, silently.
const _: () = {
    assert!((WIDTH_PT * SCALE) as u32 == OG_IMAGE_SIZE.0);
    assert!((HEIGHT_PT * SCALE) as u32 == OG_IMAGE_SIZE.1);
};

// The site's dark palette (`apps/web/assets/input.css`). `--accent` is authored
// there in oklch, which is a wider gamut than PNG carries, so it is stated here
// as the sRGB a browser clamps it to — the same colour a visitor sees.
const BG: &str = "#0a0d14";
const FG: &str = "#e8ecf2";
const MUTED: &str = "#6b7689";
const LINE: &str = "#232936";
const ACCENT: &str = "#00b6ff";

/// Renders the card as PNG bytes.
///
/// `job_title` and `description` come from the caller's translations rather than
/// from [`CONFIG`] so the card speaks the same language as the page that
/// references it, should a per-locale card ever be wanted; today one English
/// card is published, matching the single global `og:image`.
pub(crate) fn render(job_title: &str, description: &str) -> Result<Vec<u8>, String> {
    render_png(build_typ(job_title, description), SCALE)
}

/// Builds the `.typ` source for the card.
///
/// The address across the top doubles as the accent element and as the one piece
/// of text that survives the aggressive downscaling some consumers apply to a
/// preview thumbnail.
fn build_typ(job_title: &str, description: &str) -> String {
    let host = CONFIG
        .url
        .trim_start_matches("https://")
        .trim_end_matches('/');

    format!(
        r#"#set page(width: {WIDTH_PT}pt, height: {HEIGHT_PT}pt, margin: 0pt, fill: rgb("{BG}"))
#set text(font: ("Inter", "Liberation Sans"), fill: rgb("{FG}"), lang: "en")
#set par(leading: 0.62em, spacing: 0pt)

#place(top + left, rect(width: 100%, height: 3pt, fill: rgb("{ACCENT}")))

#pad(x: 44pt, y: 40pt)[
  #text(size: 11pt, weight: "bold", tracking: 2.4pt, fill: rgb("{ACCENT}"))[{host}]

  #v(1fr)

  #text(size: 52pt, weight: "bold", tracking: -1.6pt)[{name}]

  #v(22pt)

  #text(size: 19pt, fill: rgb("{FG}"))[{job_title}]

  #v(1fr)

  #line(length: 100%, stroke: 0.75pt + rgb("{LINE}"))

  #v(10pt)

  #text(size: 12.5pt, fill: rgb("{MUTED}"))[{description}]
]
"#,
        host = esc(host),
        name = esc(CONFIG.full_name),
        job_title = esc(job_title),
        description = esc(description),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The published size is the box every consumer crops to.
    #[test]
    fn the_card_is_published_at_the_conventional_size() {
        assert_eq!(OG_IMAGE_SIZE, (1200, 630));
        // 1.91:1, the ratio Facebook, LinkedIn and Slack all letterbox into.
        let (w, h) = OG_IMAGE_SIZE;
        assert!((f64::from(w) / f64::from(h) - 1.91).abs() < 0.01);
    }

    /// Renders the real card and checks it is a PNG of the expected dimensions,
    /// read straight out of the IHDR chunk — the header a consumer parses.
    #[test]
    fn the_card_renders_to_a_png_of_that_size() {
        let png = render("Software Engineer", "A portfolio.").expect("the card renders");

        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "not a PNG");
        // IHDR payload starts at byte 16: width and height, big-endian u32s.
        let dimension = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
        assert_eq!((dimension(16), dimension(20)), OG_IMAGE_SIZE);
    }

    /// The card carries the facts it exists to show. Rendering is opaque once it
    /// is a bitmap, so the check is on the markup that produced it.
    #[test]
    fn the_card_is_built_from_the_shared_config() {
        let typ = build_typ("Software Engineer", "A portfolio.");

        assert!(typ.contains(&esc(CONFIG.full_name)), "{typ}");
        assert!(typ.contains("Software Engineer"), "{typ}");
        // The address, with the scheme stripped: a preview shows a host, not a URL.
        assert!(typ.contains("tim-schoenle.de"), "{typ}");
        assert!(!typ.contains("https://"), "{typ}");
    }
}
