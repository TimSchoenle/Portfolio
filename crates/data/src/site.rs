//! Site-wide facts: identity, links, SEO metadata and the generated social assets.

/// The file name of the generated Open Graph card, which is served at `/` plus this name.
pub const OG_IMAGE_FILE: &str = "og-image.png";

/// The web manifest's raster icons as `(edge length in pixels, file name)`.
///
/// Each is served at `/` plus its file name, and `portfolio-resume` renders them from the favicon.
/// Browsers offer to install a site only when its manifest lists a 192 and a 512 pixel raster
/// icon.
pub const APP_ICONS: [(u32, &str); 2] = [(192, "icon-192.png"), (512, "icon-512.png")];

/// The pixel size of the Open Graph card, the 1.91:1 box every major link-preview consumer crops
/// to.
///
/// `portfolio-resume` renders the image at this size, and the web app declares it in
/// `og:image:width` and `og:image:height`. A card that disagrees with its declared size is
/// letterboxed or dropped, and nothing warns about it.
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

/// The text between the name and the role in a document title.
pub const TITLE_SEPARATOR: &str = title_separator!();

/// Returns the document title for `role` in the caller's language, as
/// `"{full_name}{TITLE_SEPARATOR}{role}"`.
///
/// [`CONFIG`]`.title` is the English result, folded at compile time.
///
/// ```
/// # use portfolio_data::{CONFIG, document_title};
/// assert_eq!(document_title("Senior Software Developer"), CONFIG.title);
/// ```
#[must_use]
pub fn document_title(role: &str) -> String {
    format!("{}{TITLE_SEPARATOR}{role}", CONFIG.full_name)
}

/// Who the site is about, and every address that identifies them.
///
/// The only value is [`CONFIG`]. Everything that names the person behind the site reads it: the
/// document head, the resume PDF, the social card and the profile API.
pub struct Config {
    /// The name as it is set in print: the resume header, the social card, `og:site_name`.
    pub full_name: &'static str,
    /// The given name alone, which the profile API's `name` carries.
    pub name: &'static str,
    /// The full document title, `"{full_name} — {job_title}"`, used verbatim for `og:title`.
    ///
    /// It is concatenated from the same literals as the other two fields, so it cannot state a
    /// name or a seniority that they do not.
    pub title: &'static str,
    /// The role on its own, in English, as schema.org `jobTitle` and the profile API publish it.
    pub job_title: &'static str,
    /// The contact country the profile API and the structured metadata publish.
    pub location: &'static str,
    /// The published contact address, on the contact card and in the profile API.
    pub email: &'static str,
    /// The canonical origin, with a scheme and no trailing slash.
    ///
    /// Every absolute URL the site emits is this plus a path.
    pub url: &'static str,
    /// The GitHub profile page, which contains [`github_username`](Self::github_username).
    ///
    /// The two are rendered as one link, and a test holds them together.
    pub github: &'static str,
    /// The account `portfolio-update-repos` lists repositories for, unless `github.username` names
    /// another.
    pub github_username: &'static str,
    /// The LinkedIn profile page, linked from the footer and from the resume sidebar.
    pub linkedin: &'static str,
    /// This repository, which the footer colophon links to.
    pub repository: &'static str,
    /// The sentence that becomes `meta description`, `og:description` and the social card's last
    /// line.
    ///
    /// Search results cut it at about 155 characters, and a test holds it between 110 and 160.
    pub description: &'static str,
    /// The technologies the site leads with, in the order the hero eyebrow prints them.
    ///
    /// The eyebrow joins them with ` · `, untranslated. Every entry must also appear in
    /// [`keywords`](Self::keywords), which a test enforces, so the line a visitor reads and the
    /// line a crawler reads name the same stack.
    pub headline_tech: &'static [&'static str],
    /// The `meta keywords` entries, joined with `, `.
    pub keywords: &'static [&'static str],
    /// The repositories pinned to the front of the projects section, matched case-insensitively.
    pub featured_repos: &'static [&'static str],
    /// The repositories an account listing leaves out of `repos.json`, matched case-insensitively.
    ///
    /// A repository named in `github.repos` is written through regardless.
    pub blacklisted_repos: &'static [&'static str],
}

/// The site's own identity.
///
/// No configuration key reaches any of it, so changing a value is a redeploy.
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
