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
