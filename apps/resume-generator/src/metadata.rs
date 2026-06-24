//! Post-processing of the rendered PDF's document-info dictionary.

use std::error::Error;

use portfolio_data::CONFIG;

/// Rewrites the document-info strings as UTF-16BE (with BOM).
///
/// printpdf 0.3 stores the title as raw UTF-8 inside a PDFDocEncoding string,
/// which viewers display as mojibake for non-ASCII ("SchÃ¶nle"). Re-encoding
/// through lopdf also rebuilds the xref correctly.
pub(crate) fn fix_metadata(bytes: &[u8], title: &str) -> Result<Vec<u8>, Box<dyn Error>> {
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

/// Lowercase hex encoding of a byte slice (for SHA-256 digests).
pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
