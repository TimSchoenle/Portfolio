//! The site languages: their translation files, and everything else that differs by
//! language without being prose.

/// English translations, embedded at compile time.
pub const I18N_EN: &str = include_str!("../i18n/en.json");
/// German translations, embedded at compile time.
pub const I18N_DE: &str = include_str!("../i18n/de.json");

/// Everything about one site language that is not prose.
///
/// Adding a language is one entry in [`SITE_LANGUAGES`] and one translation file; every list
/// below and every consumer in the web app and the resume generator is derived from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language {
    /// The primary language subtag (`en`), which is also the value of the `lang` cookie and
    /// the `?lang=` query parameter.
    pub code: &'static str,
    /// The `og:locale` value (`en_US`).
    pub og_locale: &'static str,
    /// The embedded translation file.
    pub translations: &'static str,
    /// The resume PDF's published file name, served under `/resume/`. Non-ASCII on purpose;
    /// browsers percent-encode it.
    pub resume_file: &'static str,
    /// The translation key labelling the resume download.
    pub resume_label_key: &'static str,
}

/// The site languages. The first entry is the default: what a request with no usable
/// preference gets, and what the unparameterized URLs serve to crawlers.
pub const SITE_LANGUAGES: [Language; 2] = [
    Language {
        code: "en",
        og_locale: "en_US",
        translations: I18N_EN,
        resume_file: "Tim-Schönle-Resume.pdf",
        resume_label_key: "contact.resumeEn",
    },
    Language {
        code: "de",
        og_locale: "de_DE",
        translations: I18N_DE,
        resume_file: "Tim-Schönle-Lebenslauf.pdf",
        resume_label_key: "contact.resumeDe",
    },
];

/// The default language, `SITE_LANGUAGES[0]`.
pub const DEFAULT_LANGUAGE: &Language = &SITE_LANGUAGES[0];

/// Language codes supported by the site, in [`SITE_LANGUAGES`] order. The first entry is the
/// default.
pub const LANGUAGES: [&str; SITE_LANGUAGES.len()] = {
    let mut codes = [""; SITE_LANGUAGES.len()];
    let mut i = 0;
    while i < codes.len() {
        codes[i] = SITE_LANGUAGES[i].code;
        i += 1;
    }
    codes
};

/// Resume PDF file names per language code, served under `/resume/`.
pub const RESUME_FILES: [(&str, &str); SITE_LANGUAGES.len()] = {
    let mut files = [("", ""); SITE_LANGUAGES.len()];
    let mut i = 0;
    while i < files.len() {
        files[i] = (SITE_LANGUAGES[i].code, SITE_LANGUAGES[i].resume_file);
        i += 1;
    }
    files
};

/// The site language whose code equals `code` exactly, if any.
///
/// ```
/// # use portfolio_data::language;
/// assert_eq!(language("de").map(|l| l.og_locale), Some("de_DE"));
/// assert!(language("fr").is_none());
/// ```
#[must_use]
pub fn language(code: &str) -> Option<&'static Language> {
    SITE_LANGUAGES.iter().find(|lang| lang.code == code)
}

/// The site language for `code`, or [`DEFAULT_LANGUAGE`] when it names none.
#[must_use]
pub fn language_or_default(code: &str) -> &'static Language {
    language(code).unwrap_or(DEFAULT_LANGUAGE)
}

/// The language after `code` in [`SITE_LANGUAGES`], wrapping around: what the language toggle
/// switches to.
///
/// ```
/// # use portfolio_data::next_language;
/// assert_eq!(next_language("en").code, "de");
/// assert_eq!(next_language("de").code, "en");
/// assert_eq!(next_language("fr").code, "de");
/// ```
#[must_use]
pub fn next_language(code: &str) -> &'static Language {
    let index = SITE_LANGUAGES
        .iter()
        .position(|lang| lang.code == code)
        .unwrap_or(0);
    &SITE_LANGUAGES[(index + 1) % SITE_LANGUAGES.len()]
}

/// The resume file name for a language code, falling back to English.
///
/// ```
/// # use portfolio_data::resume_file;
/// assert_eq!(resume_file("de"), "Tim-Schönle-Lebenslauf.pdf");
/// assert_eq!(resume_file("fr"), resume_file("en"));
/// ```
#[must_use]
pub fn resume_file(lang: &str) -> &'static str {
    language_or_default(lang).resume_file
}
