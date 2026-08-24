//! Read-only view over one embedded translation file.
//!
//! A missing key panics instead of returning an `Option` nobody would have anything useful to do
//! with, so a drifted translation file fails the image build rather than emitting a resume with a
//! blank heading on it. `portfolio_data`'s `translation_key_sets_match` holds the two files to one
//! key set, so a key missing here is missing from both languages rather than one.

use std::error::Error;

use portfolio_data::{YearMonth, format_period};

/// One language's translations, with the two lookups the date formatter needs already resolved.
pub(crate) struct Translations {
    root: serde_json::Value,
    months: Vec<String>,
    present: String,
}

impl Translations {
    /// Parses a translation document and resolves the twelve month names and the "present"
    /// label out of it.
    ///
    /// Errors on malformed JSON. Panics if any of those thirteen keys is missing, for the reason
    /// in the module comment.
    pub(crate) fn parse(json: &str) -> Result<Self, Box<dyn Error>> {
        let root: serde_json::Value = serde_json::from_str(json)?;
        let mut t = Translations {
            root,
            months: Vec::new(),
            present: String::new(),
        };
        t.months = (1..=12)
            .map(|n| t.get(&format!("common.months.m{n}")))
            .collect();
        t.present = t.get("common.present");
        Ok(t)
    }

    /// Looks up a dotted key, panicking when it is absent or does not hold a string.
    pub(crate) fn get(&self, key: &str) -> String {
        let mut value = &self.root;
        for part in key.split('.') {
            value = &value[part];
        }
        value
            .as_str()
            .unwrap_or_else(|| panic!("missing translation key '{key}'"))
            .to_string()
    }

    /// A date range in this language, e.g. `Nov 2018 – Jan 2023`, open-ended when `end` is
    /// `None`.
    pub(crate) fn period(&self, start: YearMonth, end: Option<YearMonth>) -> String {
        format_period(start, end, &self.months, &self.present)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal but well-formed translation document: twelve months plus the
    /// "present" label and a couple of nested keys.
    fn sample() -> Translations {
        let json = r#"{
            "common": {
                "present": "Present",
                "months": {
                    "m1": "Jan", "m2": "Feb", "m3": "Mar", "m4": "Apr",
                    "m5": "May", "m6": "Jun", "m7": "Jul", "m8": "Aug",
                    "m9": "Sep", "m10": "Oct", "m11": "Nov", "m12": "Dec"
                }
            },
            "resume": { "summary": "Hello" }
        }"#;
        Translations::parse(json).expect("sample translations parse")
    }

    #[test]
    fn parse_rejects_invalid_json() {
        assert!(Translations::parse("{ not json").is_err());
    }

    #[test]
    fn get_resolves_dotted_keys() {
        let t = sample();
        assert_eq!(t.get("resume.summary"), "Hello");
        assert_eq!(t.get("common.months.m1"), "Jan");
    }

    #[test]
    #[should_panic(expected = "missing translation key")]
    fn get_panics_on_missing_key() {
        sample().get("resume.does_not_exist");
    }

    #[test]
    fn period_uses_localized_months_and_present_label() {
        let t = sample();
        assert_eq!(
            t.period(
                YearMonth {
                    year: 2018,
                    month: 11
                },
                Some(YearMonth {
                    year: 2023,
                    month: 1
                })
            ),
            "Nov 2018 – Jan 2023"
        );
        assert_eq!(
            t.period(
                YearMonth {
                    year: 2026,
                    month: 3
                },
                None
            ),
            "Mar 2026 – Present"
        );
    }
}
