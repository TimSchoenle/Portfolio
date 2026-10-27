//! The site languages: their translation files, and everything else that differs by language
//! without being prose.
//!
//! Every list here is derived from [`SITE_LANGUAGES`], so adding a language is one entry there
//! and one translation file.

/// The English translation file, embedded at compile time.
pub const I18N_EN: &str = include_str!("../i18n/en.json");
/// The German translation file, embedded at compile time.
pub const I18N_DE: &str = include_str!("../i18n/de.json");

/// Everything about one site language that is not prose.
///
/// The only values are the entries of [`SITE_LANGUAGES`]; look one up with [`language`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language {
    /// The primary language subtag, such as `en`.
    ///
    /// It is also the value of the `lang` cookie and the `?lang=` query parameter.
    pub code: &'static str,
    /// The `og:locale` value, such as `en_US`.
    pub og_locale: &'static str,
    /// The embedded translation file.
    pub translations: &'static str,
    /// The resume PDF's published file name, served under `/resume/`.
    ///
    /// It may be non-ASCII; browsers percent-encode it.
    pub resume_file: &'static str,
    /// The translation key labelling the resume download.
    pub resume_label_key: &'static str,
}

/// The site languages, the default first.
///
/// The default is what a request with no usable preference gets, and what the unparameterized
/// URLs serve to crawlers.
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

/// The language codes of [`SITE_LANGUAGES`], in the same order.
pub const LANGUAGES: [&str; SITE_LANGUAGES.len()] = {
    let mut codes = [""; SITE_LANGUAGES.len()];
    let mut i = 0;
    while i < codes.len() {
        codes[i] = SITE_LANGUAGES[i].code;
        i += 1;
    }
    codes
};

/// The resume PDF file name of each language, as `(code, file name)` pairs.
pub const RESUME_FILES: [(&str, &str); SITE_LANGUAGES.len()] = {
    let mut files = [("", ""); SITE_LANGUAGES.len()];
    let mut i = 0;
    while i < files.len() {
        files[i] = (SITE_LANGUAGES[i].code, SITE_LANGUAGES[i].resume_file);
        i += 1;
    }
    files
};

/// The name of the cookie and of the i18nrs storage entry holding the selected language.
///
/// The privacy notice documents it (the template in `legal/privacy.toml`, and the
/// published text a deployment mounts in its place), so a rename has to reach both.
pub const LANG_STORAGE_KEY: &str = "lang";

/// The query parameter naming a language explicitly, as in `/?lang=de`.
///
/// The same name as the cookie on purpose: both carry a [`Language::code`].
pub const LANG_QUERY_PARAM: &str = LANG_STORAGE_KEY;

/// [`LANG_STORAGE_KEY`] followed by `=`, the prefix of a `lang` cookie or query pair.
// A constant so the readers, which test every pair of a header or query, do not `format!` it.
pub const LANG_PAIR_PREFIX: &str = "lang=";

// `concat!` takes only literals, so nothing but this check stops a rename of one constant from
// leaving the other behind, which would read the cookie under a name nothing writes.
const _: () = {
    let key = LANG_STORAGE_KEY.as_bytes();
    let prefix = LANG_PAIR_PREFIX.as_bytes();
    assert!(prefix.len() == key.len() + 1);
    assert!(prefix[key.len()] == b'=');
    let mut i = 0;
    while i < key.len() {
        assert!(prefix[i] == key[i]);
        i += 1;
    }
};

/// Returns the `Set-Cookie` value that persists `code` for a year.
///
/// `SameSite=Lax` and no `Secure`: the value is a UI preference with no authority, and a
/// development server on plain HTTP has to be able to set it too.
///
/// ```
/// # use portfolio_data::lang_cookie;
/// assert!(lang_cookie("de").starts_with("lang=de; Path=/;"));
/// ```
#[must_use]
pub fn lang_cookie(code: &str) -> String {
    format!("{LANG_PAIR_PREFIX}{code}; Path=/; Max-Age=31536000; SameSite=Lax")
}

/// Returns the site language whose code equals `code` exactly, if any.
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

/// Returns the site language for `code`, or [`DEFAULT_LANGUAGE`] when it names none.
#[must_use]
pub fn language_or_default(code: &str) -> &'static Language {
    language(code).unwrap_or(DEFAULT_LANGUAGE)
}

/// Returns the language after `code` in [`SITE_LANGUAGES`], wrapping around.
///
/// This is what the language toggle switches to. An unknown `code` counts as the default.
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

/// Returns the resume file name for a language code, or the default language's.
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
