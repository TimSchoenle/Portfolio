//! Language-neutral portfolio data shared by the frontend and the resume generator.
//!
//! All user-visible prose lives in the embedded translation files ([`I18N_EN`],
//! [`I18N_DE`]); this crate only holds facts (names, dates, URLs, confidence
//! values) and the schemas for the build-time generated documents — `repos.json`
//! and the third-party licence inventory in [`licenses`].
//!
//! Nothing here is read at run time. Every item is a compile-time constant or a schema for a
//! document produced during the image build, so changing any of it is a redeploy.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod licenses;
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

/// File name of the generated Open Graph card, which is also the path it is
/// served from (`/og-image.png`).
pub const OG_IMAGE_FILE: &str = "og-image.png";

/// Pixel size of that card: the 1.91:1 box every major link-preview consumer
/// crops to.
///
/// Shared because both ends of the pipeline need it and they have to agree —
/// `resume-generator` renders the image at exactly this size, and the web app
/// declares it in `og:image:width` / `og:image:height` so a consumer can reserve
/// the space before the bytes arrive. A card that disagrees with its own
/// declared size is letterboxed or dropped, and nothing warns about it.
pub const OG_IMAGE_SIZE: (u32, u32) = (1200, 630);

/// The resume file name for a language code, falling back to English.
///
/// ```
/// # use portfolio_data::resume_file;
/// assert_eq!(resume_file("de"), "Tim-Schönle-Lebenslauf.pdf");
/// assert_eq!(resume_file("fr"), resume_file("en"));
/// ```
pub fn resume_file(lang: &str) -> &'static str {
    RESUME_FILES
        .iter()
        .find(|(l, _)| *l == lang)
        .map(|(_, f)| *f)
        .unwrap_or(RESUME_FILES[0].1)
}

/// SHA-256 checksums of the generated resume PDFs.
///
/// Written by the resume generator as `resume-fingerprint.json` and embedded
/// into the frontend at build time, where it is shown on the contact card so a
/// downloaded resume can be verified against its published digest.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResumeFingerprints {
    /// Hash algorithm used for the digests (e.g. "SHA-256").
    pub algorithm: String,
    /// RFC 3339 timestamp of when the manifest was generated.
    pub generated_at: String,
    /// Resume file name (e.g. "Tim-Schönle-Resume.pdf") -> hex digest.
    pub files: BTreeMap<String, String>,
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
}

// ---------- Site configuration ----------

/// A processor the privacy policy has to name.
///
/// Nothing renders one. The GDPR wants these facts in the reader's own language, so the prose
/// that states them is in the translation files; this is the copy that prose is checked against.
pub struct ExternalService {
    /// The legal entity, spelled as it appears in its own imprint.
    pub name: &'static str,
    /// Registered seat, in the format that country writes addresses in.
    pub address: &'static str,
    /// The processor's own privacy policy, which the prose links to.
    pub policy_url: &'static str,
}

/// What the imprint and privacy pages are obliged to state.
///
/// Five of these reach a rendered page. The rest is duplicated inside the translated prose for
/// the reason [`ExternalService`] gives, and is kept here so there is one place to correct when
/// a provider moves or a retention period changes.
pub struct Legal {
    /// Postal address used in the imprint and as GDPR controller address.
    pub address_lines: &'static [&'static str],
    /// VAT identification number. German law obliges the imprint to show it.
    pub vat_id: &'static str,
    /// Second contact channel required by the imprint service.
    pub second_contact_url: &'static str,
    /// Who runs the machine the site is served from.
    pub hosting: ExternalService,
    /// Who runs the edge the site is delivered through.
    pub cloudflare: ExternalService,
    /// How long access logs are kept before deletion.
    pub log_retention_days: u32,
    /// ISO date shown as "last updated" on the imprint page.
    pub imprint_last_change: &'static str,
    /// ISO date shown as "last updated" on the privacy page.
    pub privacy_last_change: &'static str,
    /// Supervisory authority for data-protection complaints.
    pub authority_url: &'static str,
    /// EU online dispute resolution platform.
    pub odr_url: &'static str,
}

/// The name, as a bare literal.
///
/// A macro rather than a `const` because [`CONFIG`]`.title` is `concat!`-ed from this and
/// [`job_title`], and `concat!` takes literals only. Nothing outside this module should expand
/// it: [`CONFIG`]`.full_name` is the same bytes and is the field everything reads.
macro_rules! full_name {
    () => {
        "Tim Schönle"
    };
}

/// The role, as a bare literal, for the reason [`full_name`] gives.
///
/// This is the *English* spelling and the one canonical statement of the seniority. Three
/// surfaces are derived from it without restating it — [`CONFIG`]`.job_title` for schema.org and
/// the profile API, [`CONFIG`]`.title` for `og:title` and the document head, and `common.jobTitle`
/// in [`I18N_EN`], which the `english_job_title_matches_config` test holds equal to it. The German
/// translation of the same fact is `common.jobTitle` in [`I18N_DE`].
///
/// Before this was one literal it was three, and they disagreed: the resume PDF said "Senior
/// Software Developer" while `og:title`, the profile API and the summary all said "Software
/// Developer".
macro_rules! job_title {
    () => {
        "Senior Software Developer"
    };
}

/// What separates the name from the role in a document title, as a literal, so the `concat!` in
/// [`CONFIG`] and the `format!` in [`document_title`] cannot punctuate the same fact differently.
macro_rules! title_separator {
    () => {
        " — "
    };
}

/// What separates the name from the role in a document title.
pub const TITLE_SEPARATOR: &str = title_separator!();

/// `"{full_name}{TITLE_SEPARATOR}{role}"`, for a title in the caller's language.
///
/// [`CONFIG`]`.title` is this function's English result, folded at compile time;
/// the `document_title_matches_config` test holds the two together. The resume generator calls
/// this with the *translated* role, so the German PDF carries a German document title.
#[must_use]
pub fn document_title(role: &str) -> String {
    format!("{}{TITLE_SEPARATOR}{role}", CONFIG.full_name)
}

/// Who the site is about, and every address that identifies them.
///
/// One value exists, [`CONFIG`]. Everything that names the person behind the site reads it: the
/// document head, the resume PDF, the social card and the profile API.
pub struct Config {
    /// Name as it is set in print: the resume header, the social card, `og:site_name`.
    pub full_name: &'static str,
    /// Given name alone, which is what the profile API's `name` carries.
    pub name: &'static str,
    /// Full document title, `"{full_name} — {job_title}"`, used verbatim for `og:title`.
    ///
    /// `concat!`-ed from the same two literals the other two fields are, so it cannot state a
    /// name or a seniority that they do not.
    pub title: &'static str,
    /// Role on its own, e.g. for schema.org `jobTitle` (the full document
    /// `title` is `"{full_name} — {job_title}"`).
    pub job_title: &'static str,
    /// Contact country exposed by the profile API and structured metadata.
    pub location: &'static str,
    /// Published contact address, on the contact card and in the profile API.
    pub email: &'static str,
    /// Canonical origin, with a scheme and no trailing slash. Every absolute URL the site emits
    /// is built by appending a path to it.
    pub url: &'static str,
    /// GitHub profile page. Contains [`github_username`](Self::github_username), which a test
    /// asserts, because the two are rendered as one link.
    pub github: &'static str,
    /// The account `update-repos` lists repositories for unless `github.username` names another.
    pub github_username: &'static str,
    /// LinkedIn profile page, linked from the imprint and from the resume sidebar.
    pub linkedin: &'static str,
    /// This repository, which the footer colophon links to.
    pub repository: &'static str,
    /// The sentence that becomes `meta description`, `og:description` and the social card's last
    /// line. Written to survive being cut at about 155 characters, which is where search results
    /// truncate it.
    pub description: &'static str,
    /// The three technologies the site leads with, in the order the hero prints them.
    ///
    /// Rendered as the hero eyebrow, joined with ` · `. It was a translation key until it became
    /// this field, and the two translation files held byte-identical values for it — a tool's
    /// name is not translated, so it was never prose. Every entry must also appear in
    /// [`keywords`](Self::keywords), which
    /// the `headline_tech_is_covered_by_keywords` test enforces, so the line a visitor reads and
    /// the line a crawler reads cannot name different stacks.
    pub headline_tech: &'static [&'static str],
    /// `meta keywords`, joined with `, `.
    pub keywords: &'static [&'static str],
    /// Repositories pinned to the front of the projects section, matched case-insensitively.
    pub featured_repos: &'static [&'static str],
    /// Repositories that must never appear in `repos.json`, regardless of their
    /// activity. Matched case-insensitively by name when listing all of the
    /// user's repositories in `update-repos`.
    pub blacklisted_repos: &'static [&'static str],
    /// The imprint and privacy facts.
    pub legal: Legal,
}

/// The site's own identity. No configuration key reaches any of it: changing one is a redeploy.
pub const CONFIG: Config = Config {
    full_name: full_name!(),
    name: "Tim",
    title: concat!(full_name!(), title_separator!(), job_title!()),
    job_title: job_title!(),
    location: "Germany",
    email: "contact@tim-schoenle.de",
    url: "https://tim-schoenle.de",
    github: "https://github.com/timschoenle",
    github_username: "timschoenle",
    linkedin: "https://www.linkedin.com/in/tim-schoenle",
    repository: "https://github.com/timschoenle/Portfolio",
    description: concat!(
        full_name!(),
        title_separator!(),
        job_title!(),
        " building scalable backend systems in Java, Rust and TypeScript, shipped end to end on \
         Kubernetes.",
    ),
    headline_tech: &["Java", "Rust", "TypeScript", "Kubernetes"],
    keywords: &[
        "Tim Schönle",
        job_title!(),
        "Java",
        "Spring Boot",
        "Rust",
        "TypeScript",
        "SQL",
        "Kubernetes",
        "Backend Engineering",
        "GitOps",
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

/// A region of the tech radar, which is also a group of the skill matrix.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Quadrant {
    /// Programming languages, and the markup and configuration formats filed with them
    /// (`Markdown`, `TOML`, `Dockerfile`).
    Languages,
    /// Libraries and frameworks. A datastore's client library is filed here and the datastore
    /// itself under [`Infra`](Self::Infra): `sqlx` and `Prisma` against `PostgreSQL`.
    Frameworks,
    /// Build, test and repository tooling. The one quadrant the profile API withholds; see
    /// [`profile::ProfileSkills`].
    Build,
    /// Runtimes, datastores and the deployment machinery around them.
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

    /// Every quadrant, in the order the radar draws them.
    pub fn all() -> [Quadrant; 4] {
        [
            Quadrant::Languages,
            Quadrant::Frameworks,
            Quadrant::Build,
            Quadrant::Infra,
        ]
    }
}

/// One entry of the skill inventory.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Skill {
    /// Shown as written, in both languages. A tool's name is not translated.
    pub name: &'static str,
    /// Which radar region it is plotted in, and which matrix group it is listed under.
    pub quadrant: Quadrant,
    /// 0.0..=1.0, mirrored from the original portfolio's skills data.
    pub confidence: f32,
    /// Radar-only skills appear as radar scatter but not in the matrix/resume.
    pub radar_only: bool,
}

impl Skill {
    /// Confidence scaled by five and rounded to the nearest integer.
    ///
    /// Clamped at both ends, so a skill nobody would list still comes back as a 1 rather than a
    /// 0 the star row would render as nothing.
    ///
    /// ```
    /// # use portfolio_data::{Quadrant, Skill};
    /// # fn skill(confidence: f32) -> Skill {
    /// #     Skill { name: "x", quadrant: Quadrant::Languages, confidence, radar_only: false }
    /// # }
    /// assert_eq!(skill(0.95).level(), 5);
    /// assert_eq!(skill(0.70).level(), 4);
    /// assert_eq!(skill(0.05).level(), 1);
    /// ```
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

/// A month on the experience and education timelines.
///
/// Deliberately no day. Every date this site renders is `Mon YYYY` or `YYYY`, so a start day
/// would be a fact nobody publishes and nothing corrects.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct YearMonth {
    /// Four-digit calendar year.
    pub year: u16,
    /// `1..=12`. [`format_period`] indexes the localized month names with it.
    pub month: u8,
}

const fn ym(year: u16, month: u8) -> YearMonth {
    YearMonth { year, month }
}

/// One role in the work history.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Experience {
    /// Stable id used as translation key segment (`experience.entries.<id>`).
    /// The localized role, organisation and bullets live in the i18n files.
    pub id: &'static str,
    /// Where the role is worked from, e.g. `Remote`. Not localized.
    pub location: &'static str,
    /// First month of the role.
    pub start: YearMonth,
    /// Last month, or `None` while the role is ongoing. Ongoing roles sort ahead of every ended
    /// one in [`experiences_sorted`], however recently the ended one started.
    pub end: Option<YearMonth>,
    /// Number of localized bullets (`…bullets.b1` ..= `…bullets.b<n>`).
    pub bullet_count: u8,
    /// Caps how many localized bullets the generated resume renders
    /// (`None` = uncapped). Only trims the PDF; the website always shows every
    /// bullet via [`bullet_count`](Self::bullet_count). Used to drop a redundant
    /// third bullet from the two oldest roles on the resume.
    pub resume_bullet_cap: Option<u8>,
    /// The chip run under the entry, in the order given.
    pub tech: &'static [&'static str],
}

/// Raw work-history entries, in no meaningful order.
///
/// The site and the resume never read this slice directly. Both go through
/// [`experiences_sorted`], which derives the rendering order from the dates, so an entry appended
/// here lands wherever its dates put it. Localized roles and bullets live in the i18n files.
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

/// Work history in the order the site and the resume render it.
///
/// Ongoing roles first, then the rest by descending start date, with the most recent end date
/// breaking equal starts. All of that comes from the dates, so an ongoing role outranks an ended
/// one that started later.
pub fn experiences_sorted() -> Vec<&'static Experience> {
    let mut out: Vec<&'static Experience> = EXPERIENCE.iter().collect();
    out.sort_by(|a, b| {
        // Ongoing roles (no end) come first.
        a.end
            .is_some()
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

/// One entry of the education history. Degree and institution are localized.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Education {
    /// Stable id used as translation key segment (`resume.education.<id>`).
    pub id: &'static str,
    /// First month of the programme.
    pub start: YearMonth,
    /// Last month, or `None` while it is ongoing.
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

/// A date range in the caller's language, e.g. `Nov 2018 – Jan 2023`.
///
/// `months` are the twelve month abbreviations in order, and `present` is the label an
/// open-ended range gets in place of an end date. A month above twelve, or a `months` shorter
/// than twelve, renders the month blank; month `0` renders the first entry of `months`.
///
/// ```
/// # use portfolio_data::{YearMonth, format_period};
/// let months: Vec<String> = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
///                            "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
///     .iter().map(|m| (*m).to_owned()).collect();
/// let ym = |year, month| YearMonth { year, month };
///
/// assert_eq!(
///     format_period(ym(2018, 11), Some(ym(2023, 1)), &months, "Present"),
///     "Nov 2018 – Jan 2023",
/// );
/// assert_eq!(
///     format_period(ym(2026, 3), None, &months, "Present"),
///     "Mar 2026 – Present",
/// );
/// ```
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

/// One repository, as `update-repos` read it out of the GitHub REST API.
///
/// This deserializes GitHub's own response, which is why the field names are GitHub's and why
/// all but three default rather than failing the listing. The same type is then written to
/// `repos.json` and read back by the web binary, so the projects section and the API response
/// cannot describe different documents.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Repo {
    /// Repository name without the owner, e.g. `Portfolio`.
    pub name: String,
    /// `owner/name`.
    #[serde(default)]
    pub full_name: String,
    /// GitHub's own description. `None` for a repository that declares none, and the card then
    /// renders without a summary line.
    pub description: Option<String>,
    /// The repository page the card links to.
    pub html_url: String,
    /// GitHub's primary-language guess, coloured by [`lang_color`]. `None` for an empty
    /// repository.
    #[serde(default)]
    pub language: Option<String>,
    /// Stars when the listing was taken, not live.
    #[serde(default)]
    pub stargazers_count: u32,
    /// Forks at that same moment.
    #[serde(default)]
    pub forks_count: u32,
    /// RFC 3339 timestamp of the last push. `update-repos` drops anything older than a year.
    #[serde(default)]
    pub updated_at: String,
    /// GitHub topics, rendered as chips.
    #[serde(default)]
    pub topics: Vec<String>,
    /// Whether the repository is a fork. Carried but never filtered on, so a fork in a generated
    /// listing is one that was meant to be there.
    #[serde(default)]
    pub fork: bool,
    /// Whether GitHub has archived it. `false` for every repository an account listing produced,
    /// since that is what the filtering removes; a repository named in `github.repos` is written
    /// through unfiltered.
    #[serde(default)]
    pub archived: bool,
    /// The site the repository declares, linked beside the source link.
    #[serde(default)]
    pub homepage: Option<String>,
}

impl Repo {
    /// Whether [`CONFIG`] pins this repository to the front of the projects section. Matched
    /// case-insensitively by name.
    pub fn is_featured(&self) -> bool {
        CONFIG
            .featured_repos
            .iter()
            .any(|f| f.eq_ignore_ascii_case(&self.name))
    }
}

/// The generated `repos.json`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReposFile {
    /// RFC 3339 timestamp the listing was taken at. `update-repos` reads it back on the next
    /// build to decide whether the file is fresh enough to skip the API call.
    pub generated_at: String,
    /// The account the repositories were listed for.
    pub user: String,
    /// The repositories. Most recently updated first when the account was listed, and in the
    /// configured order when `github.repos` named them.
    pub repos: Vec<Repo>,
}

/// The colour GitHub paints a language in, for the dot on a repository card.
///
/// The values are linguist's, copied rather than fetched, so a card matches what a visitor sees
/// on github.com. An unrecognised language gets a neutral grey rather than no dot.
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
    /// Two roles once shared two bullets word for word, which read as copy-paste on the resume
    /// and left the current role with no distinct content of its own.
    #[test]
    fn experience_bullets_are_distinct() {
        for (lang, json) in [("en", I18N_EN), ("de", I18N_DE)] {
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

    /// The English copy is en-US throughout.
    ///
    /// It was previously both: `containerised` and `containerized` appeared five lines apart,
    /// and `Specialising` sat above `specializing`.
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
    /// is a worse trade than editing the sentence. One bullet ran to 38 words in a single sentence
    /// before this existed.
    #[test]
    fn experience_bullets_stay_within_two_lines() {
        const MAX_WORDS: usize = 24;
        let mut offences = Vec::new();
        for (lang, json) in [("en", I18N_EN), ("de", I18N_DE)] {
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
    /// The first six were in the copy: the hero called itself "passionate about building great
    /// software", and both the about text and the resume summary said "growing experience in
    /// Rust" beside a workspace written in Rust. The last two have never appeared here and are
    /// listed because they are the next two of the same kind.
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
        for (lang, json) in [("en", I18N_EN), ("de", I18N_DE)] {
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
