//! The embedded-font Typst [`World`] and the compile → PDF helpers.
//!
//! The generator ships its own fonts (Inter, with Liberation Sans as a
//! metric-compatible fallback) and a single in-memory main source, so
//! compilation is fully self-contained: no font discovery, package downloads or
//! filesystem access. Typst subsets the embedded fonts and writes a tagged,
//! standard PDF 1.7 (RGB) with live `/URI` link annotations.

use typst::diag::{FileError, FileResult, SourceDiagnostic, Warned};
use typst::foundations::{Bytes, Datetime, Duration, Smart};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::{LazyHash, Scalar};
use typst::{Library, LibraryExt, World};
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards};
use typst_render::RenderOptions;

// Inter (SIL OFL) — the brand family. One face per file (index 0).
const INTER_REGULAR: &[u8] = include_bytes!("../fonts/Inter-Regular.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../fonts/Inter-Bold.ttf");
const INTER_ITALIC: &[u8] = include_bytes!("../fonts/Inter-Italic.ttf");
const INTER_BOLD_ITALIC: &[u8] = include_bytes!("../fonts/Inter-BoldItalic.ttf");

// Liberation Sans (SIL OFL) — last-resort metric-compatible fallback only.
const REGULAR: &[u8] = include_bytes!("../fonts/LiberationSans-Regular.ttf");
const BOLD: &[u8] = include_bytes!("../fonts/LiberationSans-Bold.ttf");
const ITALIC: &[u8] = include_bytes!("../fonts/LiberationSans-Italic.ttf");
const BOLD_ITALIC: &[u8] = include_bytes!("../fonts/LiberationSans-BoldItalic.ttf");

/// A minimal [`World`]: the Typst standard library, the four embedded faces and
/// a single detached main source.
struct ResumeWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    source: Source,
}

impl ResumeWorld {
    fn new(markup: String) -> Self {
        let fonts: Vec<Font> = [
            INTER_REGULAR,
            INTER_BOLD,
            INTER_ITALIC,
            INTER_BOLD_ITALIC,
            REGULAR,
            BOLD,
            ITALIC,
            BOLD_ITALIC,
        ]
        .into_iter()
        .filter_map(|bytes| Font::new(Bytes::new(bytes), 0))
        .collect();
        let book = FontBook::from_fonts(&fonts);
        ResumeWorld {
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(book),
            fonts,
            source: Source::detached(markup),
        }
    }
}

impl World for ResumeWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(std::path::PathBuf::from("resume.typ")))
        }
    }

    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        // Self-contained: the document references no external files.
        Err(FileError::NotFound(std::path::PathBuf::from("resume.typ")))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        // The document sets `date: none`, so `today` is never consulted.
        None
    }
}

/// Compiles Typst `markup` and exports it as a tagged, standard **PDF 1.7**
/// (RGB) with subsetted fonts and live link annotations (F1–F4), returning the
/// rendered **page count** alongside the bytes.
///
/// The compiled document type [`typst_layout::PagedDocument`] is not re-exported
/// by the `typst` umbrella crate, so it is pulled directly from `typst-layout`.
pub(crate) fn render(markup: String) -> Result<(usize, Vec<u8>), String> {
    let world = ResumeWorld::new(markup);
    let Warned { output, .. } = typst::compile(&world);
    let document: PagedDocument = output.map_err(|diags| diagnostics(&diags))?;
    let pages = document.pages().len();

    let standards = PdfStandards::new(&[PdfStandard::V_1_7]).map_err(|e| format!("{e:?}"))?;
    let options = PdfOptions {
        ident: Smart::Auto,
        creator: Smart::Auto,
        timestamp: None,
        page_ranges: None,
        standards,
        // Tagged PDF (structure tree + marked content) for accessibility and
        // reliable reading order in resume parsers.
        tagged: true,
        pretty: false,
    };
    let bytes = typst_pdf::pdf(&document, &options).map_err(|diags| diagnostics(&diags))?;
    Ok((pages, bytes))
}

/// Compiles Typst `markup` and rasterizes its first page to a PNG at
/// `pixel_per_pt` device pixels per typographic point.
///
/// A typographic point is 1/72 inch and the caller states its page size in
/// points, so the two together fix the output resolution exactly: a
/// 600pt × 315pt page at 2.0 is a 1200 × 630 image, which is what the social
/// card needs. Sharing [`ResumeWorld`] means the card is laid out with the same
/// embedded brand font as the resume and needs no toolchain of its own.
pub(crate) fn render_png(markup: String, pixel_per_pt: f64) -> Result<Vec<u8>, String> {
    let world = ResumeWorld::new(markup);
    let Warned { output, .. } = typst::compile(&world);
    let document: PagedDocument = output.map_err(|diags| diagnostics(&diags))?;
    let page = document
        .pages()
        .first()
        .ok_or_else(|| "the document produced no pages".to_string())?;

    let options = RenderOptions {
        pixel_per_pt: Scalar::new(pixel_per_pt),
        // The card is a screen asset with no trim, so there is no bleed to keep.
        render_bleed: false,
    };
    typst_render::render(page, &options)
        .encode_png()
        .map_err(|err| format!("PNG encoding failed: {err}"))
}

fn diagnostics(diags: &[SourceDiagnostic]) -> String {
    diags
        .iter()
        .map(|d| d.message.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}
