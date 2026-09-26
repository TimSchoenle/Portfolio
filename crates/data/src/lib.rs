//! Language-neutral portfolio data shared by the frontend and the resume generator.
//!
//! All user-visible prose lives in the embedded translation files ([`I18N_EN`],
//! [`I18N_DE`], listed per language in [`SITE_LANGUAGES`]); this crate only holds facts (names, dates, URLs, confidence
//! values) and the schemas for the build-time generated documents — `repos.json`
//! and the third-party license inventory in [`licenses`].
//!
//! Nothing here is read at run time. Every item is a compile-time constant or a schema for a
//! document produced during the image build, so changing any of it is a redeploy.

mod career;
mod language;
pub mod licenses;
pub mod profile;
mod repos;
mod resume;
mod site;
mod skills;

pub use career::*;
pub use language::*;
pub use repos::*;
pub use resume::*;
pub use site::*;
pub use skills::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::career::ym;
    use serde_json::Value;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    fn collect_keys(value: &Value, prefix: &str, keys: &mut BTreeSet<String>) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                    let path = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{prefix}.{k}")
                    };
                    collect_keys(v, &path, keys);
                }
            }
            Value::String(s) => {
                assert!(!s.trim().is_empty(), "empty translation for key '{prefix}'");
                keys.insert(prefix.to_string());
            }
            other => panic!("unsupported JSON value at '{prefix}': {other:?}"),
        }
    }

    fn keys_of(json: &str) -> BTreeSet<String> {
        let value: Value = serde_json::from_str(json).expect("translation file is valid JSON");
        let mut keys = BTreeSet::new();
        collect_keys(&value, "", &mut keys);
        keys
    }

    /// Every string in a translation document, keyed by its dotted path.
    fn strings_of(json: &str) -> BTreeMap<String, String> {
        fn walk(value: &Value, prefix: &str, out: &mut BTreeMap<String, String>) {
            match value {
                Value::Object(map) => {
                    for (k, v) in map {
                        let path = if prefix.is_empty() {
                            k.clone()
                        } else {
                            format!("{prefix}.{k}")
                        };
                        walk(v, &path, out);
                    }
                }
                Value::String(s) => {
                    out.insert(prefix.to_string(), s.clone());
                }
                other => panic!("unsupported JSON value at '{prefix}': {other:?}"),
            }
        }
        let value: Value = serde_json::from_str(json).expect("translation file is valid JSON");
        let mut out = BTreeMap::new();
        walk(&value, "", &mut out);
        out
    }

    /// The English translation of the role must be the one [`CONFIG`] publishes.
    ///
    /// `common.jobTitle` is what the hero and the resume PDF print; `CONFIG.job_title` is what
    /// schema.org, the profile API and `og:title` carry. They are the same fact in two
    /// representations — one localizable, one not — and they disagreed before this test existed.
    #[test]
    fn english_job_title_matches_config() {
        assert_eq!(
            strings_of(I18N_EN)
                .get("common.jobTitle")
                .map(String::as_str),
            Some(CONFIG.job_title),
            "en.json's common.jobTitle must equal CONFIG.job_title",
        );
    }

    /// German must translate the role rather than inherit the English one.
    #[test]
    fn german_job_title_is_translated() {
        let de = strings_of(I18N_DE);
        let role = de
            .get("common.jobTitle")
            .expect("de.json has common.jobTitle");
        assert_ne!(
            role, CONFIG.job_title,
            "de.json's common.jobTitle is still the English string",
        );
    }

    /// [`CONFIG::title`] and [`document_title`] must punctuate and order the same way.
    #[test]
    fn document_title_matches_config() {
        assert_eq!(document_title(CONFIG.job_title), CONFIG.title);
    }

    /// The stack a visitor reads in the hero and the stack a crawler reads in `meta keywords`
    /// must be the same stack.
    #[test]
    fn headline_tech_is_covered_by_keywords() {
        for tech in CONFIG.headline_tech {
            assert!(
                CONFIG.keywords.contains(tech),
                "headline tech '{tech}' is missing from CONFIG.keywords",
            );
        }
    }

    /// The description is cut at roughly 155 characters in a search result, and the sentence has
    /// to still say something there.
    #[test]
    fn description_survives_serp_truncation() {
        let chars = CONFIG.description.chars().count();
        assert!(
            (110..=160).contains(&chars),
            "description is {chars} characters; aim for 110-160",
        );
    }

    /// No two experience bullets may be byte-identical, in either language.
    ///
    /// A bullet repeated across roles reads as copy-paste on the resume and leaves one of the
    /// roles without content of its own.
    #[test]
    fn experience_bullets_are_distinct() {
        for (lang, json) in SITE_LANGUAGES.map(|l| (l.code, l.translations)) {
            let mut seen: BTreeMap<String, String> = BTreeMap::new();
            for (key, text) in strings_of(json) {
                let Some(rest) = key.strip_prefix("experience.entries.") else {
                    continue;
                };
                if !rest.contains(".bullets.") {
                    continue;
                }
                if let Some(first) = seen.insert(text.clone(), key.clone()) {
                    panic!("{lang}: {key} repeats {first} verbatim: {text:?}");
                }
            }
        }
    }

    /// The English copy is en-US throughout, so one page never mixes `containerised` with
    /// `containerized`.
    #[test]
    fn english_copy_uses_us_spelling() {
        // Whole words, not suffixes: `-ising` as a substring also matches `advertising`, which is
        // the American spelling and appears in the privacy policy.
        const BRITISH: [(&str, &str); 20] = [
            ("specialising", "specializing"),
            ("specialised", "specialized"),
            ("containerised", "containerized"),
            ("customised", "customized"),
            ("customisation", "customization"),
            ("standardised", "standardized"),
            ("standardising", "standardizing"),
            ("organisation", "organization"),
            ("organisational", "organizational"),
            ("optimisation", "optimization"),
            ("prioritise", "prioritize"),
            ("prioritised", "prioritized"),
            ("recognise", "recognize"),
            ("summarise", "summarize"),
            ("utilise", "utilize"),
            ("analyse", "analyze"),
            ("centre", "center"),
            ("licence", "license"),
            ("licences", "licenses"),
            ("modelling", "modeling"),
        ];
        let mut offences = Vec::new();
        for (key, text) in strings_of(I18N_EN) {
            let words: BTreeSet<String> = text
                .split(|c: char| !c.is_alphabetic())
                .filter(|w| !w.is_empty())
                .map(str::to_lowercase)
                .collect();
            for (british, american) in BRITISH {
                if words.contains(british) {
                    offences.push(format!("{key}: '{british}' -> '{american}'"));
                }
            }
        }
        assert!(
            offences.is_empty(),
            "en-GB spellings in en.json:\n  {}",
            offences.join("\n  "),
        );
    }

    /// No experience bullet may outgrow two printed lines.
    ///
    /// The cap is words rather than characters because the PDF sets them at one size in one
    /// column, so length in words is what decides the wrap. A bullet over it costs the fit ladder
    /// a rung: the generator condenses roles and then shrinks the type to win back the line, which
    /// is a worse trade than editing the sentence.
    #[test]
    fn experience_bullets_stay_within_two_lines() {
        const MAX_WORDS: usize = 24;
        let mut offences = Vec::new();
        for (lang, json) in SITE_LANGUAGES.map(|l| (l.code, l.translations)) {
            for (key, text) in strings_of(json) {
                let Some(rest) = key.strip_prefix("experience.entries.") else {
                    continue;
                };
                if !rest.contains(".bullets.") {
                    continue;
                }
                let words = text.split_whitespace().count();
                if words > MAX_WORDS {
                    offences.push(format!("{lang}/{key}: {words} words"));
                }
            }
        }
        assert!(
            offences.is_empty(),
            "bullets over {MAX_WORDS} words:\n  {}",
            offences.join("\n  "),
        );
    }

    /// Phrases that describe an attitude instead of a capability, banned from both languages.
    ///
    /// Each one claims a quality without evidence ("passionate", "team player") or undersells a
    /// skill the rest of the page demonstrates ("growing experience").
    #[test]
    fn copy_avoids_hedges() {
        const BANNED: [&str; 8] = [
            "passionate",
            "leidenschaftlich",
            "great software",
            "growing experience",
            "wachsende erfahrung",
            "a few things",
            "team player",
            "think outside",
        ];
        let mut offences = Vec::new();
        for (lang, json) in SITE_LANGUAGES.map(|l| (l.code, l.translations)) {
            for (key, text) in strings_of(json) {
                let lower = text.to_lowercase();
                for phrase in BANNED {
                    if lower.contains(phrase) {
                        offences.push(format!("{lang}/{key}: {phrase:?}"));
                    }
                }
            }
        }
        assert!(
            offences.is_empty(),
            "hedging copy:\n  {}",
            offences.join("\n  ")
        );
    }

    /// i18nrs falls back to an arbitrary language for missing keys, so both
    /// translation files must define exactly the same key set.
    #[test]
    fn translation_key_sets_match() {
        let en = keys_of(I18N_EN);
        let de = keys_of(I18N_DE);
        let only_en: Vec<_> = en.difference(&de).collect();
        let only_de: Vec<_> = de.difference(&en).collect();
        assert!(
            only_en.is_empty() && only_de.is_empty(),
            "translation keys differ:\n  only in en: {only_en:?}\n  only in de: {only_de:?}"
        );
    }

    #[test]
    fn experience_entries_have_translations() {
        let en = keys_of(I18N_EN);
        for e in EXPERIENCE {
            for field in ["role", "org"] {
                let key = format!("experience.entries.{}.{field}", e.id);
                assert!(en.contains(&key), "missing key '{key}'");
            }
            for n in 1..=e.bullet_count {
                let bullet = format!("experience.entries.{}.bullets.b{n}", e.id);
                assert!(en.contains(&bullet), "missing key '{bullet}'");
            }
        }
    }

    #[test]
    fn experiences_sorted_prioritizes_ongoing_then_recent_start() {
        let order = experiences_sorted();
        // No entry lost or duplicated by sorting.
        assert_eq!(order.len(), EXPERIENCE.len());

        // All ongoing roles (no end) come before any ended one.
        let first_ended = order.iter().position(|e| e.end.is_some());
        let last_ongoing = order.iter().rposition(|e| e.end.is_none());
        if let (Some(first_ended), Some(last_ongoing)) = (first_ended, last_ongoing) {
            assert!(
                last_ongoing < first_ended,
                "an ended role precedes an ongoing one"
            );
        }

        // Within each group (ongoing, then ended) the start dates are
        // non-increasing, so each group is reverse-chronological by start.
        for pair in order.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if a.end.is_none() == b.end.is_none() {
                assert!(
                    (a.start.year, a.start.month) >= (b.start.year, b.start.month),
                    "{} (start {:?}) must not precede {} (start {:?})",
                    a.id,
                    a.start,
                    b.id,
                    b.start,
                );
            }
        }

        // The "two most recent roles" the resume keeps in full are the two
        // ongoing engagements, newest start first.
        assert_eq!(order[0].id, "sixtwenty");
        assert_eq!(order[1].id, "self-employed");
        // Ongoing roles outrank ended ones that started later: the Independent
        // role (ongoing, Oct 2021) precedes Mineplex Studios (ended, Aug 2023).
        let self_pos = order.iter().position(|e| e.id == "self-employed").unwrap();
        let studios_pos = order
            .iter()
            .position(|e| e.id == "mineplex-studios")
            .unwrap();
        assert!(self_pos < studios_pos);
    }

    #[test]
    fn education_entries_have_translations() {
        let en = keys_of(I18N_EN);
        for e in EDUCATION {
            for field in ["degree", "institution"] {
                let key = format!("resume.education.{}.{field}", e.id);
                assert!(en.contains(&key), "missing key '{key}'");
            }
        }
    }

    #[test]
    fn months_are_complete() {
        let en = keys_of(I18N_EN);
        for n in 1..=12 {
            let key = format!("common.months.m{n}");
            assert!(en.contains(&key), "missing key '{key}'");
        }
    }

    #[test]
    fn skill_levels_in_range() {
        for skill in SKILLS {
            assert!((0.0..=1.0).contains(&skill.confidence), "{}", skill.name);
            assert!((1..=5).contains(&skill.level()), "{}", skill.name);
        }
    }

    #[test]
    fn period_formatting() {
        let months: Vec<String> = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        assert_eq!(
            format_period(ym(2018, 11), Some(ym(2023, 1)), &months, "Present"),
            "Nov 2018 – Jan 2023"
        );
        assert_eq!(
            format_period(ym(2026, 3), None, &months, "Present"),
            "Mar 2026 – Present"
        );
    }
}
