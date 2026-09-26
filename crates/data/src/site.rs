//! Site-wide facts: identity, links, SEO metadata and the generated social assets.

/// File name of the generated Open Graph card, which is also the path it is
/// served from (`/og-image.png`).
pub const OG_IMAGE_FILE: &str = "og-image.png";

/// The web manifest's raster icons as `(edge length in pixels, file name)`.
///
/// Each is served from `/<file name>`. Browsers offer to install a site only when its manifest
/// lists a 192 and a 512 pixel raster icon; `resume-generator` renders them from the favicon.
pub const APP_ICONS: [(u32, &str); 2] = [(192, "icon-192.png"), (512, "icon-512.png")];

/// Pixel size of that card: the 1.91:1 box every major link-preview consumer
/// crops to.
///
/// Shared because both ends of the pipeline need it and they have to agree —
/// `resume-generator` renders the image at exactly this size, and the web app
/// declares it in `og:image:width` / `og:image:height` so a consumer can reserve
/// the space before the bytes arrive. A card that disagrees with its own
/// declared size is letterboxed or dropped, and nothing warns about it.
pub const OG_IMAGE_SIZE: (u32, u32) = (1200, 630);

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
    /// LinkedIn profile page, linked from the footer and from the resume sidebar.
    pub linkedin: &'static str,
    /// This repository, which the footer colophon links to.
    pub repository: &'static str,
    /// The sentence that becomes `meta description`, `og:description` and the social card's last
    /// line. Written to survive being cut at about 155 characters, which is where search results
    /// truncate it.
    pub description: &'static str,
    /// The three technologies the site leads with, in the order the hero prints them.
    ///
    /// Rendered as the hero eyebrow, joined with ` · `. Not a translation key, because a tool's
    /// name is the same in every language. Every entry must also appear in
    /// [`keywords`](Self::keywords), which the `headline_tech_is_covered_by_keywords` test
    /// enforces, so the line a visitor reads and the line a crawler reads name the same stack.
    pub headline_tech: &'static [&'static str],
    /// `meta keywords`, joined with `, `.
    pub keywords: &'static [&'static str],
    /// Repositories pinned to the front of the projects section, matched case-insensitively.
    pub featured_repos: &'static [&'static str],
    /// Repositories that must never appear in `repos.json`, regardless of their
    /// activity. Matched case-insensitively by name when listing all of the
    /// user's repositories in `update-repos`.
    pub blacklisted_repos: &'static [&'static str],
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
};
