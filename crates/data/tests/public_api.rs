//! Integration tests exercising the public `portfolio-data` API the way the
//! frontend and resume generator consume it. These run against the crate from
//! the outside (no `super::*`), so they only touch items that are genuinely
//! exported and stay a guard rail against accidental visibility regressions.

use std::collections::BTreeMap;

use portfolio_data::{
    CONFIG, EDUCATION, EXPERIENCE, LANGUAGES, MIN_CONFIDENCE, Quadrant, RESUME_FILES, Repo,
    ReposFile, ResumeFingerprints, SKILLS, YearMonth, experiences_sorted,
    format_period, format_period_years, lang_color, matrix_skills, resume_file,
};

fn months() -> Vec<String> {
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

// ---------- configuration ----------

#[test]
fn config_contact_facts_are_well_formed() {
    assert!(CONFIG.url.starts_with("https://"), "url must be https");
    assert!(
        CONFIG.email.contains('@'),
        "email must look like an address"
    );
    assert!(
        CONFIG.github.contains(CONFIG.github_username),
        "github url should embed the username"
    );
    assert!(!CONFIG.full_name.is_empty());
    assert!(!CONFIG.featured_repos.is_empty());
}

#[test]
fn languages_default_first() {
    assert_eq!(LANGUAGES, ["en", "de"]);
    assert_eq!(LANGUAGES[0], "en", "default language is listed first");
}

#[test]
fn resume_file_maps_languages_and_falls_back_to_english() {
    assert_eq!(resume_file("en"), RESUME_FILES[0].1);
    assert_eq!(resume_file("de"), RESUME_FILES[1].1);
    // Unknown languages fall back to the English file rather than panicking.
    assert_eq!(resume_file("fr"), RESUME_FILES[0].1);
}

// ---------- skills ----------

#[test]
fn matrix_skills_are_sorted_and_filtered() {
    let matrix = matrix_skills();
    assert!(!matrix.is_empty());

    // Strongest first.
    for pair in matrix.windows(2) {
        assert!(
            pair[0].confidence >= pair[1].confidence,
            "matrix skills must be sorted by descending confidence"
        );
    }

    // Radar-only and sub-threshold skills are excluded.
    for skill in &matrix {
        assert!(!skill.radar_only, "{} is radar-only", skill.name);
        assert!(
            skill.confidence >= MIN_CONFIDENCE,
            "{} is below the confidence floor",
            skill.name
        );
    }
}

#[test]
fn skill_levels_stay_in_the_one_to_five_band() {
    for skill in SKILLS {
        let level = skill.level();
        assert!(
            (1..=5).contains(&level),
            "{} mapped to out-of-range level {level}",
            skill.name
        );
        assert!((0.0..=1.0).contains(&skill.confidence), "{}", skill.name);
    }
}

#[test]
fn quadrants_are_distinct_and_labelled() {
    let all = Quadrant::all();
    assert_eq!(all.len(), 4);
    for (i, q) in all.iter().enumerate() {
        assert!(!q.i18n_key().is_empty());
        assert!(q.color().starts_with('#'));
        // No duplicate quadrants in `all()`.
        for other in &all[i + 1..] {
            assert_ne!(q, other);
        }
    }
}

// ---------- experience & education ----------

#[test]
fn experiences_sorted_keeps_every_entry() {
    let sorted = experiences_sorted();
    assert_eq!(sorted.len(), EXPERIENCE.len());

    // Ongoing roles (no end date) always precede ended ones.
    let first_ended = sorted.iter().position(|e| e.end.is_some());
    let last_ongoing = sorted.iter().rposition(|e| e.end.is_none());
    if let (Some(first_ended), Some(last_ongoing)) = (first_ended, last_ongoing) {
        assert!(last_ongoing < first_ended);
    }
}

#[test]
fn education_history_is_reverse_chronological() {
    assert!(!EDUCATION.is_empty());
    for pair in EDUCATION.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        assert!(
            (a.start.year, a.start.month) >= (b.start.year, b.start.month),
            "education must be newest-first"
        );
    }
}

#[test]
fn format_period_renders_localized_ranges() {
    let m = months();
    assert_eq!(
        format_period(
            YearMonth {
                year: 2018,
                month: 11
            },
            Some(YearMonth {
                year: 2023,
                month: 1
            }),
            &m,
            "Present",
        ),
        "Nov 2018 – Jan 2023"
    );
    assert_eq!(
        format_period(
            YearMonth {
                year: 2026,
                month: 3
            },
            None,
            &m,
            "Present",
        ),
        "Mar 2026 – Present"
    );
}

#[test]
fn format_period_years_collapses_same_year_and_open_ranges() {
    let start = YearMonth {
        year: 2018,
        month: 11,
    };
    assert_eq!(
        format_period_years(
            start,
            Some(YearMonth {
                year: 2023,
                month: 1
            }),
            "now"
        ),
        "2018 — 2023"
    );
    // Same start/end year collapses to a single year.
    assert_eq!(
        format_period_years(
            start,
            Some(YearMonth {
                year: 2018,
                month: 2
            }),
            "now"
        ),
        "2018"
    );
    assert_eq!(format_period_years(start, None, "now"), "2018 — now");
}

// ---------- repos.json schema ----------

fn sample_repo(name: &str) -> Repo {
    Repo {
        name: name.to_string(),
        full_name: format!("{}/{name}", CONFIG.github_username),
        description: Some("demo".to_string()),
        html_url: format!("https://github.com/{}/{name}", CONFIG.github_username),
        language: Some("Rust".to_string()),
        stargazers_count: 7,
        forks_count: 1,
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        topics: vec!["portfolio".to_string()],
        fork: false,
        archived: false,
        homepage: None,
    }
}

#[test]
fn is_featured_matches_config_case_insensitively() {
    let featured = CONFIG.featured_repos[0];
    assert!(sample_repo(&featured.to_uppercase()).is_featured());
    assert!(!sample_repo("definitely-not-featured").is_featured());
}

#[test]
fn repos_file_round_trips_through_json() {
    let file = ReposFile {
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        user: CONFIG.github_username.to_string(),
        repos: vec![sample_repo("Portfolio"), sample_repo("helm-charts")],
    };
    let json = serde_json::to_string(&file).expect("serializes");
    let back: ReposFile = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(file, back);
}

#[test]
fn repo_defaults_fill_optional_fields() {
    // Only the required fields are present; the rest must default.
    let json = r#"{
        "name": "minimal",
        "description": null,
        "html_url": "https://example.com/minimal"
    }"#;
    let repo: Repo = serde_json::from_str(json).expect("minimal repo deserializes");
    assert_eq!(repo.name, "minimal");
    assert_eq!(repo.full_name, "");
    assert_eq!(repo.stargazers_count, 0);
    assert!(repo.topics.is_empty());
    assert!(!repo.archived);
}

#[test]
fn lang_color_known_and_fallback() {
    assert_eq!(lang_color("Rust"), "#dea584");
    assert_eq!(lang_color("Java"), "#b07219");
    // Unknown languages get the neutral fallback swatch.
    assert_eq!(lang_color("Brainfuck"), "#9aa4b2");
}

// ---------- resume fingerprints ----------

#[test]
fn empty_fingerprints_report_no_digests() {
    let fp = ResumeFingerprints::default();
    assert!(fp.is_empty());
    assert!(fp.digest_for("en").is_none());
}

#[test]
fn fingerprints_look_up_digest_per_language() {
    let mut files = BTreeMap::new();
    files.insert(resume_file("en").to_string(), "abc123".to_string());

    let fp = ResumeFingerprints {
        algorithm: "SHA-256".to_string(),
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        files,
    };

    assert!(!fp.is_empty());
    assert_eq!(fp.digest_for("en"), Some("abc123"));
    assert!(fp.digest_for("de").is_none());
}
