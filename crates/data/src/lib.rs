//! Language-neutral portfolio data shared by the frontend and the resume generator.
//!
//! All user-visible prose lives in the embedded translation files ([`I18N_EN`],
//! [`I18N_DE`]); this crate only holds facts (names, dates, URLs, confidence
//! values) and the schema for `repos.json`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod profile;

/// English translations, embedded at compile time.
pub const I18N_EN: &str = include_str!("../i18n/en.json");
/// German translations, embedded at compile time.
pub const I18N_DE: &str = include_str!("../i18n/de.json");

/// Language codes supported by the site. The first entry is the default.
pub const LANGUAGES: [&str; 2] = ["en", "de"];

/// Resume PDF file names per language, served under `/resume/`. The non-ASCII
/// names are intentional; browsers percent-encode them in URLs.
pub const RESUME_FILES: [(&str, &str); 2] = [
    ("en", "Tim-Schönle-Resume.pdf"),
    ("de", "Tim-Schönle-Lebenslauf.pdf"),
];

/// The resume file name for a language code, falling back to English.
pub fn resume_file(lang: &str) -> &'static str {
    RESUME_FILES
        .iter()
        .find(|(l, _)| *l == lang)
        .map(|(_, f)| *f)
        .unwrap_or(RESUME_FILES[0].1)
}

/// A keyless Sigstore signature appended to a resume PDF.
///
/// Produced on CI by the [`pdf-sign`](https://github.com/0x77dev/pdf-sign) tool
/// and recorded so the signer identity can be shown next to the fingerprint and
/// the resume verified against the public transparency log.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResumeSignature {
    /// Signing backend that produced the signature (e.g. "sigstore").
    pub backend: String,
    /// OIDC identity bound to the signing certificate — on GitHub Actions the
    /// signing workflow ref, e.g.
    /// `https://github.com/<owner>/<repo>/.github/workflows/release-please.yaml@refs/heads/main`.
    pub identity: String,
    /// OIDC issuer that vouched for the identity, e.g.
    /// `https://token.actions.githubusercontent.com`.
    pub issuer: String,
    /// Rekor transparency-log entry URL, when the signing tool reports one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rekor_log_url: Option<String>,
    /// RFC 3339 timestamp of when the PDF was signed.
    pub signed_at: String,
}

/// SHA-256 checksums of the generated resume PDFs.
///
/// Written by the resume generator as `resume-fingerprint.json` and embedded
/// into the frontend at build time, where it is shown on the contact card so a
/// downloaded resume can be verified against its published digest. Sigstore
/// signatures (when signed on CI) are recorded in
/// [`signatures`](Self::signatures).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResumeFingerprints {
    /// Hash algorithm used for the digests (e.g. "SHA-256").
    pub algorithm: String,
    /// RFC 3339 timestamp of when the manifest was generated.
    pub generated_at: String,
    /// Resume file name (e.g. "Tim-Schönle-Resume.pdf") -> hex digest.
    pub files: BTreeMap<String, String>,
    /// Resume file name -> Sigstore signature, for PDFs signed on CI. Empty on
    /// dev builds (no signing token), so the field is omitted from the JSON.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub signatures: BTreeMap<String, ResumeSignature>,
}

impl ResumeFingerprints {
    /// `true` when no resume digests are present (e.g. dev builds without
    /// generated resumes).
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Digest of the resume for the given language, if present.
    pub fn digest_for(&self, lang: &str) -> Option<&str> {
        self.files.get(resume_file(lang)).map(String::as_str)
    }

    /// Sigstore signature of the resume for the given language, if present.
    pub fn signature_for(&self, lang: &str) -> Option<&ResumeSignature> {
        self.signatures.get(resume_file(lang))
    }
}

// ---------- Site configuration ----------

pub struct ExternalService {
    pub name: &'static str,
    pub address: &'static str,
    pub policy_url: &'static str,
}

pub struct Legal {
    /// Postal address used in the imprint and as GDPR controller address.
    pub address_lines: &'static [&'static str],
    pub vat_id: &'static str,
    /// Second contact channel required by the imprint service.
    pub second_contact_url: &'static str,
    pub hosting: ExternalService,
    pub cloudflare: ExternalService,
    pub log_retention_days: u32,
    /// ISO dates shown as "last updated" on the legal pages.
    pub imprint_last_change: &'static str,
    pub privacy_last_change: &'static str,
    /// Supervisory authority for data-protection complaints.
    pub authority_url: &'static str,
    /// EU online dispute resolution platform.
    pub odr_url: &'static str,
}

pub struct Config {
    pub full_name: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    /// Role on its own, e.g. for schema.org `jobTitle` (the full document
    /// `title` is `"{full_name} — {job_title}"`).
    pub job_title: &'static str,
    /// Contact country exposed by the profile API and structured metadata.
    pub location: &'static str,
    pub email: &'static str,
    pub url: &'static str,
    pub github: &'static str,
    pub github_username: &'static str,
    pub linkedin: &'static str,
    pub repository: &'static str,
    pub description: &'static str,
    pub keywords: &'static [&'static str],
    pub featured_repos: &'static [&'static str],
    /// Repositories that must never appear in `repos.json`, regardless of their
    /// activity. Matched case-insensitively by name when listing all of the
    /// user's repositories in `update-repos`.
    pub blacklisted_repos: &'static [&'static str],
    pub legal: Legal,
}

pub const CONFIG: Config = Config {
    full_name: "Tim Schönle",
    name: "Tim",
    title: "Tim Schönle — Software Developer",
    job_title: "Software Developer",
    location: "Germany",
    email: "contact@tim-schoenle.de",
    url: "https://tim-schoenle.de",
    github: "https://github.com/timschoenle",
    github_username: "timschoenle",
    linkedin: "https://www.linkedin.com/in/tim-schoenle",
    repository: "https://github.com/timschoenle/Portfolio",
    description: "Portfolio of Tim Schönle — Software Developer specializing in Java, \
                  Rust and Next.js. Open-source contributor and passionate about building \
                  great software.",
    keywords: &[
        "Tim",
        "Software Developer",
        "Java",
        "Rust",
        "Next.js",
        "Portfolio",
        "Open Source",
        "Germany",
    ],
    featured_repos: &[
        "cloudflare-access-webhook-redirect",
        "s3-bucket-perma-link",
        "Portfolio",
        "helm-charts",
    ],
    blacklisted_repos: &["TimSchoenle", "actions-testing"],
    legal: Legal {
        address_lines: &[
            "tim-schoenle.de – Tim Schönle",
            "c/o Online-Impressum.de #5279",
            "Europaring 90",
            "53757 Sankt Augustin",
        ],
        vat_id: "DE347101415",
        second_contact_url: "https://mein.online-impressum.de/tim-schoenle-de/#Zweiter_Kontaktweg",
        hosting: ExternalService {
            name: "netcup GmbH",
            address: "Daimlerstraße 25, 76185 Karlsruhe, Germany",
            policy_url: "https://www.netcup.de/kontakt/datenschutzerklaerung.php",
        },
        cloudflare: ExternalService {
            name: "Cloudflare, Inc.",
            address: "101 Townsend St, San Francisco, CA 94107, USA",
            policy_url: "https://www.cloudflare.com/privacypolicy/",
        },
        log_retention_days: 7,
        imprint_last_change: "2026-06-10",
        privacy_last_change: "2026-06-10",
        authority_url: "https://www.baden-wuerttemberg.datenschutz.de",
        odr_url: "https://ec.europa.eu/consumers/odr/",
    },
};

// ---------- Skills ----------

/// Skills below this confidence are hidden from the matrix and the resume.
pub const MIN_CONFIDENCE: f32 = 0.6;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Quadrant {
    Languages,
    Frameworks,
    Build,
    Infra,
}

impl Quadrant {
    /// Translation key for the quadrant label.
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Quadrant::Languages => "skills.languages",
            Quadrant::Frameworks => "skills.frameworks",
            Quadrant::Build => "skills.build",
            Quadrant::Infra => "skills.infrastructure",
        }
    }

    /// Quadrant colors from the v4 design (radar-v3).
    pub fn color(&self) -> &'static str {
        match self {
            Quadrant::Languages => "#60a5fa",
            Quadrant::Frameworks => "#22d3ee",
            Quadrant::Build => "#34d399",
            Quadrant::Infra => "#a78bfa",
        }
    }

    pub fn all() -> [Quadrant; 4] {
        [
            Quadrant::Languages,
            Quadrant::Frameworks,
            Quadrant::Build,
            Quadrant::Infra,
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Skill {
    pub name: &'static str,
    pub quadrant: Quadrant,
    /// 0.0..=1.0, mirrored from the original portfolio's skills data.
    pub confidence: f32,
    /// Radar-only skills appear as radar scatter but not in the matrix/resume.
    pub radar_only: bool,
}

impl Skill {
    /// Proficiency level 1..=5 derived from confidence.
    pub fn level(&self) -> u8 {
        ((self.confidence * 5.0).round() as u8).clamp(1, 5)
    }
}

const fn s(name: &'static str, quadrant: Quadrant, confidence: f32) -> Skill {
    Skill {
        name,
        quadrant,
        confidence,
        radar_only: false,
    }
}

const fn r(name: &'static str, quadrant: Quadrant, confidence: f32) -> Skill {
    Skill {
        name,
        quadrant,
        confidence,
        radar_only: true,
    }
}

/// Full skill inventory, mirrored from the original portfolio's `skills.ts`.
pub const SKILLS: &[Skill] = &{
    use Quadrant::{Build, Frameworks, Infra, Languages};
    [
        // languages
        s("Java", Languages, 0.95),
        s("Rust", Languages, 0.70),
        s("TypeScript", Languages, 0.60),
        s("SQL", Languages, 0.85),
        s("WebAssembly (WASM)", Languages, 0.65),
        s("JavaScript", Languages, 0.50),
        s("Kotlin", Languages, 0.55),
        s("C#", Languages, 0.35),
        s("Python", Languages, 0.30),
        s("Lua", Languages, 0.25),
        s("Go", Languages, 0.20),
        s("C++", Languages, 0.10),
        r("Markdown", Languages, 0.70),
        r("Dockerfile", Languages, 0.75),
        r("Regular Expressions", Languages, 0.70),
        r("TOML", Languages, 0.60),
        r("Bash / Shell", Languages, 0.50),
        r("HTML", Languages, 0.50),
        r("CSS", Languages, 0.45),
        r("Properties / INI", Languages, 0.50),
        // frameworks
        s("Spring Boot", Frameworks, 0.86),
        s("gRPC", Frameworks, 0.82),
        s("PaperMC", Frameworks, 0.85),
        s("Next.js", Frameworks, 0.80),
        s("React", Frameworks, 0.76),
        s("Tailwind CSS", Frameworks, 0.75),
        s("Yew", Frameworks, 0.75),
        s("Node.js", Frameworks, 0.65),
        r("Bukkit API", Frameworks, 0.95),
        r("Spigot API", Frameworks, 0.95),
        r("next-intl", Frameworks, 0.75),
        r("Zod", Frameworks, 0.72),
        r("shadcn/ui", Frameworks, 0.70),
        r("Lucide React", Frameworks, 0.70),
        r("Pino", Frameworks, 0.65),
        r("Tokio", Frameworks, 0.60),
        r("Serwist", Frameworks, 0.60),
        r("Express", Frameworks, 0.60),
        r("Webhooks", Frameworks, 0.60),
        r("Octokit", Frameworks, 0.60),
        r("React Query", Frameworks, 0.55),
        r("reqwest", Frameworks, 0.55),
        r("React-PDF", Frameworks, 0.55),
        r("Prisma", Frameworks, 0.50),
        r("Radix UI", Frameworks, 0.50),
        r("ratatui", Frameworks, 0.50),
        r("NextAuth.js", Frameworks, 0.45),
        r("Axum", Frameworks, 0.45),
        r("Actix Web", Frameworks, 0.45),
        r("tRPC", Frameworks, 0.40),
        r("sqlx", Frameworks, 0.40),
        r("Lineicons", Frameworks, 0.40),
        r("wasm-bindgen", Frameworks, 0.35),
        r("Rocket", Frameworks, 0.30),
        // build & tools
        s("Gradle", Build, 0.90),
        s("Git", Build, 0.90),
        s("GitHub Actions", Build, 0.90),
        s("JUnit", Build, 0.85),
        s("Mockito", Build, 0.85),
        s("Maven", Build, 0.80),
        s("Playwright", Build, 0.60),
        r("pnpm", Build, 0.85),
        r("Checkstyle", Build, 0.85),
        r("ESLint", Build, 0.80),
        r("Prettier", Build, 0.80),
        r("Bun", Build, 0.75),
        r("Renovate", Build, 0.75),
        r("release-please", Build, 0.75),
        r("SonarQube", Build, 0.75),
        r("Docker Buildx", Build, 0.75),
        r("Flyway", Build, 0.70),
        r("Testcontainers", Build, 0.70),
        r("Trivy", Build, 0.70),
        r("pre-commit", Build, 0.65),
        r("JaCoCo", Build, 0.65),
        r("Codecov", Build, 0.65),
        r("Hadolint", Build, 0.65),
        r("npm", Build, 0.60),
        r("Cargo", Build, 0.60),
        r("Zizmor", Build, 0.60),
        r("Jest", Build, 0.55),
        r("commitlint", Build, 0.50),
        r("Knip", Build, 0.50),
        r("Husky", Build, 0.50),
        r("lint-staged", Build, 0.45),
        r("Vitest", Build, 0.40),
        // infrastructure
        s("Docker", Infra, 0.85),
        s("Kubernetes", Infra, 0.80),
        s("ArgoCD", Infra, 0.80),
        s("PostgreSQL", Infra, 0.80),
        s("Helm", Infra, 0.80),
        s("Linux", Infra, 0.75),
        s("TimescaleDB", Infra, 0.75),
        s("MongoDB", Infra, 0.65),
        s("Redis", Infra, 0.60),
        r("Cert-Manager", Infra, 0.80),
        r("Docker Compose", Infra, 0.80),
        r("OpenTelemetry", Infra, 0.80),
        r("ExternalDNS", Infra, 0.75),
        r("MetalLB", Infra, 0.75),
        r("Sealed Secrets", Infra, 0.75),
        r("Loki", Infra, 0.75),
        r("Tempo", Infra, 0.75),
        r("Traefik", Infra, 0.75),
        r("OpenEBS", Infra, 0.70),
        r("MariaDB", Infra, 0.70),
        r("MySQL", Infra, 0.70),
        r("Sentry", Infra, 0.70),
        r("MinIO", Infra, 0.65),
        r("CrowdSec", Infra, 0.65),
        r("SQLite", Infra, 0.65),
        r("Prometheus", Infra, 0.65),
        r("Harbor", Infra, 0.60),
        r("Cloudflare Tunnels", Infra, 0.60),
        r("AWS S3", Infra, 0.55),
        r("Grafana", Infra, 0.50),
        r("Nginx", Infra, 0.50),
        r("Docker Hub", Infra, 0.50),
        r("Apache Kafka", Infra, 0.45),
        r("Cloudflare Workers", Infra, 0.45),
        r("Reverse Proxies (general)", Infra, 0.45),
        r("Elasticsearch", Infra, 0.35),
        r("RabbitMQ", Infra, 0.35),
        r("Pingora", Infra, 0.30),
    ]
};

/// Skills shown in the skill matrix and on the resume, strongest first.
pub fn matrix_skills() -> Vec<Skill> {
    let mut out: Vec<Skill> = SKILLS
        .iter()
        .filter(|s| !s.radar_only && s.confidence >= MIN_CONFIDENCE)
        .copied()
        .collect();
    out.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

// ---------- Experience & education ----------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct YearMonth {
    pub year: u16,
    pub month: u8,
}

const fn ym(year: u16, month: u8) -> YearMonth {
    YearMonth { year, month }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Experience {
    /// Stable id used as translation key segment (`experience.entries.<id>`).
    /// The localized role, organisation and bullets live in the i18n files.
    pub id: &'static str,
    pub location: &'static str,
    pub start: YearMonth,
    pub end: Option<YearMonth>,
    /// Number of localized bullets (`…bullets.b1` ..= `…bullets.b<n>`).
    pub bullet_count: u8,
    /// Caps how many localized bullets the generated resume renders
    /// (`None` = uncapped). Only trims the PDF; the website always shows every
    /// bullet via [`bullet_count`](Self::bullet_count). Used to drop a redundant
    /// third bullet from the two oldest roles on the resume.
    pub resume_bullet_cap: Option<u8>,
    pub tech: &'static [&'static str],
}

/// Raw work-history entries. The declaration order here is *not* significant:
/// the site and resume never read this slice directly but go through
/// [`experiences_sorted`], which derives the canonical order. Localized
/// roles/bullets live in the i18n files.
pub const EXPERIENCE: &[Experience] = &[
    Experience {
        id: "sixtwenty",
        location: "Remote",
        start: ym(2026, 3),
        end: None,
        bullet_count: 2,
        resume_bullet_cap: None,
        tech: &["Java", "TypeScript", "Rust", "Kubernetes", "GitOps"],
    },
    Experience {
        id: "mineplex-studios",
        location: "Remote",
        start: ym(2023, 8),
        end: Some(ym(2026, 3)),
        bullet_count: 3,
        resume_bullet_cap: None,
        tech: &["Java", "TypeScript", "Rust", "Kubernetes", "GitOps"],
    },
    Experience {
        id: "self-employed",
        location: "Remote",
        start: ym(2021, 10),
        end: None,
        bullet_count: 3,
        resume_bullet_cap: None,
        tech: &[
            "Java",
            "Spring Boot",
            "TypeScript",
            "Node.js",
            "Rust",
            "gRPC",
        ],
    },
    Experience {
        id: "mineplex-dev",
        location: "Remote",
        start: ym(2021, 10),
        end: Some(ym(2023, 1)),
        bullet_count: 3,
        resume_bullet_cap: Some(2),
        tech: &["Java", "PaperMC", "Bukkit API", "Spigot API"],
    },
    Experience {
        id: "mineplex-qa",
        location: "Remote",
        start: ym(2018, 11),
        end: Some(ym(2023, 1)),
        bullet_count: 3,
        resume_bullet_cap: Some(2),
        tech: &["Java", "QA", "Testing"],
    },
];

/// Work history in the canonical order the site and resume render: ongoing
/// roles first (no end date), then the rest in reverse-chronological order by
/// start date, with the most recent end date breaking equal-start ties. The
/// order is derived entirely from the dates, so an ongoing role can outrank an
/// ended one that started later.
pub fn experiences_sorted() -> Vec<&'static Experience> {
    let mut out: Vec<&'static Experience> = EXPERIENCE.iter().collect();
    out.sort_by(|a, b| {
        // Ongoing roles (no end) come first.
        a.end.is_some()
            .cmp(&b.end.is_some())
            // Then most recent start first.
            .then_with(|| (b.start.year, b.start.month).cmp(&(a.start.year, a.start.month)))
            // Finally, the most recent end breaks equal-start ties.
            .then_with(|| match (a.end, b.end) {
                (Some(ae), Some(be)) => (be.year, be.month).cmp(&(ae.year, ae.month)),
                _ => std::cmp::Ordering::Equal,
            })
    });
    out
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Education {
    /// Stable id used as translation key segment (`resume.education.<id>`).
    pub id: &'static str,
    pub start: YearMonth,
    pub end: Option<YearMonth>,
}

/// Education history, newest first. Degrees/institutions are localized.
pub const EDUCATION: &[Education] = &[
    Education {
        id: "uni-konstanz",
        start: ym(2019, 10),
        end: Some(ym(2021, 9)),
    },
    Education {
        id: "abitur",
        start: ym(2016, 9),
        end: Some(ym(2019, 7)),
    },
];

/// Formats a date range like "Nov 2018 – Jan 2023" from localized month names.
///
/// `months` must contain the twelve localized month abbreviations (Jan..Dec);
/// `present` is the localized label for open-ended ranges.
pub fn format_period(
    start: YearMonth,
    end: Option<YearMonth>,
    months: &[String],
    present: &str,
) -> String {
    let fmt = |d: YearMonth| {
        let month = months
            .get((d.month as usize).saturating_sub(1))
            .map(String::as_str)
            .unwrap_or("");
        format!("{month} {year}", year = d.year)
    };
    match end {
        Some(end) => format!("{} – {}", fmt(start), fmt(end)),
        None => format!("{} – {present}", fmt(start)),
    }
}

/// Compact year-based range like "2018 — 2023" or "2026 — now", as used by
/// the web experience accordion (the PDF uses [`format_period`]).
pub fn format_period_years(start: YearMonth, end: Option<YearMonth>, now: &str) -> String {
    match end {
        Some(end) if end.year == start.year => format!("{}", start.year),
        Some(end) => format!("{} — {}", start.year, end.year),
        None => format!("{} — {now}", start.year),
    }
}

// ---------- repos.json schema ----------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Repo {
    pub name: String,
    #[serde(default)]
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub stargazers_count: u32,
    #[serde(default)]
    pub forks_count: u32,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub fork: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub homepage: Option<String>,
}

impl Repo {
    pub fn is_featured(&self) -> bool {
        CONFIG
            .featured_repos
            .iter()
            .any(|f| f.eq_ignore_ascii_case(&self.name))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReposFile {
    pub generated_at: String,
    pub user: String,
    pub repos: Vec<Repo>,
}

/// GitHub-style language color for a repo card.
pub fn lang_color(lang: &str) -> &'static str {
    match lang {
        "Rust" => "#dea584",
        "Java" => "#b07219",
        "TypeScript" => "#3178c6",
        "JavaScript" => "#f1e05a",
        "Python" => "#3572A5",
        "Shell" => "#89e051",
        "Go" => "#00ADD8",
        "HTML" => "#e34c26",
        "CSS" => "#563d7c",
        "Dockerfile" => "#384d54",
        "PLpgSQL" => "#336790",
        "Helm" => "#0f1689",
        "Kotlin" => "#A97BFF",
        "C#" => "#178600",
        "C++" => "#f34b7d",
        "Lua" => "#000080",
        "YAML" => "#cb171e",
        "Smarty" => "#f0c040",
        _ => "#9aa4b2",
    }
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
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
    fn experiences_sorted_prioritises_ongoing_then_recent_start() {
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
