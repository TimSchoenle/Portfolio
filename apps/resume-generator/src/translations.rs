//! Read-only view over one embedded translation file.

use std::error::Error;

use portfolio_data::{YearMonth, format_period};

pub(crate) struct Translations {
    root: serde_json::Value,
    months: Vec<String>,
    present: String,
}

impl Translations {
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

    /// Looks up a dotted key; the data crate's tests guarantee presence.
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
