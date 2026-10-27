//! The header band's QR code, as Typst vector graphics.
//!
//! The modules are emitted as filled rectangles rather than a raster image, so
//! the code stays crisp at any print resolution, adds no image data to the PDF
//! and needs nothing from the [`world`](crate::world) beyond what is already
//! there. Runs of adjacent dark modules on a row collapse into one rectangle,
//! which is what keeps a 25 × 25 grid down to a couple of hundred draw calls.

use std::fmt::Write as _;

use qrcode::{Color, EcLevel, QrCode};

use crate::style::{self, Layout};

/// Builds the QR module field for `url` as a Typst box, sized
/// [`style::QR_SIZE`] square including its quiet zone.
///
/// # Errors
///
/// Returns the encoder's message when `url` does not fit any QR version — for
/// the site URL this cannot happen, but the generator would rather fail the
/// image build than emit a sheet with a blank square in the header.
pub(crate) fn field(l: &Layout, url: &str) -> Result<String, String> {
    let code = QrCode::with_error_correction_level(url.as_bytes(), EcLevel::M)
        .map_err(|err| format!("cannot encode {url:?} as a QR code: {err}"))?;
    let colors = code.to_colors();
    let width = code.width();

    // The quiet zone is drawn inside the field rather than around it: the frame
    // the header puts behind the code is white too, so the margin the spec asks
    // for is there either way, and the field keeps the size the design gives it.
    let span = width + 2 * style::QR_QUIET_MODULES as usize;
    #[expect(
        clippy::cast_precision_loss,
        reason = "a QR side is at most 177 modules"
    )]
    let unit = style::QR_SIZE / span as f64;
    let at = |modules: usize| {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a QR side is at most 177 modules"
        )]
        let modules = modules as f64;
        l.pt((modules + f64::from(style::QR_QUIET_MODULES)) * unit)
    };

    let mut out = format!(
        "#box(width: {side}, height: {side})[",
        side = l.pt(style::QR_SIZE)
    );
    for (y, row) in colors.chunks_exact(width).enumerate() {
        let mut x = 0;
        while x < width {
            if row[x] != Color::Dark {
                x += 1;
                continue;
            }
            let start = x;
            while x < width && row[x] == Color::Dark {
                x += 1;
            }
            #[expect(
                clippy::cast_precision_loss,
                reason = "a QR side is at most 177 modules"
            )]
            let run = (x - start) as f64;
            write!(
                out,
                "#place(top + left, dx: {dx}, dy: {dy}, \
                 rect(width: {w}, height: {h}, fill: rgb(\"{ink}\"), stroke: none, inset: 0pt))",
                dx = at(start),
                dy = at(y),
                w = l.pt(run * unit),
                h = l.pt(unit),
                ink = style::INK,
            )
            .expect("a String never fails to grow");
        }
    }
    out.push(']');
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> Layout {
        Layout {
            scale: 1.0,
            dense: false,
        }
    }

    #[test]
    fn encodes_the_site_url_into_a_closed_typst_box() {
        let markup = field(&layout(), portfolio_data::CONFIG.url).expect("the site URL encodes");
        assert!(markup.starts_with("#box(width: 37.500pt, height: 37.500pt)["));
        assert!(markup.ends_with(']'));
        assert!(markup.contains("#place(top + left"));
    }

    #[test]
    fn every_module_stays_inside_the_field() {
        let l = layout();
        let markup = field(&l, portfolio_data::CONFIG.url).expect("the site URL encodes");
        let side = style::QR_SIZE * 0.75;
        for offset in markup.split("dx: ").skip(1) {
            let (value, _) = offset.split_once("pt").expect("a pt literal");
            assert!(value.parse::<f64>().expect("a number") < side);
        }
    }

    #[test]
    fn scale_shrinks_the_field_with_the_rest_of_the_sheet() {
        let half = Layout {
            scale: 0.5,
            dense: false,
        };
        let markup = field(&half, portfolio_data::CONFIG.url).expect("the site URL encodes");
        assert!(markup.starts_with("#box(width: 18.750pt, height: 18.750pt)["));
    }

    #[test]
    fn refuses_a_payload_no_qr_version_can_hold() {
        let too_long = "x".repeat(8000);
        assert!(field(&layout(), &too_long).is_err());
    }
}
