//! The web manifest's raster icons, rasterized from the site's own `favicon.svg`.
//!
//! Browsers install a site as an app only when its manifest lists a 192 × 192 and a 512 × 512
//! raster icon; an SVG alone does not qualify. Rendering them here, from the same SVG the page
//! uses, keeps them a function of that file rather than two committed copies that drift from it.

use portfolio_data::APP_ICONS;

use crate::world::{Asset, render_png};

/// The favicon, the one source every icon is drawn from.
const FAVICON: &[u8] = include_bytes!("../../web/assets/favicon.svg");

/// The name the favicon is registered under in the Typst world.
const FAVICON_NAME: &str = "favicon.svg";

/// The site background (`--bg` in `apps/web/assets/input.css`), matching the manifest's
/// `background_color` so the splash screen and the icon share one colour.
const BACKGROUND: &str = "#0a0d14";

/// Every icon in [`APP_ICONS`], as `(file name, PNG bytes)`.
///
/// # Errors
///
/// A description of the first icon that failed to compile or encode.
pub(crate) fn render_all() -> Result<Vec<(&'static str, Vec<u8>)>, String> {
    let assets = [Asset {
        name: FAVICON_NAME.to_owned(),
        bytes: typst::foundations::Bytes::new(FAVICON),
    }];
    APP_ICONS
        .iter()
        .map(|&(size, name)| {
            // One point per pixel, so the page size in points is the icon size in pixels.
            render_png(markup(size), 1.0, &assets)
                .map(|png| (name, png))
                .map_err(|err| format!("{name}: {err}"))
        })
        .collect()
}

/// A square page of `size` points on the site background with the mark centred, inset so it
/// stays inside the circle a launcher may crop the icon to.
fn markup(size: u32) -> String {
    format!(
        "#set page(width: {size}pt, height: {size}pt, margin: 0pt, fill: rgb(\"{BACKGROUND}\"))\n\
         #align(center + horizon, image(\"{FAVICON_NAME}\", width: 70%))\n"
    )
}

#[cfg(test)]
mod tests {
    use super::render_all;
    use portfolio_data::APP_ICONS;

    /// The PNG header's width and height, big-endian at bytes 16..24.
    fn dimensions(png: &[u8]) -> (u32, u32) {
        let word = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
        (word(16), word(20))
    }

    #[test]
    fn every_icon_renders_at_its_declared_size() {
        let icons = render_all().expect("the icons render");
        assert_eq!(icons.len(), APP_ICONS.len());
        for ((name, png), (size, expected)) in icons.iter().zip(APP_ICONS) {
            assert_eq!(*name, expected);
            assert_eq!(dimensions(png), (size, size), "{name}");
        }
    }
}
